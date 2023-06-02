use rspack_error::Result;

use crate::{
  CodeGeneratable, CodeGeneratableContext, CodeGeneratableResult, CodeReplaceSourceDependency,
  CodeReplaceSourceDependencyContext, CodeReplaceSourceDependencyReplaceSource, Dependency,
  RuntimeGlobals,
};

#[derive(Debug, Eq, PartialEq, Clone, Hash)]
pub struct RuntimeRequirementsDependency {
  pub runtime_requirements: RuntimeGlobals,
}

impl Dependency for RuntimeRequirementsDependency {}

impl CodeGeneratable for RuntimeRequirementsDependency {
  fn generate(
    &self,
    code_generatable_context: &mut CodeGeneratableContext,
  ) -> Result<CodeGeneratableResult> {
    code_generatable_context
      .runtime_requirements
      .add(self.runtime_requirements);

    Ok(CodeGeneratableResult::default())
  }
}

impl CodeReplaceSourceDependency for RuntimeRequirementsDependency {
  fn apply(
    &self,
    _source: &mut CodeReplaceSourceDependencyReplaceSource,
    code_generatable_context: &mut CodeReplaceSourceDependencyContext,
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
