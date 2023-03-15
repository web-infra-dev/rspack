use crate::{
  CodeGeneratable, CodeGeneratableContext, CodeGeneratableResult, Dependency, DependencyCategory,
  DependencyId, DependencyType, ErrorSpan, ModuleDependency, ModuleIdentifier,
};

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct EntryDependency {
  id: Option<DependencyId>,
  request: String,
}

impl EntryDependency {
  pub fn new(request: String) -> Self {
    Self { request, id: None }
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

  fn id(&self) -> Option<DependencyId> {
    self.id
  }

  fn set_id(&mut self, id: Option<DependencyId>) {
    self.id = id;
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
  fn generate(
    &self,
    _code_generatable_context: &mut CodeGeneratableContext,
  ) -> rspack_error::Result<CodeGeneratableResult> {
    Ok(CodeGeneratableResult::default())
  }
}
