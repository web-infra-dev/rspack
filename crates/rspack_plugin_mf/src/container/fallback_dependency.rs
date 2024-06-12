use rspack_core::{
  AsContextDependency, AsDependencyTemplate, Dependency, DependencyCategory, DependencyId,
  DependencyType, ModuleDependency,
};

#[derive(Debug, Clone)]
pub struct FallbackDependency {
  id: DependencyId,
  resource_identifier: String,
  pub requests: Vec<String>,
}

impl FallbackDependency {
  pub fn new(requests: Vec<String>) -> Self {
    let resource_identifier = format!("fallback {}", &requests.join(" "));
    Self {
      id: DependencyId::new(),
      resource_identifier,
      requests,
    }
  }
}

impl Dependency for FallbackDependency {
  fn id(&self) -> &DependencyId {
    &self.id
  }

  fn category(&self) -> &DependencyCategory {
    &DependencyCategory::Esm
  }

  fn dependency_type(&self) -> &DependencyType {
    &DependencyType::RemoteToFallback
  }

  fn resource_identifier(&self) -> Option<&str> {
    Some(&self.resource_identifier)
  }
}

impl ModuleDependency for FallbackDependency {
  fn request(&self) -> &str {
    &self.resource_identifier
  }
}

impl AsContextDependency for FallbackDependency {}
impl AsDependencyTemplate for FallbackDependency {}
