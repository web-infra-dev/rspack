use rspack_core::{
  AsContextDependency, AsDependencyTemplate, Dependency, DependencyCategory, DependencyId,
  DependencyType, ModuleDependency,
};

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

impl ModuleDependency for FallbackItemDependency {
  fn request(&self) -> &str {
    &self.request
  }
}

impl AsContextDependency for FallbackItemDependency {}
impl AsDependencyTemplate for FallbackItemDependency {}
