use rspack_error::Result;

use crate::{
  dependency::{
    CodeGeneratable, CodeGeneratableContext, CodeGeneratableResult, Dependency, DependencyCategory,
    ModuleDependency,
  },
  ModuleIdentifier,
};

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
struct EsmImportDependency {
  parent_module_identifier: Option<ModuleIdentifier>,
  request: String,
  user_request: String,
}

impl EsmImportDependency {
  pub fn new(
    parent_module_identifier: Option<ModuleIdentifier>,
    request: String,
    user_request: String,
  ) -> Self {
    Self {
      parent_module_identifier,
      request,
      user_request,
    }
  }
}

impl Dependency for EsmImportDependency {
  fn parent_module_identifier(&self) -> Option<&ModuleIdentifier> {
    self.parent_module_identifier.as_ref()
  }

  fn category(&self) -> &DependencyCategory {
    &DependencyCategory::Esm
  }
}

// impl_module_dependency_cast!(EsmImportDependency);

impl ModuleDependency for EsmImportDependency {
  fn request(&self) -> &str {
    &self.request
  }

  fn user_request(&self) -> &str {
    &self.user_request
  }
}

impl CodeGeneratable for EsmImportDependency {
  fn generate(
    &self,
    code_generatable_context: CodeGeneratableContext,
  ) -> Result<CodeGeneratableResult> {
    todo!()
  }
}
