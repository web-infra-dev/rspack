//! HarmonyCompatibilityDependency

use rspack_core::{
  create_javascript_visitor, CodeGeneratable, CodeGeneratableContext, CodeGeneratableResult,
  Dependency, DependencyCategory, DependencyId, DependencyType, ErrorSpan, ModuleDependency,
  ModuleIdentifier, RuntimeGlobals,
};

#[derive(Debug, Eq, PartialEq, Clone, Hash)]
pub struct HarmonyCompatibilityDependency;

impl HarmonyCompatibilityDependency {
  pub fn new() -> Self {
    Self
  }
}

impl Dependency for HarmonyCompatibilityDependency {
  fn parent_module_identifier(&self) -> Option<&ModuleIdentifier> {
    None
  }

  fn dependency_type(&self) -> &DependencyType {
    &DependencyType::HarmonyExportHeader
  }
}

impl CodeGeneratable for HarmonyCompatibilityDependency {
  fn generate(
    &self,
    code_generatable_context: &mut CodeGeneratableContext,
  ) -> rspack_error::Result<CodeGeneratableResult> {
    let mut code_gen = CodeGeneratableResult::default();
    if code_generatable_context
      .compilation
      .module_graph
      .is_async(&code_generatable_context.module.identifier())
    {
      code_generatable_context
        .runtime_requirements
        .insert(RuntimeGlobals::MODULE);
      code_generatable_context
        .runtime_requirements
        .insert(RuntimeGlobals::ASYNC_MODULE);
      code_gen.visitors.push(
        create_javascript_visitor!(visit_mut_program(n: &mut Program) {

        }),
      );
    }

    Ok(code_gen)
  }
}
