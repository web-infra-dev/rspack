use crate::{
  dependency::{
    CodeGeneratable, CodeGeneratableContext, CodeGeneratableResult, Dependency, DependencyCategory,
    ModuleDependency,
  },
  DependencyType, ModuleIdentifier,
};

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
struct EsmDynamicImportDependency {
  parent_module_identifier: Option<ModuleIdentifier>,
  request: String,
  user_request: String,
}

impl EsmDynamicImportDependency {
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

impl Dependency for EsmDynamicImportDependency {
  fn parent_module_identifier(&self) -> Option<&ModuleIdentifier> {
    self.parent_module_identifier.as_ref()
  }

  fn category(&self) -> &DependencyCategory {
    &DependencyCategory::Esm
  }

  fn dependency_type(&self) -> &DependencyType {
    &DependencyType::DynamicImport
  }
}

// impl_module_dependency_cast!(EsmDynamicImportDependency);

impl ModuleDependency for EsmDynamicImportDependency {
  fn request(&self) -> &str {
    &self.request
  }

  fn user_request(&self) -> &str {
    &self.user_request
  }
}

impl CodeGeneratable for EsmDynamicImportDependency {
  fn generate(
    &self,
    _code_generatable_context: CodeGeneratableContext,
  ) -> rspack_error::Result<CodeGeneratableResult> {
    todo!()
  }
}
