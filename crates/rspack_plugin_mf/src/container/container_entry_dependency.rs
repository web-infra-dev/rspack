use rspack_core::{
  AsContextDependency, AsDependencyTemplate, Dependency, DependencyCategory, DependencyId,
  DependencyType, ModuleDependency,
};

use crate::ExposeOptions;

#[derive(Debug, Clone)]
pub struct ContainerEntryDependency {
  id: DependencyId,
  pub name: String,
  pub exposes: Vec<(String, ExposeOptions)>,
  pub share_scope: String,
  resource_identifier: String,
  pub(crate) enhanced: bool,
}

impl ContainerEntryDependency {
  pub fn new(
    name: String,
    exposes: Vec<(String, ExposeOptions)>,
    share_scope: String,
    enhanced: bool,
  ) -> Self {
    let resource_identifier = format!("container-entry-{}", &name);
    Self {
      id: DependencyId::new(),
      name,
      exposes,
      share_scope,
      resource_identifier,
      enhanced,
    }
  }
}

impl Dependency for ContainerEntryDependency {
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
}

impl AsContextDependency for ContainerEntryDependency {}
impl AsDependencyTemplate for ContainerEntryDependency {}
