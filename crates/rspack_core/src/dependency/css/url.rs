use rspack_error::Result;

use crate::{
  CodeGeneratable, CodeGeneratableContext, CodeGeneratableResult, Dependency, DependencyCategory,
  DependencyType, ModuleDependency, ModuleIdentifier,
};

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct CssUrlDependency {
  parent_module_identifier: Option<ModuleIdentifier>,
  request: String,
}

impl CssUrlDependency {
  pub fn new(request: String) -> Self {
    Self {
      parent_module_identifier: None,
      request,
    }
  }
}

impl Dependency for CssUrlDependency {
  fn parent_module_identifier(&self) -> Option<&ModuleIdentifier> {
    self.parent_module_identifier.as_ref()
  }

  fn set_parent_module_identifier(&mut self, identifier: Option<ModuleIdentifier>) {
    self.parent_module_identifier = identifier;
  }

  fn category(&self) -> &DependencyCategory {
    &DependencyCategory::Url
  }

  fn dependency_type(&self) -> &DependencyType {
    &DependencyType::CssUrl
  }
}

impl ModuleDependency for CssUrlDependency {
  fn request(&self) -> &str {
    &self.request
  }

  fn user_request(&self) -> &str {
    &self.request
  }
}

impl CodeGeneratable for CssUrlDependency {
  fn generate(
    &self,
    _code_generatable_context: CodeGeneratableContext,
  ) -> Result<CodeGeneratableResult> {
    todo!()
  }
}
