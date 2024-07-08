use std::path::PathBuf;

use rspack_core::{
  AsContextDependency, AsDependencyTemplate, ConnectionState, Dependency, DependencyCategory,
  DependencyId, ModuleDependency, ModuleGraph, ModuleIdentifier,
};
use rustc_hash::FxHashSet;

use crate::css_module::DEPENDENCY_TYPE;

#[derive(Debug, Clone)]
pub struct CssDependency {
  pub(crate) id: DependencyId,
  pub(crate) identifier: String,
  pub(crate) content: String,
  pub(crate) context: String,
  pub(crate) media: String,
  pub(crate) supports: String,
  pub(crate) source_map: String,

  // One module can be split apart by using `@import` in the middle of one module
  pub(crate) identifier_index: u32,

  // determine module's postOrderIndex
  pub(crate) order_index: u32,

  resource_identifier: String,
  pub(crate) cacheable: bool,
  pub(crate) file_dependencies: FxHashSet<PathBuf>,
  pub(crate) context_dependencies: FxHashSet<PathBuf>,
  pub(crate) missing_dependencies: FxHashSet<PathBuf>,
  pub(crate) build_dependencies: FxHashSet<PathBuf>,
}

impl CssDependency {
  #[allow(clippy::too_many_arguments)]
  pub(crate) fn new(
    identifier: String,
    content: String,
    context: String,
    media: String,
    supports: String,
    source_map: String,
    identifier_index: u32,
    order_index: u32,
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
      context,
      media,
      supports,
      source_map,
      identifier_index,
      order_index,
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

impl Dependency for CssDependency {
  fn resource_identifier(&self) -> Option<&str> {
    Some(&self.resource_identifier)
  }

  fn id(&self) -> &DependencyId {
    &self.id
  }

  fn dependency_type(&self) -> &rspack_core::DependencyType {
    &DEPENDENCY_TYPE
  }

  fn category(&self) -> &DependencyCategory {
    &DependencyCategory::Unknown
  }

  fn get_module_evaluation_side_effects_state(
    &self,
    _module_graph: &ModuleGraph,
    _module_chain: &mut FxHashSet<ModuleIdentifier>,
  ) -> ConnectionState {
    ConnectionState::TransitiveOnly
  }

  // compare to Webpack, which has SortableSet to store
  // the connections in order, if dependency has no span,
  // it can keep the right order, but Rspack uses HashSet,
  // when determining the postOrderIndex, Rspack uses
  // dependency span to set correct order
  fn span(&self) -> Option<rspack_core::ErrorSpan> {
    Some(rspack_core::ErrorSpan {
      start: self.order_index,
      end: self.order_index + 1,
    })
  }
}

impl ModuleDependency for CssDependency {
  fn request(&self) -> &str {
    &self.identifier
  }
}
