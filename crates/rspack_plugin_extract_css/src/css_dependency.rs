use rspack_cacheable::{cacheable, cacheable_dyn};
use rspack_collections::IdentifierSet;
use rspack_core::{
  AffectType, AsContextDependency, AsDependencyTemplate, ConnectionState, Dependency,
  DependencyCategory, DependencyId, DependencyRange, DependencyType, FactorizeInfo,
  ModuleDependency, ModuleGraph,
};
use rspack_paths::ArcPath;
use rustc_hash::FxHashSet;

#[cacheable]
#[derive(Debug, Clone)]
pub struct CssDependency {
  pub(crate) id: DependencyId,
  pub(crate) identifier: String,
  pub(crate) content: String,
  pub(crate) context: String,
  pub(crate) media: Option<String>,
  pub(crate) supports: Option<String>,
  pub(crate) source_map: Option<String>,
  pub(crate) layer: Option<String>,

  // One module can be split apart by using `@import` in the middle of one module
  pub(crate) identifier_index: u32,

  // determine module's postOrderIndex
  range: DependencyRange,
  resource_identifier: String,
  pub(crate) cacheable: bool,
  pub(crate) file_dependencies: FxHashSet<ArcPath>,
  pub(crate) context_dependencies: FxHashSet<ArcPath>,
  pub(crate) missing_dependencies: FxHashSet<ArcPath>,
  pub(crate) build_dependencies: FxHashSet<ArcPath>,
  factorize_info: FactorizeInfo,
}

impl CssDependency {
  #[allow(clippy::too_many_arguments)]
  pub(crate) fn new(
    identifier: String,
    layer: Option<String>,
    content: String,
    context: String,
    media: Option<String>,
    supports: Option<String>,
    source_map: Option<String>,
    identifier_index: u32,
    range: DependencyRange,
    cacheable: bool,
    file_dependencies: FxHashSet<ArcPath>,
    context_dependencies: FxHashSet<ArcPath>,
    missing_dependencies: FxHashSet<ArcPath>,
    build_dependencies: FxHashSet<ArcPath>,
  ) -> Self {
    let resource_identifier = format!("css-module-{}-{}", &identifier, identifier_index);
    Self {
      id: DependencyId::new(),
      identifier,
      content,
      layer,
      context,
      media,
      supports,
      source_map,
      identifier_index,
      range,
      resource_identifier,
      cacheable,
      file_dependencies,
      context_dependencies,
      missing_dependencies,
      build_dependencies,
      factorize_info: Default::default(),
    }
  }
}

impl AsDependencyTemplate for CssDependency {}
impl AsContextDependency for CssDependency {}

#[cacheable_dyn]
impl Dependency for CssDependency {
  fn resource_identifier(&self) -> Option<&str> {
    Some(&self.resource_identifier)
  }

  fn id(&self) -> &DependencyId {
    &self.id
  }

  fn dependency_type(&self) -> &rspack_core::DependencyType {
    &DependencyType::ExtractCSS
  }

  fn category(&self) -> &DependencyCategory {
    &DependencyCategory::Unknown
  }

  fn get_module_evaluation_side_effects_state(
    &self,
    _module_graph: &ModuleGraph,
    _module_chain: &mut IdentifierSet,
  ) -> ConnectionState {
    ConnectionState::TransitiveOnly
  }

  // compare to Webpack, which has SortableSet to store
  // the connections in order, if dependency has no span,
  // it can keep the right order, but Rspack uses HashSet,
  // when determining the postOrderIndex, Rspack uses
  // dependency span to set correct order
  fn range(&self) -> Option<&DependencyRange> {
    Some(&self.range)
  }

  fn get_layer(&self) -> Option<&rspack_core::ModuleLayer> {
    self.layer.as_ref()
  }

  fn could_affect_referencing_module(&self) -> AffectType {
    AffectType::Transitive
  }
}

#[cacheable_dyn]
impl ModuleDependency for CssDependency {
  fn request(&self) -> &str {
    &self.identifier
  }

  fn factorize_info(&self) -> &FactorizeInfo {
    &self.factorize_info
  }

  fn factorize_info_mut(&mut self) -> &mut FactorizeInfo {
    &mut self.factorize_info
  }
}
