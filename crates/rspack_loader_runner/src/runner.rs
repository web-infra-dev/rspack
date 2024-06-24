use std::{fmt::Debug, path::PathBuf, sync::Arc};

use rspack_error::{error, IntoTWithDiagnosticArray, Result, TWithDiagnosticArray};
use rspack_sources::SourceMap;
use rustc_hash::FxHashSet as HashSet;

use crate::{
  content::{AdditionalData, Content, ResourceData},
  context::{LoaderContext, State},
  loader::{Loader, LoaderItem},
  plugin::LoaderRunnerPlugin,
};

impl<Context> LoaderContext<Context> {
  async fn start_yielding(&mut self) -> Result<bool> {
    if let Some(plugin) = &self.plugin
      && plugin.should_yield(self)?
    {
      plugin.clone().start_yielding(self).await?;
      return Ok(true);
    }
    Ok(false)
  }
}

async fn process_resource<Context: Send>(
  loader_context: &mut LoaderContext<Context>,
) -> Result<()> {
  if let Some(plugin) = &loader_context.plugin
    && let Some(processed_resource) = plugin
      .process_resource(&loader_context.resource_data)
      .await?
  {
    loader_context.content = Some(processed_resource);
  }

  let resource_data = &loader_context.resource_data;
  if loader_context.content.is_none() {
    if !resource_data.resource_path.to_string_lossy().is_empty() {
      let result = tokio::fs::read(&loader_context.resource_data.resource_path)
        .await
        .map_err(|e| {
          let r = loader_context
            .resource_data
            .resource_path
            .to_string_lossy()
            .to_string();
          error!("{e}, failed to read {r}")
        })?;
      loader_context.content = Some(Content::from(result));
    } else if !resource_data.get_scheme().is_none() {
      let resource = &resource_data.resource;
      let scheme = resource_data.get_scheme();
      return Err(error!(
        r#"Reading from "{resource}" is not handled by plugins (Unhandled scheme).
Rspack supports "data:" and "file:" URIs by default.
You may need an additional plugin to handle "{scheme}:" URIs."#
      ));
    }
  }

  Ok(())
}

async fn create_loader_context<Context: 'static>(
  loader_items: Vec<LoaderItem<Context>>,
  resource_data: Arc<ResourceData>,
  plugin: Option<Arc<dyn LoaderRunnerPlugin<Context = Context>>>,
  context: Context,
  additional_data: AdditionalData,
) -> Result<LoaderContext<Context>> {
  let mut file_dependencies: HashSet<PathBuf> = Default::default();
  if resource_data.resource_path.is_absolute() {
    file_dependencies.insert(resource_data.resource_path.clone());
  }

  let mut loader_context = LoaderContext {
    hot: false,
    cacheable: true,
    file_dependencies,
    context_dependencies: Default::default(),
    missing_dependencies: Default::default(),
    build_dependencies: Default::default(),
    asset_filenames: Default::default(),
    content: None,
    context,
    source_map: None,
    additional_data,
    state: State::Init,
    loader_index: 0,
    loader_items,
    plugin,
    resource_data,
    diagnostics: vec![],
  };

  if let Some(plugin) = loader_context.plugin.clone() {
    plugin.before_all(&mut loader_context)?;
  }

  Ok(loader_context)
}

pub async fn run_loaders<Context: 'static + Send>(
  loaders: Vec<Arc<dyn Loader<Context>>>,
  resource_data: Arc<ResourceData>,
  plugins: Option<Arc<dyn LoaderRunnerPlugin<Context = Context>>>,
  context: Context,
  additional_data: AdditionalData,
) -> Result<TWithDiagnosticArray<LoaderResult>> {
  let loaders = loaders
    .iter()
    .map(|i| i.clone().into())
    .collect::<Vec<LoaderItem<Context>>>();

  let mut cx =
    create_loader_context(loaders, resource_data, plugins, context, additional_data).await?;

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

        if cx.start_yielding().await? {
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
        loader.pitch(&mut cx).await?;
        if cx.content.is_some() {
          cx.state.transition(State::Normal);
          cx.loader_index -= 1;
          continue;
        }
      }
      State::ProcessResource => {
        process_resource(&mut cx).await?;
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

        if cx.start_yielding().await? {
          continue;
        }

        if cx.current_loader().normal_executed() {
          cx.loader_index -= 1;
          continue;
        }

        cx.current_loader().set_normal_executed();
        let loader = cx.current_loader().loader().clone();
        loader.run(&mut cx).await?;
      }
      State::Finished => break,
    }
  }

  cx.try_into()
}

#[derive(Debug)]
pub struct LoaderResult {
  pub cacheable: bool,
  pub file_dependencies: HashSet<PathBuf>,
  pub context_dependencies: HashSet<PathBuf>,
  pub missing_dependencies: HashSet<PathBuf>,
  pub build_dependencies: HashSet<PathBuf>,
  pub asset_filenames: HashSet<String>,
  pub content: Content,
  pub source_map: Option<SourceMap>,
  pub additional_data: AdditionalData,
}

impl<Context> TryFrom<LoaderContext<Context>> for TWithDiagnosticArray<LoaderResult> {
  type Error = rspack_error::Error;
  fn try_from(loader_context: LoaderContext<Context>) -> std::result::Result<Self, Self::Error> {
    let content = loader_context.content.ok_or_else(|| {
      if !loader_context.loader_items.is_empty() {
        let loader = loader_context.loader_items[0].to_string();
        error!("Final loader({loader}) didn't return a Buffer or String")
      } else {
        panic!("content should be available");
      }
    })?;

    Ok(
      LoaderResult {
        cacheable: loader_context.cacheable,
        file_dependencies: loader_context.file_dependencies,
        context_dependencies: loader_context.context_dependencies,
        missing_dependencies: loader_context.missing_dependencies,
        build_dependencies: loader_context.build_dependencies,
        asset_filenames: loader_context.asset_filenames,
        content,
        source_map: loader_context.source_map,
        additional_data: loader_context.additional_data,
      }
      .with_diagnostic(loader_context.diagnostics),
    )
  }
}

#[cfg(test)]
mod test {
  use std::{cell::RefCell, sync::Arc};

  use once_cell::sync::OnceCell;
  use rspack_error::Result;
  use rspack_identifier::{Identifiable, Identifier};

  use super::{run_loaders, Loader, LoaderContext, ResourceData};
  use crate::{content::Content, plugin::LoaderRunnerPlugin};

  struct TestContentPlugin;

  #[async_trait::async_trait]
  impl LoaderRunnerPlugin for TestContentPlugin {
    type Context = ();

    fn name(&self) -> &'static str {
      "test-content"
    }

    fn before_all(&self, _context: &mut LoaderContext<Self::Context>) -> Result<()> {
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

    struct Pitching;

    impl Identifiable for Pitching {
      fn identifier(&self) -> Identifier {
        "/rspack/pitching-loader1".into()
      }
    }

    #[async_trait::async_trait]
    impl Loader<()> for Pitching {
      async fn pitch(&self, _loader_context: &mut LoaderContext<()>) -> Result<()> {
        IDENTS.with(|i| i.borrow_mut().push("pitch1".to_string()));
        Ok(())
      }
    }

    struct Pitching2;

    impl Identifiable for Pitching2 {
      fn identifier(&self) -> Identifier {
        "/rspack/pitching-loader2".into()
      }
    }

    #[async_trait::async_trait]
    impl Loader<()> for Pitching2 {
      async fn pitch(&self, _loader_context: &mut LoaderContext<()>) -> Result<()> {
        IDENTS.with(|i| i.borrow_mut().push("pitch2".to_string()));
        Ok(())
      }
    }

    struct Normal;

    impl Identifiable for Normal {
      fn identifier(&self) -> Identifier {
        "/rspack/normal-loader1".into()
      }
    }

    #[async_trait::async_trait]
    impl Loader<()> for Normal {
      async fn run(&self, _loader_context: &mut LoaderContext<()>) -> Result<()> {
        IDENTS.with(|i| i.borrow_mut().push("normal1".to_string()));
        Ok(())
      }
    }

    struct Normal2;

    impl Identifiable for Normal2 {
      fn identifier(&self) -> Identifier {
        "/rspack/normal-loader2".into()
      }
    }

    #[async_trait::async_trait]
    impl Loader<()> for Normal2 {
      async fn run(&self, _loader_context: &mut LoaderContext<()>) -> Result<()> {
        IDENTS.with(|i| i.borrow_mut().push("normal2".to_string()));
        Ok(())
      }
    }

    struct PitchNormalBase;

    impl Identifiable for PitchNormalBase {
      fn identifier(&self) -> Identifier {
        "/rspack/pitch-normal-base-loader".into()
      }
    }

    #[async_trait::async_trait]
    impl Loader<()> for PitchNormalBase {
      async fn run(&self, _loader_context: &mut LoaderContext<()>) -> Result<()> {
        IDENTS.with(|i| i.borrow_mut().push("pitch-normal-base-normal".to_string()));
        Ok(())
      }

      async fn pitch(&self, _loader_context: &mut LoaderContext<()>) -> Result<()> {
        IDENTS.with(|i| i.borrow_mut().push("pitch-normal-base-pitch".to_string()));
        Ok(())
      }
    }

    struct PitchNormal;

    impl Identifiable for PitchNormal {
      fn identifier(&self) -> Identifier {
        "/rspack/pitch-normal-loader".into()
      }
    }

    #[async_trait::async_trait]
    impl Loader<()> for PitchNormal {
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

    struct PitchNormal2;

    impl Identifiable for PitchNormal2 {
      fn identifier(&self) -> Identifier {
        "/rspack/pitch-normal-2-loader".into()
      }
    }

    #[async_trait::async_trait]
    impl Loader<()> for PitchNormal2 {
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

    let rs = Arc::new(ResourceData {
      scheme: OnceCell::new(),
      resource: "/rspack/main.js?abc=123#efg".to_owned(),
      resource_description: None,
      resource_fragment: None,
      resource_query: None,
      resource_path: Default::default(),
      mimetype: None,
      parameters: None,
      encoding: None,
      encoded_content: None,
    });

    run_loaders(
      vec![p1, p2, c1, c2],
      rs.clone(),
      Some(Arc::new(TestContentPlugin)),
      (),
      Default::default(),
    )
    .await
    .unwrap();
    IDENTS.with(|i| assert_eq!(*i.borrow(), &["pitch1", "pitch2", "normal2", "normal1"]));
    IDENTS.with(|i| i.borrow_mut().clear());

    let p1 = Arc::new(PitchNormalBase) as Arc<dyn Loader<()>>;
    let p2 = Arc::new(PitchNormal) as Arc<dyn Loader<()>>;
    let p3 = Arc::new(PitchNormal2) as Arc<dyn Loader<()>>;

    run_loaders(
      vec![p1, p2, p3],
      rs.clone(),
      Some(Arc::new(TestContentPlugin)),
      (),
      Default::default(),
    )
    .await
    .unwrap();
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
    struct Normal;

    impl Identifiable for Normal {
      fn identifier(&self) -> Identifier {
        "/rspack/normal-loader1".into()
      }
    }

    #[async_trait::async_trait]
    impl Loader<()> for Normal {
      async fn run(&self, loader_context: &mut LoaderContext<()>) -> Result<()> {
        let data = loader_context.additional_data.get::<&str>().unwrap();
        assert_eq!(*data, "additional-data");
        Ok(())
      }
    }

    struct Normal2;

    impl Identifiable for Normal2 {
      fn identifier(&self) -> Identifier {
        "/rspack/normal-loader2".into()
      }
    }

    #[async_trait::async_trait]
    impl Loader<()> for Normal2 {
      async fn run(&self, loader_context: &mut LoaderContext<()>) -> Result<()> {
        loader_context.additional_data.insert("additional-data");
        Ok(())
      }
    }

    let rs = Arc::new(ResourceData {
      scheme: OnceCell::new(),
      resource: "/rspack/main.js?abc=123#efg".to_owned(),
      resource_description: None,
      resource_fragment: None,
      resource_query: None,
      resource_path: Default::default(),
      mimetype: None,
      parameters: None,
      encoding: None,
      encoded_content: None,
    });

    run_loaders(
      vec![Arc::new(Normal) as Arc<dyn Loader>, Arc::new(Normal2)],
      rs,
      Some(Arc::new(TestContentPlugin)),
      (),
      Default::default(),
    )
    .await
    .unwrap();
  }
}
