use crate::{
  AsDependency, DependencyTemplate, RuntimeGlobals, TemplateContext, TemplateReplaceSource,
};

#[derive(Debug, Eq, PartialEq, Clone, Hash)]
pub struct RuntimeRequirementsDependency {
  pub runtime_requirements: RuntimeGlobals,
}

impl DependencyTemplate for RuntimeRequirementsDependency {
  fn apply(
    &self,
    _source: &mut TemplateReplaceSource,
    code_generatable_context: &mut TemplateContext,
  ) {
    code_generatable_context
      .runtime_requirements
      .insert(self.runtime_requirements);
  }

  fn dependency_id(&self) -> Option<crate::DependencyId> {
    None
  }
}
impl AsDependency for RuntimeRequirementsDependency {}

impl RuntimeRequirementsDependency {
  pub fn new(runtime_requirements: RuntimeGlobals) -> Self {
    Self {
      runtime_requirements,
    }
  }
}
