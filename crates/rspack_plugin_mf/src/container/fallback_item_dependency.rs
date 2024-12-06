use rspack_cacheable::{cacheable, cacheable_dyn};
use rspack_core::{
  AsContextDependency, AsDependencyTemplate, Dependency, DependencyCategory, DependencyId,
  DependencyType, ModuleDependency,
};

#[cacheable]
#[derive(Debug, Clone)]
pub struct FallbackItemDependency {
  id: DependencyId,
  request: String,
}

impl FallbackItemDependency {
  pub fn new(request: String) -> Self {
    Self {
      id: DependencyId::new(),
      request,
    }
  }
}

#[cacheable_dyn]
impl Dependency for FallbackItemDependency {
  fn id(&self) -> &DependencyId {
    &self.id
  }

  fn category(&self) -> &DependencyCategory {
    &DependencyCategory::Esm
  }

  fn dependency_type(&self) -> &DependencyType {
    &DependencyType::RemoteToFallbackItem
  }

  fn could_affect_referencing_module(&self) -> rspack_core::AffectType {
    rspack_core::AffectType::True
  }
}

#[cacheable_dyn]
impl ModuleDependency for FallbackItemDependency {
  fn request(&self) -> &str {
    &self.request
  }
}

impl AsContextDependency for FallbackItemDependency {}
impl AsDependencyTemplate for FallbackItemDependency {}
