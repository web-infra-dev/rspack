use super::ExposeOptions;
use crate::{
  AsContextDependency, AsDependencyTemplate, Dependency, DependencyCategory, DependencyId,
  DependencyType, ModuleDependency,
};

#[derive(Debug, Clone)]
pub struct ContainerEntryDependency {
  id: DependencyId,
  pub name: String,
  pub exposes: Vec<(String, ExposeOptions)>,
  pub share_scope: String,
  resource_identifier: String,
}

impl ContainerEntryDependency {
  pub fn new(name: String, exposes: Vec<(String, ExposeOptions)>, share_scope: String) -> Self {
    let resource_identifier = format!("container-entry-{}", &name);
    Self {
      id: DependencyId::new(),
      name,
      exposes,
      share_scope,
      resource_identifier,
    }
  }
}

impl Dependency for ContainerEntryDependency {
  fn dependency_debug_name(&self) -> &'static str {
    "ContainerEntryDependency"
  }

  fn id(&self) -> &DependencyId {
    &self.id
  }

  fn category(&self) -> &DependencyCategory {
    &DependencyCategory::Esm
  }

  fn dependency_type(&self) -> &DependencyType {
    &DependencyType::ContainerEntry
  }

  fn resource_identifier(&self) -> Option<&str> {
    Some(&self.resource_identifier)
  }
}

impl ModuleDependency for ContainerEntryDependency {
  fn request(&self) -> &str {
    &self.resource_identifier
  }

  fn user_request(&self) -> &str {
    &self.resource_identifier
  }

  fn set_request(&mut self, _request: String) {}
}

impl AsContextDependency for ContainerEntryDependency {}
impl AsDependencyTemplate for ContainerEntryDependency {}
