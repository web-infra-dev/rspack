use rspack_cacheable::{cacheable, cacheable_dyn};
use rspack_core::{
  AffectType, AsContextDependency, AsDependencyTemplate, Dependency, DependencyCategory,
  DependencyId, DependencyType, ModuleDependency,
};

#[cacheable]
#[derive(Debug, Clone)]
pub struct DelegatedSourceDependency {
  id: DependencyId,
  request: String,
}

impl DelegatedSourceDependency {
  pub fn new(request: String) -> Self {
    Self {
      id: DependencyId::new(),
      request,
    }
  }
}

#[cacheable_dyn]
impl Dependency for DelegatedSourceDependency {
  fn id(&self) -> &DependencyId {
    &self.id
  }

  fn could_affect_referencing_module(&self) -> AffectType {
    AffectType::True
  }

  fn category(&self) -> &DependencyCategory {
    &DependencyCategory::Esm
  }

  fn dependency_type(&self) -> &DependencyType {
    &DependencyType::DelegatedSource
  }
}

#[cacheable_dyn]
impl ModuleDependency for DelegatedSourceDependency {
  fn request(&self) -> &str {
    &self.request
  }

  fn set_request(&mut self, request: String) {
    self.request = request;
  }
}

impl AsContextDependency for DelegatedSourceDependency {}

impl AsDependencyTemplate for DelegatedSourceDependency {}
