use rspack_core::{
  AsContextDependency, AsDependencyTemplate, Dependency, DependencyCategory, DependencyId,
  DependencyType, ModuleDependency,
};

use super::provide_shared_plugin::ProvideVersion;

#[derive(Debug, Clone)]
pub struct ProvideSharedDependency {
  id: DependencyId,
  request: String,
  pub share_scope: String,
  pub name: String,
  pub version: ProvideVersion,
  pub eager: bool,
  resource_identifier: String,
}

impl ProvideSharedDependency {
  pub fn new(
    share_scope: String,
    name: String,
    version: ProvideVersion,
    request: String,
    eager: bool,
  ) -> Self {
    let resource_identifier = format!(
      "provide module ({}) {} as {} @ {} {}",
      &share_scope,
      &request,
      &name,
      &version,
      eager.then_some("eager").unwrap_or_default()
    );
    Self {
      id: DependencyId::new(),
      request,
      share_scope,
      name,
      version,
      eager,
      resource_identifier,
    }
  }
}

impl Dependency for ProvideSharedDependency {
  fn id(&self) -> &DependencyId {
    &self.id
  }

  fn dependency_type(&self) -> &DependencyType {
    &DependencyType::ProvideSharedModule
  }

  fn category(&self) -> &DependencyCategory {
    &DependencyCategory::Esm
  }

  fn resource_identifier(&self) -> Option<&str> {
    Some(&self.resource_identifier)
  }
}

impl ModuleDependency for ProvideSharedDependency {
  fn request(&self) -> &str {
    &self.request
  }
}

impl AsContextDependency for ProvideSharedDependency {}
impl AsDependencyTemplate for ProvideSharedDependency {}
