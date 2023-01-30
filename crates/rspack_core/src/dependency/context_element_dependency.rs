use rspack_error::Result;

use crate::{
  CodeGeneratable, CodeGeneratableResult, Dependency, DependencyCategory, DependencyType,
  ModuleDependency,
};

#[derive(Debug, Eq, PartialEq, Clone, Hash)]
pub struct ContextElementDependency {
  pub request: String,
  pub user_request: String,
  pub category: DependencyCategory,
  pub context: String,
}

impl Dependency for ContextElementDependency {
  fn parent_module_identifier(&self) -> Option<&crate::ModuleIdentifier> {
    None
  }

  fn category(&self) -> &DependencyCategory {
    &self.category
  }

  fn dependency_type(&self) -> &DependencyType {
    &DependencyType::ContextElement
  }

  fn get_context(&self) -> Option<&str> {
    Some(&self.context)
  }
}

impl ModuleDependency for ContextElementDependency {
  fn request(&self) -> &str {
    &self.request
  }

  fn user_request(&self) -> &str {
    &self.user_request
  }

  fn span(&self) -> Option<&crate::ErrorSpan> {
    None
  }
}

impl CodeGeneratable for ContextElementDependency {
  fn generate(
    &self,
    _context: &mut crate::CodeGeneratableContext,
  ) -> Result<CodeGeneratableResult> {
    Ok(CodeGeneratableResult::default())
  }
}
