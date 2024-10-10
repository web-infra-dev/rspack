use rspack_cacheable::{cacheable, cacheable_dyn};
use rspack_core::{
  AsContextDependency, AsDependencyTemplate, Dependency, DependencyCategory, DependencyId,
  DependencyType, ModuleDependency,
};

#[cacheable]
#[derive(Debug, Clone)]
pub struct ConsumeSharedFallbackDependency {
  id: DependencyId,
  request: String,
}

impl ConsumeSharedFallbackDependency {
  pub fn new(request: String) -> Self {
    Self {
      id: DependencyId::new(),
      request,
    }
  }
}

#[cacheable_dyn]
impl Dependency for ConsumeSharedFallbackDependency {
  fn id(&self) -> &DependencyId {
    &self.id
  }

  fn dependency_type(&self) -> &DependencyType {
    &DependencyType::ConsumeSharedFallback
  }

  fn category(&self) -> &DependencyCategory {
    &DependencyCategory::Esm
  }

  fn could_affect_referencing_module(&self) -> rspack_core::AffectType {
    rspack_core::AffectType::True
  }
}

#[cacheable_dyn]
impl ModuleDependency for ConsumeSharedFallbackDependency {
  fn request(&self) -> &str {
    &self.request
  }
}

impl AsContextDependency for ConsumeSharedFallbackDependency {}
impl AsDependencyTemplate for ConsumeSharedFallbackDependency {}
