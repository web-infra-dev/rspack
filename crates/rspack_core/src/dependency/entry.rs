use crate::{
  Dependency, DependencyCategory, DependencyId, DependencyType, ErrorSpan, ModuleDependency,
};

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct EntryDependency {
  id: DependencyId,
  request: String,
}

impl EntryDependency {
  pub fn new(request: String) -> Self {
    Self {
      request,
      id: DependencyId::new(),
    }
  }
}

impl Dependency for EntryDependency {
  fn category(&self) -> &DependencyCategory {
    &DependencyCategory::Esm
  }

  fn dependency_type(&self) -> &DependencyType {
    &DependencyType::Entry
  }
}

impl ModuleDependency for EntryDependency {
  fn id(&self) -> &DependencyId {
    &self.id
  }

  fn request(&self) -> &str {
    &self.request
  }

  fn user_request(&self) -> &str {
    &self.request
  }

  fn span(&self) -> Option<&ErrorSpan> {
    None
  }

  fn set_request(&mut self, request: String) {
    self.request = request;
  }
}
