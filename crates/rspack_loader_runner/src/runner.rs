use std::{fmt::Debug, path::PathBuf, sync::Arc};

use rspack_error::{Diagnostic, Error, Result, error};
use rspack_fs::ReadableFileSystem;
use rspack_sources::SourceMap;
use rustc_hash::FxHashSet as HashSet;
use tokio::task::spawn_blocking;
use tracing::{Instrument, info_span};

use crate::{
  ParseMeta,
  content::{AdditionalData, Content, ResourceData},
  context::{LoaderContext, State},
  loader::{Loader, LoaderItem},
  plugin::LoaderRunnerPlugin,
};

impl<Context: Send> LoaderContext<Context> {
  async fn start_yielding(&mut self) -> Result<bool> {
    if let Some(plugin) = &self.plugin
      && plugin.should_yield(self).await?
    {
      plugin.clone().start_yielding(self).await?;
      return Ok(true);
    }
    Ok(false)
  }
}

#[tracing::instrument("LoaderRunner:process_resource",
  skip_all,
  fields(resource = loader_context.resource_data.resource())
)]
async fn process_resource<Context: Send>(
  loader_context: &mut LoaderContext<Context>,
  fs: Arc<dyn ReadableFileSystem>,
) -> Result<()> {
  if let Some(plugin) = &loader_context.plugin
    && let Some(processed_resource) = plugin
      .process_resource(&loader_context.resource_data)
      .await?
  {
    loader_context.content = Some(processed_resource);
    return Ok(());
  }

  let resource_data = &loader_context.resource_data;
  let scheme = resource_data.get_scheme();
  if scheme.is_none() {
    if let Some(resource_path) = resource_data.path()
      && !resource_path.as_str().is_empty()
    {
      let resource_path_owned = resource_path.to_owned();
      // use spawn_blocking to avoid block, see https://docs.rs/tokio/latest/src/tokio/fs/read.rs.html#48
      let result = spawn_blocking(move || fs.read_sync(resource_path_owned.as_path()))
        .await
        .map_err(|e| error!("{e}, spawn task failed"))?;
      let result = result.map_err(|e| error!("{e}, failed to read {resource_path}"))?;
      loader_context.content = Some(Content::from(result));
    }
    return Ok(());
  }

  let resource = resource_data.resource();
  Err(error!(
    r#"Reading from "{resource}" is not handled by plugins (Unhandled scheme).
Rspack supports "data:" and "file:" URIs by default.
You may need an additional plugin to handle "{scheme}:" URIs."#
  ))
}

fn create_loader_context<Context: Send>(
  loader_items: Vec<LoaderItem<Context>>,
  resource_data: Arc<ResourceData>,
  plugin: Option<Arc<dyn LoaderRunnerPlugin<Context = Context>>>,
  context: Context,
) -> LoaderContext<Context> {
  let mut file_dependencies: HashSet<PathBuf> = Default::default();
  if let Some(resource_path) = resource_data.path()
    && resource_path.is_absolute()
  {
    file_dependencies.insert(resource_path.to_owned().into_std_path_buf());
  }

  LoaderContext {
    hot: false,
    cacheable: true,
    parse_meta: Default::default(),
    file_dependencies,
    context_dependencies: Default::default(),
    missing_dependencies: Default::default(),
    build_dependencies: Default::default(),
    content: None,
    context,
    source_map: None,
    additional_data: None,
    state: State::Init,
    loader_index: 0,
    loader_items,
    plugin,
    resource_data,
    diagnostics: vec![],
  }
}

#[tracing::instrument("LoaderRunner:run_loaders", skip_all, level = "trace")]
pub async fn run_loaders<Context: Send>(
  loaders: Vec<Arc<dyn Loader<Context>>>,
  resource_data: Arc<ResourceData>,
  plugin: Option<Arc<dyn LoaderRunnerPlugin<Context = Context>>>,
  context: Context,
  fs: Arc<dyn ReadableFileSystem>,
) -> (LoaderResult, Option<Error>) {
  let loaders = loaders
    .into_iter()
    .map(|i| i.into())
    .collect::<Vec<LoaderItem<Context>>>();
  let mut cx = create_loader_context(loaders, resource_data, plugin, context);
  let result = run_loaders_impl(&mut cx, fs).await;
  (LoaderResult::new(cx), result.err())
}

async fn run_loaders_impl<Context: Send>(
  cx: &mut LoaderContext<Context>,
  fs: Arc<dyn ReadableFileSystem>,
) -> Result<()> {
  if let Some(plugin) = cx.plugin.clone() {
    plugin.before_all(cx).await?;
  }
  let resource = cx.resource().to_owned();
  let resource = resource.as_str();
  loop {
    match cx.state {
      State::Init => {
        cx.state.transition(State::Pitching);
      }
      State::Pitching => {
        if cx.loader_index >= cx.loader_items.len() as i32 {
          cx.state.transition(State::ProcessResource);
          continue;
        }
        let span = info_span!("run_loader:pitch:yield_to_js", resource);
        if cx.start_yielding().instrument(span).await? {
          if cx.content.is_some() {
            cx.state.transition(State::Normal);
            cx.loader_index -= 1;
          }
          continue;
        }

        if cx.current_loader().pitch_executed() {
          cx.loader_index += 1;
          continue;
        }

        cx.current_loader().set_pitch_executed();
        let loader = cx.current_loader().loader().clone();
        let span = info_span!("run_loader:pitch", resource);
        loader.pitch(cx).instrument(span).await?;
        if cx.content.is_some() {
          cx.state.transition(State::Normal);
          cx.loader_index -= 1;
          continue;
        }
      }
      State::ProcessResource => {
        let span = info_span!("run_loader:process_resource", resource);
        process_resource(cx, fs.clone()).instrument(span).await?;
        cx.loader_index = cx.loader_items.len() as i32 - 1;
        cx.state.transition(State::Normal);
      }
      State::Normal => {
        if cx.loader_index < 0 {
          cx.state.transition(State::Finished);
          continue;
        }

        if cx.loader_index == 0 && cx.current_loader().normal_executed() {
          cx.state.transition(State::Finished);
          continue;
        }
        let span = info_span!("run_loader:yield_to_js", resource);
        if cx.start_yielding().instrument(span).await? {
          continue;
        }

        if cx.current_loader().normal_executed() {
          cx.loader_index -= 1;
          continue;
        }

        cx.current_loader().set_normal_executed();
        let loader = cx.current_loader().loader().clone();

        let span = info_span!("run_loader:normal", resource);
        loader.run(cx).instrument(span).await?;
        if !cx.current_loader().finish_called() {
          // If nothing is returned from this loader,
          // we set everything to [None] and move to the next loader.
          // This mocks the behavior of webpack loader-runner.
          cx.finish_with_empty();
        }
      }
      State::Finished => break,
    }
  }

  if cx.content.is_none() {
    if !cx.loader_items.is_empty() {
      let loader = cx.loader_items[0].to_string();
      return Err(error!(
        "Final loader({loader}) didn't return a Buffer or String"
      ));
    } else {
      panic!("content should be available");
    }
  }

  Ok(())
}

#[derive(Debug)]
pub struct LoaderResult {
  pub cacheable: bool,
  pub file_dependencies: HashSet<PathBuf>,
  pub context_dependencies: HashSet<PathBuf>,
  pub missing_dependencies: HashSet<PathBuf>,
  pub build_dependencies: HashSet<PathBuf>,
  pub diagnostics: Vec<Diagnostic>,
  pub content: Content,
  pub source_map: Option<SourceMap>,
  pub additional_data: Option<AdditionalData>,
  pub parse_meta: ParseMeta,
}

impl LoaderResult {
  pub fn new<T: Send>(loader_context: LoaderContext<T>) -> Self {
    LoaderResult {
      cacheable: loader_context.cacheable,
      file_dependencies: loader_context.file_dependencies,
      context_dependencies: loader_context.context_dependencies,
      missing_dependencies: loader_context.missing_dependencies,
      build_dependencies: loader_context.build_dependencies,
      diagnostics: loader_context.diagnostics,
      content: loader_context
        .content
        .unwrap_or(Content::String(String::new())),
      source_map: loader_context.source_map,
      additional_data: loader_context.additional_data,
      parse_meta: loader_context.parse_meta,
    }
  }
}

#[cfg(test)]
mod test {
  use std::{cell::RefCell, sync::Arc};

  use rspack_cacheable::{cacheable, cacheable_dyn};
  use rspack_collections::Identifier;
  use rspack_error::Result;
  use rspack_fs::NativeFileSystem;

  use super::{Loader, LoaderContext, ResourceData, run_loaders};
  use crate::{AdditionalData, content::Content, plugin::LoaderRunnerPlugin};

  struct TestContentPlugin;

  #[async_trait::async_trait]
  impl LoaderRunnerPlugin for TestContentPlugin {
    type Context = ();

    fn name(&self) -> &'static str {
      "test-content"
    }

    async fn before_all(&self, _context: &mut LoaderContext<Self::Context>) -> Result<()> {
      Ok(())
    }

    async fn process_resource(&self, _resource_data: &ResourceData) -> Result<Option<Content>> {
      Ok(Some(Content::Buffer(vec![])))
    }
  }

  #[tokio::test]
  async fn should_have_the_right_execution_order() {
    thread_local! {
      static IDENTS: RefCell<Vec<String>> = RefCell::default();
    }

    #[cacheable]
    struct Pitching;

    #[cacheable_dyn]
    #[async_trait::async_trait]
    impl Loader<()> for Pitching {
      fn identifier(&self) -> Identifier {
        "/rspack/pitching-loader1".into()
      }

      async fn pitch(&self, _loader_context: &mut LoaderContext<()>) -> Result<()> {
        IDENTS.with(|i| i.borrow_mut().push("pitch1".to_string()));
        Ok(())
      }
    }

    #[cacheable]
    struct Pitching2;

    #[cacheable_dyn]
    #[async_trait::async_trait]
    impl Loader<()> for Pitching2 {
      fn identifier(&self) -> Identifier {
        "/rspack/pitching-loader2".into()
      }

      async fn pitch(&self, _loader_context: &mut LoaderContext<()>) -> Result<()> {
        IDENTS.with(|i| i.borrow_mut().push("pitch2".to_string()));
        Ok(())
      }
    }

    #[cacheable]
    struct Normal;

    #[cacheable_dyn]
    #[async_trait::async_trait]
    impl Loader<()> for Normal {
      fn identifier(&self) -> Identifier {
        "/rspack/normal-loader1".into()
      }

      async fn run(&self, _loader_context: &mut LoaderContext<()>) -> Result<()> {
        IDENTS.with(|i| i.borrow_mut().push("normal1".to_string()));
        Ok(())
      }
    }

    #[cacheable]
    struct Normal2;

    #[cacheable_dyn]
    #[async_trait::async_trait]
    impl Loader<()> for Normal2 {
      fn identifier(&self) -> Identifier {
        "/rspack/normal-loader2".into()
      }

      async fn run(&self, _loader_context: &mut LoaderContext<()>) -> Result<()> {
        IDENTS.with(|i| i.borrow_mut().push("normal2".to_string()));
        Ok(())
      }
    }

    #[cacheable]
    struct PitchNormalBase;

    #[cacheable_dyn]
    #[async_trait::async_trait]
    impl Loader<()> for PitchNormalBase {
      fn identifier(&self) -> Identifier {
        "/rspack/pitch-normal-base-loader".into()
      }

      async fn run(&self, _loader_context: &mut LoaderContext<()>) -> Result<()> {
        IDENTS.with(|i| i.borrow_mut().push("pitch-normal-base-normal".to_string()));
        Ok(())
      }

      async fn pitch(&self, _loader_context: &mut LoaderContext<()>) -> Result<()> {
        IDENTS.with(|i| i.borrow_mut().push("pitch-normal-base-pitch".to_string()));
        Ok(())
      }
    }

    #[cacheable]
    struct PitchNormal;

    #[cacheable_dyn]
    #[async_trait::async_trait]
    impl Loader<()> for PitchNormal {
      fn identifier(&self) -> Identifier {
        "/rspack/pitch-normal-loader".into()
      }

      async fn run(&self, _loader_context: &mut LoaderContext<()>) -> Result<()> {
        IDENTS.with(|i| i.borrow_mut().push("pitch-normal-normal".to_string()));
        Ok(())
      }

      async fn pitch(&self, loader_context: &mut LoaderContext<()>) -> Result<()> {
        IDENTS.with(|i| i.borrow_mut().push("pitch-normal-pitch".to_string()));
        loader_context.content = Some(Content::Buffer(vec![]));
        Ok(())
      }
    }

    #[cacheable]
    struct PitchNormal2;

    #[cacheable_dyn]
    #[async_trait::async_trait]
    impl Loader<()> for PitchNormal2 {
      fn identifier(&self) -> Identifier {
        "/rspack/pitch-normal-2-loader".into()
      }

      async fn run(&self, _loader_context: &mut LoaderContext<()>) -> Result<()> {
        IDENTS.with(|i| i.borrow_mut().push("pitch-normal-normal-2".to_string()));
        Ok(())
      }

      async fn pitch(&self, loader_context: &mut LoaderContext<()>) -> Result<()> {
        IDENTS.with(|i| i.borrow_mut().push("pitch-normal-pitch-2".to_string()));
        loader_context.content = Some(Content::Buffer(vec![]));
        Ok(())
      }
    }

    let c1 = Arc::new(Normal) as Arc<dyn Loader<()>>;
    let c2 = Arc::new(Normal2) as Arc<dyn Loader<()>>;
    let p1 = Arc::new(Pitching) as Arc<dyn Loader<()>>;
    let p2 = Arc::new(Pitching2) as Arc<dyn Loader<()>>;

    let rs = Arc::new(ResourceData::new_with_resource(
      "/rspack/main.js?abc=123#efg".to_owned(),
    ));

    // Ignore error: Final loader didn't return a Buffer or String
    assert!(
      run_loaders(
        vec![p1, p2, c1, c2],
        rs.clone(),
        Some(Arc::new(TestContentPlugin)),
        (),
        Arc::new(NativeFileSystem::new(false))
      )
      .await
      .1
      .is_some()
    );
    IDENTS.with(|i| assert_eq!(*i.borrow(), &["pitch1", "pitch2", "normal2", "normal1"]));
    IDENTS.with(|i| i.borrow_mut().clear());

    let p1 = Arc::new(PitchNormalBase) as Arc<dyn Loader<()>>;
    let p2 = Arc::new(PitchNormal) as Arc<dyn Loader<()>>;
    let p3 = Arc::new(PitchNormal2) as Arc<dyn Loader<()>>;

    // Ignore error: Final loader didn't return a Buffer or String
    assert!(
      run_loaders(
        vec![p1, p2, p3],
        rs.clone(),
        Some(Arc::new(TestContentPlugin)),
        (),
        Arc::new(NativeFileSystem::new(false))
      )
      .await
      .1
      .is_some()
    );
    IDENTS.with(|i| {
      // should not execute p3, as p2 pitched successfully.
      assert!(!i.borrow().contains(&"pitch-normal-normal-2".to_string()));
      assert!(!i.borrow().contains(&"pitch-normal-pitch-2".to_string()));
      // should skip normal stage of p2.
      assert!(!i.borrow().contains(&"pitch-normal-normal".to_string()));
      // should still run the normal stage of pitch normal base.
      assert_eq!(i.borrow()[0], "pitch-normal-base-pitch".to_string());
      assert_eq!(i.borrow()[2], "pitch-normal-base-normal".to_string());
      // p2 pitched successfully.
      assert_eq!(i.borrow()[1], "pitch-normal-pitch".to_string());
    });
  }

  #[tokio::test]
  async fn should_able_to_consume_additional_data() {
    #[cacheable]
    struct Normal;

    #[cacheable_dyn]
    #[async_trait::async_trait]
    impl Loader<()> for Normal {
      fn identifier(&self) -> Identifier {
        "/rspack/normal-loader1".into()
      }

      async fn run(&self, loader_context: &mut LoaderContext<()>) -> Result<()> {
        let data = loader_context
          .additional_data
          .as_ref()
          .unwrap()
          .get::<&str>()
          .unwrap();
        assert_eq!(*data, "additional-data");
        loader_context.finish_with(("".to_string(), None, None));
        Ok(())
      }
    }

    #[cacheable]
    struct Normal2;

    #[cacheable_dyn]
    #[async_trait::async_trait]
    impl Loader<()> for Normal2 {
      fn identifier(&self) -> Identifier {
        "/rspack/normal-loader2".into()
      }

      async fn run(&self, loader_context: &mut LoaderContext<()>) -> Result<()> {
        let mut additional_data: AdditionalData = Default::default();
        additional_data.insert("additional-data");
        loader_context.finish_with(("".to_string(), None, Some(additional_data)));
        Ok(())
      }
    }

    let rs = Arc::new(ResourceData::new_with_resource(
      "/rspack/main.js?abc=123#efg".to_owned(),
    ));

    assert!(
      run_loaders(
        vec![Arc::new(Normal) as Arc<dyn Loader>, Arc::new(Normal2)],
        rs,
        Some(Arc::new(TestContentPlugin)),
        (),
        Arc::new(NativeFileSystem::new(false)),
      )
      .await
      .1
      .is_none()
    );
  }

  #[tokio::test]
  async fn should_override_data_if_finish_with_is_not_called() {
    #[cacheable]
    struct Normal;

    #[cacheable_dyn]
    #[async_trait::async_trait]
    impl Loader<()> for Normal {
      fn identifier(&self) -> Identifier {
        "/rspack/normal-loader1".into()
      }

      async fn run(&self, loader_context: &mut LoaderContext<()>) -> Result<()> {
        assert!(loader_context.content.is_some());
        // Does not call `LoaderContext::finish_with`
        Ok(())
      }
    }

    let rs = Arc::new(ResourceData::new_with_resource(
      "/rspack/main.js?abc=123#efg".to_owned(),
    ));

    #[cacheable]
    struct Normal2;

    #[cacheable_dyn]
    #[async_trait::async_trait]
    impl Loader<()> for Normal2 {
      fn identifier(&self) -> Identifier {
        "/rspack/normal-loader2".into()
      }

      async fn run(&self, loader_context: &mut LoaderContext<()>) -> Result<()> {
        let (content, source_map, additional_data) = loader_context.take_all();
        assert!(content.is_none());
        assert!(source_map.is_none());
        assert!(additional_data.is_none());
        Ok(())
      }
    }

    // Ignore error: Final loader didn't return a Buffer or String
    assert!(
      run_loaders(
        vec![Arc::new(Normal2), Arc::new(Normal)],
        rs,
        Some(Arc::new(TestContentPlugin)),
        (),
        Arc::new(NativeFileSystem::new(false))
      )
      .await
      .1
      .is_some()
    );
  }
}
