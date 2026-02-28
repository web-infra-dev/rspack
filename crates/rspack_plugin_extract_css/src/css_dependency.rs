use rspack_cacheable::{cacheable, cacheable_dyn};
use rspack_collections::{IdentifierMap, IdentifierSet};
use rspack_core::{
  AffectType, AsContextDependency, AsDependencyCodeGeneration, ConnectionState, Dependency,
  DependencyCategory, DependencyId, DependencyRange, DependencyType, FactorizeInfo,
  ModuleDependency, ModuleGraph, ModuleGraphCacheArtifact, ModuleLayer, ResourceIdentifier,
};
use rspack_paths::ArcPathSet;

#[cacheable]
#[derive(Debug, Clone)]
pub struct CssDependency {
  pub(crate) id: DependencyId,
  pub(crate) identifier: String,
  pub(crate) module_layer: Option<ModuleLayer>,
  pub(crate) content: String,
  pub(crate) context: String,
  pub(crate) media: Option<String>,
  pub(crate) supports: Option<String>,
  pub(crate) source_map: Option<String>,
  pub(crate) css_layer: Option<String>,

  // One module can be split apart by using `@import` in the middle of one module
  pub(crate) identifier_index: u32,

  // determine module's postOrderIndex
  range: DependencyRange,
  resource_identifier: ResourceIdentifier,
  pub(crate) cacheable: bool,
  pub(crate) file_dependencies: ArcPathSet,
  pub(crate) context_dependencies: ArcPathSet,
  pub(crate) missing_dependencies: ArcPathSet,
  pub(crate) build_dependencies: ArcPathSet,
  factorize_info: FactorizeInfo,
}

impl CssDependency {
  #[allow(clippy::too_many_arguments)]
  pub(crate) fn new(
    identifier: String,
    module_layer: Option<ModuleLayer>,
    css_layer: Option<String>,
    content: String,
    context: String,
    media: Option<String>,
    supports: Option<String>,
    source_map: Option<String>,
    identifier_index: u32,
    range: DependencyRange,
    cacheable: bool,
    file_dependencies: ArcPathSet,
    context_dependencies: ArcPathSet,
    missing_dependencies: ArcPathSet,
    build_dependencies: ArcPathSet,
  ) -> Self {
    let resource_identifier = format!("css-module-{}-{}", &identifier, identifier_index).into();
    Self {
      id: DependencyId::new(),
      identifier,
      content,
      module_layer,
      css_layer,
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

impl AsDependencyCodeGeneration for CssDependency {}
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
    _module_graph_cache: &ModuleGraphCacheArtifact,
    _module_chain: &mut IdentifierSet,
    _connection_state_cache: &mut IdentifierMap<ConnectionState>,
  ) -> ConnectionState {
    ConnectionState::TransitiveOnly
  }

  // compare to Webpack, which has SortableSet to store
  // the connections in order, if dependency has no span,
  // it can keep the right order, but Rspack uses HashSet,
  // when determining the postOrderIndex, Rspack uses
  // dependency span to set correct order
  fn range(&self) -> Option<DependencyRange> {
    Some(self.range)
  }

  fn get_layer(&self) -> Option<&rspack_core::ModuleLayer> {
    self.module_layer.as_ref()
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
