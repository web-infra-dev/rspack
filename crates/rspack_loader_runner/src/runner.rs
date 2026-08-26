use std::{fmt::Debug, sync::Arc};

use rspack_error::{Diagnostic, Error, Result, error};
use rspack_fs::ReadableFileSystem;
use rspack_paths::Utf8PathBuf;
use rspack_sources::SourceMap;
use tracing::{Instrument, info_span};

use crate::{
  LoaderRunnerOptions, ParseMeta,
  content::{AdditionalData, Content, ResourceData},
  context::{LoaderContext, LoaderDependencyContext, State},
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
    && let Some((content, source_map, file_dependencies)) = plugin
      .process_resource(&loader_context.resource_data, fs)
      .await?
  {
    loader_context.content = Some(content);
    loader_context.source_map = source_map.map(Box::new);
    for dependency in file_dependencies {
      loader_context.add_file_dependency(dependency);
    }
    loader_context.merge_dependency_context_changes();
    return Ok(());
  }

  let resource_data = &loader_context.resource_data;
  let scheme = resource_data.get_scheme();

  if scheme.is_none() {
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
  let mut dependency_context = LoaderDependencyContext::default();
  if let Some(resource_path) = resource_data.path()
    && resource_path.is_absolute()
  {
    dependency_context.file.insert(resource_path.into());
  }

  LoaderContext {
    hot: false,
    cacheable: true,
    parse_meta: Default::default(),
    dependency_context,
    added_dependency_context: Default::default(),
    removed_dependency_context: Default::default(),
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
  loader_options: Option<Vec<LoaderRunnerOptions>>,
  resource_data: Arc<ResourceData>,
  plugin: Option<Arc<dyn LoaderRunnerPlugin<Context = Context>>>,
  context: Context,
  fs: Arc<dyn ReadableFileSystem>,
) -> (LoaderResult<Context>, Option<Error>) {
  let loaders = if let Some(loader_options) = loader_options {
    assert_eq!(
      loaders.len(),
      loader_options.len(),
      "loader options must stay aligned with loaders"
    );
    loaders
      .into_iter()
      .zip(loader_options)
      .map(|(loader, options)| LoaderItem::new(loader, options))
      .collect::<Vec<LoaderItem<Context>>>()
  } else {
    loaders.into_iter().map(LoaderItem::from).collect()
  };
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
        cx.reset_dependency_context_changes();
        let result = loader.pitch(cx).instrument(span).await;
        cx.merge_dependency_context_changes();
        result?;
        if cx.content.is_some() {
          cx.state.transition(State::Normal);
          cx.loader_index -= 1;
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
        cx.reset_dependency_context_changes();
        let result = if let Some(plugin) = cx.plugin.clone() {
          plugin.run_normal_loader(cx, loader).instrument(span).await
        } else {
          let result = loader.run(cx).instrument(span).await;
          if result.is_ok() && !cx.current_loader().finish_called() {
            // If nothing is returned from this loader,
            // we set everything to [None] and move to the next loader.
            // This mocks the behavior of webpack loader-runner.
            cx.finish_with_empty();
          }
          result
        };
        cx.merge_dependency_context_changes();
        result?;
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
pub struct LoaderResult<Context> {
  pub context: Context,
  pub cacheable: bool,
  pub dependency_context: LoaderDependencyContext,
  pub diagnostics: Vec<Diagnostic>,
  pub content: Content,
  pub source_map: Option<Box<SourceMap<'static>>>,
  pub additional_data: Option<AdditionalData>,
  pub parse_meta: ParseMeta,
  pub current_loader: Option<Utf8PathBuf>,
}

impl<Context: Send> LoaderResult<Context> {
  pub fn new(loader_context: LoaderContext<Context>) -> Self {
    LoaderResult {
      context: loader_context.context,
      cacheable: loader_context.cacheable,
      dependency_context: loader_context.dependency_context,
      diagnostics: loader_context.diagnostics,
      content: loader_context
        .content
        .unwrap_or(Content::String(String::new())),
      source_map: loader_context.source_map,
      additional_data: loader_context.additional_data,
      parse_meta: loader_context.parse_meta,
      current_loader: (loader_context.loader_index >= 0)
        .then(|| {
          loader_context
            .loader_items
            .get(loader_context.loader_index as usize)
        })
        .flatten()
        .map(|loader| loader.path().to_path_buf()),
    }
  }
}

#[cfg(test)]
mod test {
  use std::{cell::RefCell, sync::Arc};

  use rspack_cacheable::{cacheable, cacheable_dyn};
  use rspack_collections::Identifier;
  use rspack_error::Result;
  use rspack_fs::{NativeFileSystem, ReadableFileSystem};
  use rspack_paths::InternedPathSet;
  use rspack_sources::SourceMap;

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

    async fn process_resource(
      &self,
      _resource_data: &ResourceData,
      _fs: Arc<dyn ReadableFileSystem>,
    ) -> Result<Option<(Content, Option<SourceMap<'static>>, InternedPathSet)>> {
      Ok(Some((Content::Buffer(vec![]), None, Default::default())))
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
        None,
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
        None,
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
        loader_context.finish_with((String::new(), None, None));
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
        loader_context.finish_with((String::new(), None, Some(additional_data)));
        Ok(())
      }
    }

    let rs = Arc::new(ResourceData::new_with_resource(
      "/rspack/main.js?abc=123#efg".to_owned(),
    ));

    assert!(
      run_loaders(
        vec![Arc::new(Normal) as Arc<dyn Loader>, Arc::new(Normal2)],
        None,
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
        None,
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
