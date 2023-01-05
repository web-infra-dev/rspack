use rspack_error::Result;

use crate::{
  CodeGeneratable, CodeGeneratableContext, CodeGeneratableResult, Dependency, DependencyCategory,
  DependencyType, ErrorSpan, ModuleDependency, ModuleIdentifier,
};

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct EntryDependency {
  request: String,
}

impl EntryDependency {
  pub fn new(request: String) -> Self {
    Self { request }
  }
}

impl Dependency for EntryDependency {
  fn parent_module_identifier(&self) -> Option<&ModuleIdentifier> {
    None
  }

  fn category(&self) -> &DependencyCategory {
    &DependencyCategory::Esm
  }

  fn dependency_type(&self) -> &DependencyType {
    &DependencyType::Entry
  }
}

impl ModuleDependency for EntryDependency {
  fn request(&self) -> &str {
    &self.request
  }

  fn user_request(&self) -> &str {
    &self.request
  }

  fn span(&self) -> Option<&ErrorSpan> {
    None
  }
}

impl CodeGeneratable for EntryDependency {
  fn generate(&self, _code_generatable_context: &CodeGeneratableContext) -> CodeGeneratableResult {
    CodeGeneratableResult { visitors: vec![] }
  }
}
