use rspack_core::{
  AsModuleDependency, Dependency, DependencyId, DependencyTemplate, ModuleIdentifier,
  TemplateContext, TemplateReplaceSource, UsedByExports,
};
#[derive(Debug, Clone)]
pub struct PureExpressionDependency {
  pub start: u32,
  pub end: u32,
  pub used_by_exports: Option<UsedByExports>,
  id: DependencyId,
  pub module_identifier: ModuleIdentifier,
}

impl PureExpressionDependency {
  pub fn new(start: u32, end: u32, module_identifier: ModuleIdentifier) -> Self {
    Self {
      start,
      end,
      used_by_exports: None,
      id: DependencyId::default(),
      module_identifier,
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
  fn apply(&self, source: &mut TemplateReplaceSource, ctx: &mut TemplateContext) {
    if self.used_by_exports != Some(UsedByExports::Bool(false)) {
      let exports_info = ctx
        .compilation
        .module_graph
        .get_exports_info(&self.module_identifier);
      // TODO: runtime optimization
      // return;
    }
    source.insert(
      self.start,
      "(/* unused pure expression or super */ null && (",
      None,
    );
    source.insert(self.end, "))", None);
  }
}
