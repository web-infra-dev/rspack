use rspack_cacheable::{cacheable, cacheable_dyn};
use rspack_core::{
  AsContextDependency, AsDependencyCodeGeneration, Dependency, DependencyCategory, DependencyId,
  DependencyType, FactorizeInfo, ModuleDependency, ResourceIdentifier,
};

#[cacheable]
#[derive(Debug, Clone)]
pub struct ContainerExposedDependency {
  id: DependencyId,
  request: String,
  pub exposed_name: String,
  resource_identifier: ResourceIdentifier,
  factorize_info: FactorizeInfo,
}

impl ContainerExposedDependency {
  pub fn new(exposed_name: String, request: String) -> Self {
    let resource_identifier = format!("exposed dependency {exposed_name}={request}").into();
    Self {
      id: DependencyId::new(),
      request,
      exposed_name,
      resource_identifier,
      factorize_info: Default::default(),
    }
  }
}

#[cacheable_dyn]
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

  fn could_affect_referencing_module(&self) -> rspack_core::AffectType {
    rspack_core::AffectType::True
  }
}

#[cacheable_dyn]
impl ModuleDependency for ContainerExposedDependency {
  fn request(&self) -> &str {
    &self.request
  }

  fn user_request(&self) -> &str {
    &self.request
  }

  fn factorize_info(&self) -> &FactorizeInfo {
    &self.factorize_info
  }

  fn factorize_info_mut(&mut self) -> &mut FactorizeInfo {
    &mut self.factorize_info
  }
}

impl AsContextDependency for ContainerExposedDependency {}
impl AsDependencyCodeGeneration for ContainerExposedDependency {}
