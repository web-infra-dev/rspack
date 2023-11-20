use crate::{
  AsContextDependency, AsDependencyTemplate, Dependency, DependencyId, ModuleDependency,
};

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct LoaderImportDependency {
  request: String,
  id: DependencyId,
}

impl LoaderImportDependency {
  pub fn new(request: String) -> Self {
    Self {
      request,
      id: DependencyId::new(),
    }
  }
}

impl AsDependencyTemplate for LoaderImportDependency {}
impl AsContextDependency for LoaderImportDependency {}

impl Dependency for LoaderImportDependency {
  fn dependency_debug_name(&self) -> &'static str {
    "LoaderImportDependency"
  }

  fn id(&self) -> &crate::DependencyId {
    &self.id
  }

  fn dependency_type(&self) -> &crate::DependencyType {
    &crate::DependencyType::LoaderImport
  }

  fn category(&self) -> &crate::DependencyCategory {
    &crate::DependencyCategory::LoaderImport
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
    self.request = request
  }
}
