use rspack_cacheable::{cacheable, cacheable_dyn};
use rspack_core::{
  AsContextDependency, AsDependencyCodeGeneration, Dependency, DependencyCategory, DependencyId,
  DependencyType, FactorizeInfo, ModuleDependency, ResourceIdentifier,
};

use crate::ExposeOptions;

#[cacheable]
#[derive(Debug, Clone)]
pub struct ContainerEntryDependency {
  id: DependencyId,
  pub name: String,
  pub exposes: Vec<(String, ExposeOptions)>,
  pub share_scope: String,
  resource_identifier: ResourceIdentifier,
  pub(crate) enhanced: bool,
  factorize_info: FactorizeInfo,
}

impl ContainerEntryDependency {
  pub fn new(
    name: String,
    exposes: Vec<(String, ExposeOptions)>,
    share_scope: String,
    enhanced: bool,
  ) -> Self {
    let resource_identifier = format!("container-entry-{}", &name).into();
    Self {
      id: DependencyId::new(),
      name,
      exposes,
      share_scope,
      resource_identifier,
      enhanced,
      factorize_info: Default::default(),
    }
  }
}

#[cacheable_dyn]
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

  fn could_affect_referencing_module(&self) -> rspack_core::AffectType {
    rspack_core::AffectType::Transitive
  }
}

#[cacheable_dyn]
impl ModuleDependency for ContainerEntryDependency {
  fn request(&self) -> &str {
    &self.resource_identifier
  }

  fn factorize_info(&self) -> &FactorizeInfo {
    &self.factorize_info
  }

  fn factorize_info_mut(&mut self) -> &mut FactorizeInfo {
    &mut self.factorize_info
  }
}

impl AsContextDependency for ContainerEntryDependency {}
impl AsDependencyCodeGeneration for ContainerEntryDependency {}
