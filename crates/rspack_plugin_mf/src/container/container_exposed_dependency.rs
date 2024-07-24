use rspack_core::{
  AsContextDependency, AsDependencyTemplate, Dependency, DependencyCategory, DependencyId,
  DependencyType, ModuleDependency,
};

#[derive(Debug, Clone)]
pub struct ContainerExposedDependency {
  id: DependencyId,
  request: String,
  pub exposed_name: String,
  resource_identifier: String,
}

impl ContainerExposedDependency {
  pub fn new(exposed_name: String, request: String) -> Self {
    let resource_identifier = format!("exposed dependency {}={}", exposed_name, request);
    Self {
      id: DependencyId::new(),
      request,
      exposed_name,
      resource_identifier,
    }
  }
}

impl Dependency for ContainerExposedDependency {
  fn id(&self) -> &DependencyId {
    &self.id
  }

  fn category(&self) -> &DependencyCategory {
    &DependencyCategory::Esm
  }

  fn dependency_type(&self) -> &DependencyType {
    &DependencyType::ContainerExposed
  }

  fn resource_identifier(&self) -> Option<&str> {
    Some(&self.resource_identifier)
  }
}

impl ModuleDependency for ContainerExposedDependency {
  fn request(&self) -> &str {
    &self.request
  }

  fn user_request(&self) -> &str {
    &self.request
  }

  fn set_request(&mut self, _request: String) {}
}

impl AsContextDependency for ContainerExposedDependency {}
impl AsDependencyTemplate for ContainerExposedDependency {}
