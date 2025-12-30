use rspack_cacheable::{cacheable, cacheable_dyn};
use rspack_core::{
  AsContextDependency, AsDependencyCodeGeneration, Dependency, DependencyCategory, DependencyId,
  DependencyType, FactorizeInfo, ModuleDependency,
};
use serde::Serialize;

#[cacheable]
#[derive(Debug, Clone)]
pub struct SharedContainerEntryDependency {
  id: DependencyId,
  pub name: String,
  pub request: String,
  pub version: String,
  resource_identifier: String,
  factorize_info: FactorizeInfo,
}

#[cacheable]
#[derive(Debug, Clone, Serialize)]
pub struct ShareContainerEntryOptions {
  pub request: String,
}

impl SharedContainerEntryDependency {
  pub fn new(name: String, request: String, version: String) -> Self {
    let resource_identifier = format!("share-container-entry-{}", &name);
    Self {
      id: DependencyId::new(),
      name,
      request,
      version,
      resource_identifier,
      factorize_info: Default::default(),
    }
  }
}

#[cacheable_dyn]
impl Dependency for SharedContainerEntryDependency {
  fn id(&self) -> &DependencyId {
    &self.id
  }

  fn category(&self) -> &DependencyCategory {
    &DependencyCategory::Esm
  }

  fn dependency_type(&self) -> &DependencyType {
    &DependencyType::ShareContainerEntry
  }

  fn resource_identifier(&self) -> Option<&str> {
    Some(&self.resource_identifier)
  }

  fn could_affect_referencing_module(&self) -> rspack_core::AffectType {
    rspack_core::AffectType::Transitive
  }
}

#[cacheable_dyn]
impl ModuleDependency for SharedContainerEntryDependency {
  fn request(&self) -> &str {
    &self.request
  }

  fn factorize_info(&self) -> &FactorizeInfo {
    &self.factorize_info
  }

  fn factorize_info_mut(&mut self) -> &mut FactorizeInfo {
    &mut self.factorize_info
  }
}

impl AsContextDependency for SharedContainerEntryDependency {}
impl AsDependencyCodeGeneration for SharedContainerEntryDependency {}
