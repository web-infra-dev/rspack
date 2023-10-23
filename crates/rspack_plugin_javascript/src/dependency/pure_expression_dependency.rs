use rspack_core::{
  AsModuleDependency, Dependency, DependencyId, DependencyTemplate, TemplateContext,
  TemplateReplaceSource, UsedByExports,
};
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
