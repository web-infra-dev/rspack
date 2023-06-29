use crate::{
  CodeGeneratableContext, CodeGeneratableDependency, CodeGeneratableSource, Dependency,
  RuntimeGlobals,
};

#[derive(Debug, Eq, PartialEq, Clone, Hash)]
pub struct RuntimeRequirementsDependency {
  pub runtime_requirements: RuntimeGlobals,
}

impl Dependency for RuntimeRequirementsDependency {}

impl CodeGeneratableDependency for RuntimeRequirementsDependency {
  fn apply(
    &self,
    _source: &mut CodeGeneratableSource,
    code_generatable_context: &mut CodeGeneratableContext,
  ) {
    code_generatable_context
      .runtime_requirements
      .add(self.runtime_requirements);
  }
}

impl RuntimeRequirementsDependency {
  pub fn new(runtime_requirements: RuntimeGlobals) -> Self {
    Self {
      runtime_requirements,
    }
  }
}
