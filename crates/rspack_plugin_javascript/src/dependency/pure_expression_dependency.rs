use rspack_core::{
  AsModuleDependency, ConnectionState, Dependency, DependencyId, DependencyTemplate, ModuleGraph,
  ModuleIdentifier, TemplateContext, TemplateReplaceSource, UsedByExports,
};
use rustc_hash::FxHashSet as HashSet;
#[derive(Debug, Clone)]
pub struct PureExpressionDependency {
  pub start: u32,
  pub end: u32,
  pub used_by_exports: Option<UsedByExports>,
  id: DependencyId,
}

impl PureExpressionDependency {
  pub fn new(start: u32, end: u32) -> Self {
    Self {
      start,
      end,
      used_by_exports: None,
      id: DependencyId::default(),
    }
  }
}

impl Dependency for PureExpressionDependency {
  fn id(&self) -> &rspack_core::DependencyId {
    &self.id
  }

  fn set_used_by_exports(&mut self, used_by_exports: Option<UsedByExports>) {
    self.used_by_exports = used_by_exports;
  }

  fn dependency_debug_name(&self) -> &'static str {
    "PureExpressionDependency"
  }

  fn get_module_evaluation_side_effects_state(
    &self,
    _module_graph: &ModuleGraph,
    _module_chain: &mut HashSet<ModuleIdentifier>,
  ) -> ConnectionState {
    ConnectionState::Bool(false)
  }
}

impl AsModuleDependency for PureExpressionDependency {
  fn as_module_dependency(&self) -> Option<&dyn rspack_core::ModuleDependency> {
    None
  }

  fn as_module_dependency_mut(&mut self) -> Option<&mut dyn rspack_core::ModuleDependency> {
    None
  }
}

impl DependencyTemplate for PureExpressionDependency {
  fn apply(
    &self,
    _source: &mut TemplateReplaceSource,
    _code_generatable_context: &mut TemplateContext,
  ) {
  }
}
