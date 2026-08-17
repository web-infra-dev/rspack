use std::{
  marker::PhantomPinned,
  path::PathBuf,
  sync::{Arc, OnceLock},
};

use derive_more::Debug;
use rspack_cacheable::{cacheable, with::Skip};
use rspack_error::{Diagnostic, Error, Result, error};
use rspack_fs::ReadableFileSystem;
use rspack_paths::Utf8PathBuf;
use rspack_sources::SourceMap;
use rspack_util::ArcComputed;
use rustc_hash::FxHashSet as HashSet;
use tracing::{Instrument, info_span};

use crate::{
  LoaderChain, ParseMeta,
  content::{AdditionalData, Content, ResourceData},
  context::{LoaderContext, State},
  loader::{Loader, LoaderItem, LoaderItemState},
  plan_loader_chains,
  plugin::LoaderRunnerPlugin,
};

#[cacheable]
#[derive(Debug)]
pub struct Loaders<Context: Send> {
  #[debug(skip)]
  loaders: Vec<Arc<dyn Loader<Context>>>,
  #[cacheable(with=Skip)]
  loader_items: OnceLock<Vec<LoaderItem<Context>>>,
  #[cacheable(with=Skip)]
  loader_chains: OnceLock<Vec<LoaderChain>>,
  #[cacheable(with=Skip)]
  _pin: PhantomPinned,
}

impl<Context: Send> Loaders<Context> {
  pub fn new(loaders: Vec<Arc<dyn Loader<Context>>>) -> Self {
    Self {
      loaders,
      loader_items: OnceLock::new(),
      loader_chains: OnceLock::new(),
      _pin: PhantomPinned,
    }
  }

  pub fn loaders(&self) -> &[Arc<dyn Loader<Context>>] {
    &self.loaders
  }

  fn loader_items(&self) -> &Vec<LoaderItem<Context>> {
    self
      .loader_items
      .get_or_init(|| self.loaders.iter().cloned().map(Into::into).collect())
  }

  fn loader_chains(&self) -> &Vec<LoaderChain> {
    self
      .loader_chains
      .get_or_init(|| plan_loader_chains(self.loader_items()))
  }

  #[tracing::instrument("LoaderRunner:run_loaders", skip_all, level = "trace")]
  pub async fn run_loaders(
    self: &Arc<Self>,
    resource_data: Arc<ResourceData>,
    plugin: Option<Arc<dyn LoaderRunnerPlugin<Context = Context>>>,
    context: Context,
    fs: Arc<dyn ReadableFileSystem>,
  ) -> (LoaderResult<Context>, Option<Error>) {
    let loader_items = ArcComputed::new(Arc::clone(self), Loaders::loader_items);
    let loader_chains = ArcComputed::new(Arc::clone(self), Loaders::loader_chains);
    let mut cx = create_loader_context(loader_items, loader_chains, resource_data, plugin, context);
    let result = run_loaders_impl(&mut cx, fs).await;
    (LoaderResult::new(cx), result.err())
  }
}

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

async fn run_pitch_chain<Context: Send>(
  cx: &mut LoaderContext<Context>,
  resource: &str,
) -> Result<()> {
  let chain = cx
    .current_execution_chain()
    .cloned()
    .expect("pitching requires a current loader chain");
  let chain_end = chain.end() as i32;
  let span = info_span!(
    "run_loader_chain:pitch",
    resource,
    chain_len = chain.len(),
    chain_start = chain.start(),
    chain_end = chain.end(),
    execution_kind = ?chain.execution_kind(),
  );

  async {
    while cx.loader_index < chain_end {
      let yield_span = info_span!("run_loader:pitch:yield_to_js", resource);
      if cx.start_yielding().instrument(yield_span).await? {
        if cx.content.is_some() {
          break;
        }
        continue;
      }

      if cx.current_loader_state().pitch_executed() {
        cx.loader_index += 1;
        continue;
      }

      cx.set_current_loader_pitch_executed();
      let loader = cx.current_loader().loader().clone();
      let loader_span = info_span!("run_loader:pitch", resource);
      loader.pitch(cx).instrument(loader_span).await?;
      if cx.content.is_some() {
        break;
      }
    }
    Ok(())
  }
  .instrument(span)
  .await
}

async fn run_normal_chain<Context: Send>(
  cx: &mut LoaderContext<Context>,
  resource: &str,
) -> Result<()> {
  let chain = cx
    .current_chain()
    .cloned()
    .expect("normal execution requires a current loader chain");
  let chain_start = chain.start() as i32;
  let span = info_span!(
    "run_loader_chain:normal",
    resource,
    chain_len = chain.len(),
    chain_start = chain.start(),
    chain_end = chain.end(),
    execution_kind = ?chain.execution_kind(),
  );

  async {
    while cx.loader_index >= chain_start {
      let yield_span = info_span!("run_loader:yield_to_js", resource);
      if cx.start_yielding().instrument(yield_span).await? {
        continue;
      }

      if cx.current_loader_state().normal_executed() {
        cx.loader_index -= 1;
        continue;
      }

      cx.set_current_loader_normal_executed();
      let loader = cx.current_loader().loader().clone();
      let loader_span = info_span!("run_loader:normal", resource);
      loader.run(cx).instrument(loader_span).await?;
      if !cx.current_loader_state().finish_called() {
        // If nothing is returned from this loader, set every output to None to
        // match webpack loader-runner behavior.
        cx.finish_with_empty();
      }
    }
    Ok(())
  }
  .instrument(span)
  .await
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
    loader_context.file_dependencies.extend(file_dependencies);
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
  loader_items: ArcComputed<Loaders<Context>, Vec<LoaderItem<Context>>>,
  loader_chains: ArcComputed<Loaders<Context>, Vec<LoaderChain>>,
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

  let loader_item_states = (0..loader_items.len())
    .map(|_| LoaderItemState::default())
    .collect();
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
    loader_chains,
    loader_item_states,
    plugin,
    resource_data,
    diagnostics: vec![],
  }
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
        if cx.loader_index >= cx.loader_items().len() as i32 {
          cx.state.transition(State::ProcessResource);
          continue;
        }
        run_pitch_chain(cx, resource).await?;
        if cx.content.is_some() {
          cx.state.transition(State::Normal);
          cx.loader_index -= 1;
        }
      }
      State::ProcessResource => {
        let span = info_span!("run_loader:process_resource", resource);
        process_resource(cx, fs.clone()).instrument(span).await?;
        cx.loader_index = cx.loader_items().len() as i32 - 1;
        cx.state.transition(State::Normal);
      }
      State::Normal => {
        if cx.loader_index < 0 {
          cx.state.transition(State::Finished);
          continue;
        }

        run_normal_chain(cx, resource).await?;
      }
      State::Finished => break,
    }
  }

  if cx.content.is_none() {
    if !cx.loader_items().is_empty() {
      let loader = cx.loader_items()[0].to_string();
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
  pub file_dependencies: HashSet<PathBuf>,
  pub context_dependencies: HashSet<PathBuf>,
  pub missing_dependencies: HashSet<PathBuf>,
  pub build_dependencies: HashSet<PathBuf>,
  pub diagnostics: Vec<Diagnostic>,
  pub content: Content,
  pub source_map: Option<Box<SourceMap<'static>>>,
  pub additional_data: Option<AdditionalData>,
  pub parse_meta: ParseMeta,
  pub current_loader: Option<Utf8PathBuf>,
}

impl<Context: Send> LoaderResult<Context> {
  pub fn new(loader_context: LoaderContext<Context>) -> Self {
    let current_loader = (loader_context.loader_index >= 0)
      .then(|| {
        loader_context
          .loader_items()
          .get(loader_context.loader_index as usize)
      })
      .flatten()
      .map(|loader| loader.path().to_path_buf());
    LoaderResult {
      context: loader_context.context,
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
      current_loader,
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
  use rspack_sources::SourceMap;
  use rustc_hash::FxHashSet as HashSet;

  use super::{Loader, LoaderContext, Loaders, ResourceData};
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
    ) -> Result<
      Option<(
        Content,
        Option<SourceMap<'static>>,
        HashSet<std::path::PathBuf>,
      )>,
    > {
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
      Arc::new(Loaders::new(vec![p1, p2, c1, c2]))
        .run_loaders(
          rs.clone(),
          Some(Arc::new(TestContentPlugin)),
          (),
          Arc::new(NativeFileSystem::new(false)),
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
      Arc::new(Loaders::new(vec![p1, p2, p3]))
        .run_loaders(
          rs.clone(),
          Some(Arc::new(TestContentPlugin)),
          (),
          Arc::new(NativeFileSystem::new(false)),
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
      Arc::new(Loaders::new(vec![
        Arc::new(Normal) as Arc<dyn Loader>,
        Arc::new(Normal2),
      ]))
      .run_loaders(
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
      Arc::new(Loaders::new(vec![Arc::new(Normal2), Arc::new(Normal)]))
        .run_loaders(
          rs,
          Some(Arc::new(TestContentPlugin)),
          (),
          Arc::new(NativeFileSystem::new(false)),
        )
        .await
        .1
        .is_some()
    );
  }
}
