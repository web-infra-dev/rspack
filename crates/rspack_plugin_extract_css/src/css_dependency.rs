use std::path::PathBuf;

use rspack_cacheable::{
  cacheable, cacheable_dyn,
  with::{AsString, AsVec},
};
use rspack_collections::IdentifierSet;
use rspack_core::{
  AffectType, AsContextDependency, AsDependencyTemplate, ConnectionState, Dependency,
  DependencyCategory, DependencyId, ModuleDependency, ModuleGraph, RealDependencyLocation,
};
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
  // @TODO(shulaoda) Does this have any additional side effects?
  // pub(crate) order_index: u32,
  range: RealDependencyLocation,
  resource_identifier: String,
  pub(crate) cacheable: bool,
  #[cacheable(with=AsVec<AsString>)]
  pub(crate) file_dependencies: FxHashSet<PathBuf>,
  #[cacheable(with=AsVec<AsString>)]
  pub(crate) context_dependencies: FxHashSet<PathBuf>,
  #[cacheable(with=AsVec<AsString>)]
  pub(crate) missing_dependencies: FxHashSet<PathBuf>,
  #[cacheable(with=AsVec<AsString>)]
  pub(crate) build_dependencies: FxHashSet<PathBuf>,
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
    range: RealDependencyLocation,
    cacheable: bool,
    file_dependencies: FxHashSet<PathBuf>,
    context_dependencies: FxHashSet<PathBuf>,
    missing_dependencies: FxHashSet<PathBuf>,
    build_dependencies: FxHashSet<PathBuf>,
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
    &rspack_core::DependencyType::MiniExtractDep
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
  fn range(&self) -> Option<&RealDependencyLocation> {
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
}
