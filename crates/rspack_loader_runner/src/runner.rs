use std::{
  fmt::Debug,
  path::{Path, PathBuf},
  sync::Arc,
};

use anymap::CloneAny;
use derivative::Derivative;
use once_cell::sync::OnceCell;
use rspack_error::{error, Diagnostic, IntoTWithDiagnosticArray, Result, TWithDiagnosticArray};
use rspack_sources::SourceMap;
use rustc_hash::FxHashSet as HashSet;

use crate::{
  content::Content,
  get_scheme,
  loader::{Loader, LoaderItem, LoaderItemList},
  plugin::LoaderRunnerPlugin,
  Scheme,
};

#[derive(Debug, Clone)]
pub struct ResourceData {
  /// Resource with absolute path, query and fragment
  pub resource: String,
  /// Absolute resource path only
  pub resource_path: PathBuf,
  /// Resource query with `?` prefix
  pub resource_query: Option<String>,
  /// Resource fragment with `#` prefix
  pub resource_fragment: Option<String>,
  pub resource_description: Option<DescriptionData>,
  pub mimetype: Option<String>,
  pub parameters: Option<String>,
  pub encoding: Option<String>,
  pub encoded_content: Option<String>,
  scheme: OnceCell<Scheme>,
}

impl ResourceData {
  pub fn new(resource: String, path: PathBuf) -> Self {
    Self {
      resource,
      resource_path: path,
      resource_query: None,
      resource_fragment: None,
      resource_description: None,
      mimetype: None,
      parameters: None,
      encoding: None,
      encoded_content: None,
      scheme: OnceCell::new(),
    }
  }

  pub fn get_scheme(&self) -> &Scheme {
    self.scheme.get_or_init(|| get_scheme(&self.resource))
  }

  pub fn set_resource(&mut self, v: String) {
    self.resource = v;
  }

  pub fn set_path(&mut self, v: PathBuf) {
    self.resource_path = v;
  }

  pub fn query(mut self, v: String) -> Self {
    self.resource_query = Some(v);
    self
  }

  pub fn set_query(&mut self, v: String) {
    self.resource_query = Some(v);
  }

  pub fn query_optional(mut self, v: Option<String>) -> Self {
    self.resource_query = v;
    self
  }

  pub fn set_query_optional(&mut self, v: Option<String>) {
    self.resource_query = v;
  }

  pub fn fragment(mut self, v: String) -> Self {
    self.resource_fragment = Some(v);
    self
  }

  pub fn set_fragment(&mut self, v: String) {
    self.resource_fragment = Some(v);
  }

  pub fn fragment_optional(mut self, v: Option<String>) -> Self {
    self.resource_fragment = v;
    self
  }

  pub fn set_fragment_optional(&mut self, v: Option<String>) {
    self.resource_fragment = v;
  }

  pub fn description(mut self, v: DescriptionData) -> Self {
    self.resource_description = Some(v);
    self
  }

  pub fn description_optional(mut self, v: Option<DescriptionData>) -> Self {
    self.resource_description = v;
    self
  }

  pub fn mimetype(mut self, v: String) -> Self {
    self.mimetype = Some(v);
    self
  }

  pub fn set_mimetype(&mut self, v: String) {
    self.mimetype = Some(v);
  }

  pub fn parameters(mut self, v: String) -> Self {
    self.parameters = Some(v);
    self
  }

  pub fn set_parameters(&mut self, v: String) {
    self.parameters = Some(v);
  }

  pub fn encoding(mut self, v: String) -> Self {
    self.encoding = Some(v);
    self
  }

  pub fn set_encoding(&mut self, v: String) {
    self.encoding = Some(v);
  }

  pub fn encoded_content(mut self, v: String) -> Self {
    self.encoded_content = Some(v);
    self
  }

  pub fn set_encoded_content(&mut self, v: String) {
    self.encoded_content = Some(v);
  }
}

/// Used for [Rule.descriptionData](https://www.rspack.dev/config/module.html#ruledescriptiondata) and
/// package.json.sideEffects in tree shaking.
#[derive(Debug, Clone)]
pub struct DescriptionData {
  /// Path to package.json
  path: PathBuf,

  /// Raw package.json
  json: Arc<serde_json::Value>,
}

impl DescriptionData {
  pub fn new(path: PathBuf, json: Arc<serde_json::Value>) -> Self {
    Self { path, json }
  }

  pub fn path(&self) -> &Path {
    &self.path
  }

  pub fn json(&self) -> &serde_json::Value {
    self.json.as_ref()
  }
}

pub type AdditionalData = anymap::Map<dyn CloneAny + Send + Sync>;

#[derive(Derivative)]
#[derivative(Debug)]
pub struct LoaderContext<'c, C> {
  pub hot: bool,

  /// Content of loader, represented by string or buffer
  /// Content should always be exist if at normal stage,
  /// It will be `None` at pitching stage.
  pub content: Option<Content>,

  /// The resource part of the request, including query and fragment.
  /// E.g. /abc/resource.js?query=1#some-fragment
  pub resource: &'c str,
  /// The resource part of the request.
  /// E.g. /abc/resource.js
  pub resource_path: &'c Path,
  /// The query of the request
  /// E.g. query=1
  pub resource_query: Option<&'c str>,
  /// The fragment of the request
  /// E.g. some-fragment
  pub resource_fragment: Option<&'c str>,

  pub context: C,
  pub source_map: Option<SourceMap>,
  pub additional_data: AdditionalData,
  pub cacheable: bool,

  pub file_dependencies: HashSet<PathBuf>,
  pub context_dependencies: HashSet<PathBuf>,
  pub missing_dependencies: HashSet<PathBuf>,
  pub build_dependencies: HashSet<PathBuf>,

  pub asset_filenames: HashSet<String>,

  // Only used for cross-crate accessing.
  // This field should not be accessed in builtin loaders.
  pub __loader_index: usize,
  // Only used for cross-crate accessing.
  // This field should not be accessed in builtin loaders.
  pub __loader_items: LoaderItemList<'c, C>,
  // Only used for cross-crate accessing.
  // This field should not be accessed in builtin loaders.
  #[derivative(Debug = "ignore")]
  pub __plugins: &'c [&'c dyn LoaderRunnerPlugin<Context = C>],
  // Only used for cross-crate accessing.
  // This field should not be accessed in builtin loaders.
  pub __resource_data: &'c ResourceData,
  // Only used for cross-crate accessing.
  // This field should not be accessed in builtin loaders.
  pub __diagnostics: Vec<Diagnostic>,
}

impl<'c, C> LoaderContext<'c, C> {
  pub fn remaining_request(&self) -> LoaderItemList<'_, C> {
    if self.__loader_index >= self.__loader_items.len() - 1 {
      return Default::default();
    }
    LoaderItemList(&self.__loader_items[self.__loader_index + 1..])
  }

  pub fn current_request(&self) -> LoaderItemList<'_, C> {
    LoaderItemList(&self.__loader_items[self.__loader_index..])
  }

  pub fn previous_request(&self) -> LoaderItemList<'_, C> {
    LoaderItemList(&self.__loader_items[..self.__loader_index])
  }

  pub fn request(&self) -> LoaderItemList<'_, C> {
    LoaderItemList(&self.__loader_items[..])
  }

  pub fn current_loader(&self) -> &LoaderItem<C> {
    &self.__loader_items[self.__loader_index]
  }

  pub fn loader_index(&self) -> usize {
    self.__loader_index
  }

  /// Emit a diagnostic, it can be a `warning` or `error`.
  pub fn emit_diagnostic(&mut self, diagnostic: Diagnostic) {
    self.__diagnostics.push(diagnostic)
  }
}

async fn process_resource<C: Send>(loader_context: &mut LoaderContext<'_, C>) -> Result<()> {
  for plugin in loader_context.__plugins {
    if let Some(processed_resource) = plugin
      .process_resource(loader_context.__resource_data)
      .await?
    {
      loader_context.content = Some(processed_resource);
    }
  }

  if loader_context.content.is_none()
    && !loader_context
      .__resource_data
      .resource_path
      .to_string_lossy()
      .is_empty()
  {
    let result = tokio::fs::read(&loader_context.__resource_data.resource_path)
      .await
      .map_err(|e| {
        let r = loader_context
          .__resource_data
          .resource_path
          .to_string_lossy()
          .to_string();
        error!("{e}, failed to read {r}")
      })?;
    loader_context.content = Some(Content::from(result));
  }

  // Bail out if loader does not exist,
  // or the last loader has been executed.
  if loader_context.__loader_index == 0
    && (loader_context.__loader_items.first().is_none()
      || loader_context
        .__loader_items
        .first()
        .map(|loader| loader.normal_executed())
        .unwrap_or_default())
  {
    return Ok(());
  }

  loader_context.__loader_index = loader_context.__loader_items.len() - 1;
  iterate_normal_loaders(loader_context).await
}

async fn create_loader_context<'c, C: 'c>(
  __loader_items: &'c [LoaderItem<C>],
  resource_data: &'c ResourceData,
  plugins: &'c [&dyn LoaderRunnerPlugin<Context = C>],
  context: C,
  additional_data: AdditionalData,
) -> Result<LoaderContext<'c, C>> {
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
    resource: &resource_data.resource,
    resource_path: &resource_data.resource_path,
    resource_query: resource_data.resource_query.as_deref(),
    resource_fragment: resource_data.resource_fragment.as_deref(),
    context,
    source_map: None,
    additional_data,
    __loader_index: 0,
    __loader_items: LoaderItemList(__loader_items),
    __plugins: plugins,
    __resource_data: resource_data,
    __diagnostics: vec![],
  };

  for plugin in plugins {
    plugin.loader_context(&mut loader_context)?;
  }

  Ok(loader_context)
}

#[async_recursion::async_recursion]
async fn iterate_normal_loaders<C: Send>(loader_context: &mut LoaderContext<'_, C>) -> Result<()> {
  let current_loader_item = loader_context.current_loader();

  if current_loader_item.normal_executed() {
    if loader_context.__loader_index == 0 {
      return Ok(());
    }
    loader_context.__loader_index -= 1;
    return iterate_normal_loaders(loader_context).await;
  }

  let loader = current_loader_item.loader.clone();
  current_loader_item.set_normal_executed();
  for p in loader_context.__plugins {
    p.before_each(loader_context)?;
  }
  loader.run(loader_context).await?;

  iterate_normal_loaders(loader_context).await
}

#[async_recursion::async_recursion]
async fn iterate_pitching_loaders<C: Send>(
  loader_context: &mut LoaderContext<'_, C>,
) -> Result<()> {
  if loader_context.__loader_index >= loader_context.__loader_items.len() {
    return process_resource(loader_context).await;
  }

  let current_loader_item = loader_context.current_loader();

  if current_loader_item.pitch_executed() {
    loader_context.__loader_index += 1;
    return iterate_pitching_loaders(loader_context).await;
  }

  let loader = current_loader_item.loader.clone();
  current_loader_item.set_pitch_executed();
  for p in loader_context.__plugins {
    p.before_each(loader_context)?;
  }
  loader.pitch(loader_context).await?;

  let current_loader_item = loader_context.current_loader();

  // If pitching loader modifies the content,
  // runner should skip the remaining pitching loaders
  // and redirect pipeline to the normal stage.
  // Or, if a pitching loader finishes the normal stage, then we should execute backwards.
  // Yes, the second one is a backdoor for JS loaders.
  if loader_context.content.is_some() || current_loader_item.normal_executed() {
    if loader_context.__loader_index == 0 {
      return Ok(());
    }
    loader_context.__loader_index -= 1;
    iterate_normal_loaders(loader_context).await?;
  } else {
    iterate_pitching_loaders(loader_context).await?;
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
  pub asset_filenames: HashSet<String>,
  pub content: Content,
  pub source_map: Option<SourceMap>,
  pub additional_data: AdditionalData,
}

impl<C> TryFrom<LoaderContext<'_, C>> for TWithDiagnosticArray<LoaderResult> {
  type Error = rspack_error::Error;
  fn try_from(loader_context: LoaderContext<'_, C>) -> std::result::Result<Self, Self::Error> {
    let content = loader_context.content.ok_or_else(|| {
      if !loader_context.__loader_items.is_empty() {
        let loader = loader_context.__loader_items[0].to_string();
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
      .with_diagnostic(loader_context.__diagnostics),
    )
  }
}

pub async fn run_loaders<C: Send>(
  loaders: &[Arc<dyn Loader<C>>],
  resource_data: &ResourceData,
  plugins: &[&dyn LoaderRunnerPlugin<Context = C>],
  context: C,
  additional_data: AdditionalData,
) -> Result<TWithDiagnosticArray<LoaderResult>> {
  let loaders = loaders
    .iter()
    .map(|i| i.clone().into())
    .collect::<Vec<LoaderItem<C>>>();

  let mut loader_context = create_loader_context(
    &loaders[..],
    resource_data,
    plugins,
    context,
    additional_data,
  )
  .await?;

  assert!(loader_context.content.is_none());
  iterate_pitching_loaders(&mut loader_context).await?;

  loader_context.try_into()
}

#[cfg(test)]
#[allow(unused)]
mod test {
  use std::{cell::RefCell, sync::Arc};

  use once_cell::sync::OnceCell;
  use rspack_error::Result;
  use rspack_identifier::{Identifiable, Identifier};

  use super::{run_loaders, Loader, LoaderContext, ResourceData};
  use crate::{
    content::Content,
    loader::test::{Composed, Custom, Custom2},
    plugin::LoaderRunnerPlugin,
    runner::Scheme,
    DisplayWithSuffix,
  };
  fn noop(_: &mut LoaderContext<()>) -> Result<()> {
    Ok(())
  }

  struct TestContentPlugin;

  #[async_trait::async_trait]
  impl LoaderRunnerPlugin for TestContentPlugin {
    type Context = ();

    fn name(&self) -> &'static str {
      "test-content"
    }

    fn loader_context(&self, _context: &mut LoaderContext<Self::Context>) -> Result<()> {
      Ok(())
    }

    fn before_each(&self, _context: &mut LoaderContext<Self::Context>) -> Result<()> {
      Ok(())
    }

    async fn process_resource(&self, _resource_data: &ResourceData) -> Result<Option<Content>> {
      Ok(Some(Content::Buffer(vec![])))
    }
  }

  #[tokio::test]
  async fn should_have_the_correct_requests() {
    thread_local! {
      static IDENTS: RefCell<Vec<String>> = RefCell::default();
    }

    struct Pitching;

    impl Identifiable for Pitching {
      fn identifier(&self) -> Identifier {
        "/rspack/pitching-loader".into()
      }
    }

    #[async_trait::async_trait]
    impl Loader<()> for Pitching {
      async fn pitch(&self, loader_context: &mut LoaderContext<'_, ()>) -> Result<()> {
        IDENTS.with(|i| {
          i.borrow_mut()
            .push(loader_context.remaining_request().to_string());
          i.borrow_mut()
            .push(loader_context.previous_request().to_string());
          i.borrow_mut()
            .push(loader_context.current_request().to_string());
          i.borrow_mut().push(
            loader_context
              .current_request()
              .display_with_suffix(loader_context.resource),
          );
        });
        Ok(())
      }
    }
    let c1 = Arc::new(Custom) as Arc<dyn Loader<()>>;
    let i1 = c1.identifier();
    let c2 = Arc::new(Custom2) as Arc<dyn Loader<()>>;
    let i2 = c2.identifier();
    let c3 = Arc::new(Composed) as Arc<dyn Loader<()>>;
    let i3 = c3.identifier();
    let p1 = Arc::new(Pitching) as Arc<dyn Loader<()>>;
    let i0 = p1.identifier();

    let rs = ResourceData {
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
    };

    run_loaders(
      &[c1, p1, c2, c3],
      &rs,
      &[&TestContentPlugin],
      (),
      Default::default(),
    )
    .await
    .unwrap();

    IDENTS.with(|i| {
      let i = i.borrow();
      // remaining request
      let expected = "".to_owned() + &**i2 + "!" + &**i3;
      assert_eq!(i[0], expected);
      // previous request
      let expected = "".to_owned() + &**i1;
      assert_eq!(i[1], expected);
      // current request
      let expected = "".to_owned() + &**i0 + "!" + &**i2 + "!" + &**i3;
      assert_eq!(i[2], expected);
      let expected = expected + "!" + &*rs.resource;
      assert_eq!(i[3], expected);
    });
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
      async fn pitch(&self, loader_context: &mut LoaderContext<'_, ()>) -> Result<()> {
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
      async fn pitch(&self, loader_context: &mut LoaderContext<'_, ()>) -> Result<()> {
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
      async fn run(&self, loader_context: &mut LoaderContext<'_, ()>) -> Result<()> {
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
      async fn run(&self, loader_context: &mut LoaderContext<'_, ()>) -> Result<()> {
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
      async fn run(&self, loader_context: &mut LoaderContext<'_, ()>) -> Result<()> {
        IDENTS.with(|i| i.borrow_mut().push("pitch-normal-base-normal".to_string()));
        Ok(())
      }

      async fn pitch(&self, loader_context: &mut LoaderContext<'_, ()>) -> Result<()> {
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
      async fn run(&self, loader_context: &mut LoaderContext<'_, ()>) -> Result<()> {
        IDENTS.with(|i| i.borrow_mut().push("pitch-normal-normal".to_string()));
        Ok(())
      }

      async fn pitch(&self, loader_context: &mut LoaderContext<'_, ()>) -> Result<()> {
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
      async fn run(&self, loader_context: &mut LoaderContext<'_, ()>) -> Result<()> {
        IDENTS.with(|i| i.borrow_mut().push("pitch-normal-normal-2".to_string()));
        Ok(())
      }

      async fn pitch(&self, loader_context: &mut LoaderContext<'_, ()>) -> Result<()> {
        IDENTS.with(|i| i.borrow_mut().push("pitch-normal-pitch-2".to_string()));
        loader_context.content = Some(Content::Buffer(vec![]));
        Ok(())
      }
    }

    let c1 = Arc::new(Normal) as Arc<dyn Loader<()>>;
    let c2 = Arc::new(Normal2) as Arc<dyn Loader<()>>;
    let p1 = Arc::new(Pitching) as Arc<dyn Loader<()>>;
    let p2 = Arc::new(Pitching2) as Arc<dyn Loader<()>>;

    let rs = ResourceData {
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
    };

    run_loaders(
      &[p1, p2, c1, c2],
      &rs,
      &[&TestContentPlugin],
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
      &[p1, p2, p3],
      &rs,
      &[&TestContentPlugin],
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
      async fn run(&self, loader_context: &mut LoaderContext<'_, ()>) -> Result<()> {
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
      async fn run(&self, loader_context: &mut LoaderContext<'_, ()>) -> Result<()> {
        loader_context.additional_data.insert("additional-data");
        Ok(())
      }
    }

    let rs = ResourceData {
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
    };

    run_loaders(
      &[Arc::new(Normal) as Arc<dyn Loader>, Arc::new(Normal2)],
      &rs,
      &[&TestContentPlugin],
      (),
      Default::default(),
    )
    .await
    .unwrap();
  }
}
