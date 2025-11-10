use rspack_cacheable::{cacheable, cacheable_dyn};
use rspack_core::{
  AsContextDependency, AsDependencyCodeGeneration, Dependency, DependencyCategory, DependencyId,
  DependencyType, FactorizeInfo, ModuleDependency, ResourceIdentifier,
};

#[cacheable]
#[derive(Debug, Clone)]
pub struct FallbackDependency {
  id: DependencyId,
  resource_identifier: ResourceIdentifier,
  pub requests: Vec<String>,
  factorize_info: FactorizeInfo,
}

impl FallbackDependency {
  pub fn new(requests: Vec<String>) -> Self {
    let resource_identifier = format!("fallback {}", &requests.join(" ")).into();
    Self {
      id: DependencyId::new(),
      resource_identifier,
      requests,
      factorize_info: Default::default(),
    }
  }
}

#[cacheable_dyn]
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

  fn could_affect_referencing_module(&self) -> rspack_core::AffectType {
    rspack_core::AffectType::Transitive
  }
}

#[cacheable_dyn]
impl ModuleDependency for FallbackDependency {
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

impl AsContextDependency for FallbackDependency {}
impl AsDependencyCodeGeneration for FallbackDependency {}
