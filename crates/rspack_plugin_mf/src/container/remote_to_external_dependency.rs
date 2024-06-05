use rspack_core::{
  AsContextDependency, AsDependencyTemplate, Dependency, DependencyCategory, DependencyId,
  DependencyType, ModuleDependency,
};

#[derive(Debug, Clone)]
pub struct RemoteToExternalDependency {
  id: DependencyId,
  request: String,
}

impl RemoteToExternalDependency {
  pub fn new(request: String) -> Self {
    Self {
      id: DependencyId::new(),
      request,
    }
  }
}

impl Dependency for RemoteToExternalDependency {
  fn id(&self) -> &DependencyId {
    &self.id
  }

  fn dependency_type(&self) -> &DependencyType {
    &DependencyType::RemoteToExternal
  }

  fn category(&self) -> &DependencyCategory {
    &DependencyCategory::Esm
  }
}

impl ModuleDependency for RemoteToExternalDependency {
  fn request(&self) -> &str {
    &self.request
  }
}

impl AsContextDependency for RemoteToExternalDependency {}
impl AsDependencyTemplate for RemoteToExternalDependency {}
