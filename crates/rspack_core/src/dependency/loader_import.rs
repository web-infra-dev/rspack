use crate::{
  AsContextDependency, AsDependencyTemplate, Context, Dependency, DependencyCategory, DependencyId,
  DependencyType, ModuleDependency,
};

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct LoaderImportDependency {
  id: DependencyId,
  context: Context,
  request: String,
}

impl LoaderImportDependency {
  pub fn new(request: String, context: Context) -> Self {
    Self {
      request,
      context,
      id: DependencyId::new(),
    }
  }
}

impl AsDependencyTemplate for LoaderImportDependency {}
impl AsContextDependency for LoaderImportDependency {}

impl Dependency for LoaderImportDependency {
  fn id(&self) -> &DependencyId {
    &self.id
  }

  fn category(&self) -> &DependencyCategory {
    &DependencyCategory::LoaderImport
  }

  fn dependency_type(&self) -> &DependencyType {
    &DependencyType::LoaderImport
  }
}

impl ModuleDependency for LoaderImportDependency {
  fn request(&self) -> &str {
    &self.request
  }

  fn user_request(&self) -> &str {
    &self.request
  }

  fn set_request(&mut self, request: String) {
    self.request = request;
  }
}
