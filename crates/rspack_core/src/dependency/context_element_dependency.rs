use rspack_error::Result;

use crate::{
  CodeGeneratable, CodeGeneratableResult, ContextOptions, Dependency, DependencyCategory,
  DependencyId, DependencyType, ModuleDependency,
};

#[derive(Debug, Eq, PartialEq, Clone, Hash)]
pub struct ContextElementDependency {
  pub id: Option<DependencyId>,
  // TODO remove this async dependency mark
  pub options: ContextOptions,
  pub request: String,
  pub user_request: String,
  pub category: DependencyCategory,
  pub context: String,
}

impl Dependency for ContextElementDependency {
  fn id(&self) -> Option<DependencyId> {
    self.id
  }

  fn set_id(&mut self, id: Option<DependencyId>) {
    self.id = id;
  }

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

  fn options(&self) -> Option<&ContextOptions> {
    Some(&self.options)
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
