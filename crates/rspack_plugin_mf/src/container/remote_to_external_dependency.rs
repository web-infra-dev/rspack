use rspack_cacheable::{cacheable, cacheable_dyn};
use rspack_core::{
  AsContextDependency, AsDependencyTemplate, Dependency, DependencyCategory, DependencyId,
  DependencyType, ModuleDependency,
};

#[cacheable]
#[derive(Debug, Clone)]
pub struct RemoteToExternalDependency {
  id: DependencyId,
  request: String,
}

impl RemoteToExternalDependency {
  pub fn new(request: String) -> Self {
    Self {
      id: DependencyId::new(),
      request,
    }
  }
}

#[cacheable_dyn]
impl Dependency for RemoteToExternalDependency {
  fn id(&self) -> &DependencyId {
    &self.id
  }

  fn dependency_type(&self) -> &DependencyType {
    &DependencyType::RemoteToExternal
  }

  fn category(&self) -> &DependencyCategory {
    &DependencyCategory::Esm
  }

  fn could_affect_referencing_module(&self) -> rspack_core::AffectType {
    rspack_core::AffectType::True
  }
}

#[cacheable_dyn]
impl ModuleDependency for RemoteToExternalDependency {
  fn request(&self) -> &str {
    &self.request
  }
}

impl AsContextDependency for RemoteToExternalDependency {}
impl AsDependencyTemplate for RemoteToExternalDependency {}
