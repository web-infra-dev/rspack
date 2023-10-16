use crate::{
  AsDependencyTemplate, Context, Dependency, DependencyCategory, DependencyId, DependencyType,
  ErrorSpan, ModuleDependency,
};

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct EntryDependency {
  id: DependencyId,
  request: String,
  context: Context,
}

impl EntryDependency {
  pub fn new(request: String, context: Context) -> Self {
    Self {
      request,
      context,
      id: DependencyId::new(),
    }
  }
}

impl Dependency for EntryDependency {
  fn id(&self) -> &DependencyId {
    &self.id
  }

  fn category(&self) -> &DependencyCategory {
    &DependencyCategory::Esm
  }

  fn dependency_type(&self) -> &DependencyType {
    &DependencyType::Entry
  }

  fn get_context(&self) -> Option<&Context> {
    Some(&self.context)
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

  fn set_request(&mut self, request: String) {
    self.request = request;
  }

  fn dependency_debug_name(&self) -> &'static str {
    "EntryDependency"
  }
}

impl AsDependencyTemplate for EntryDependency {}
