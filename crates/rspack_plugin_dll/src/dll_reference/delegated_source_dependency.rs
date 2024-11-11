use rspack_core::{
  AffectType, AsContextDependency, AsDependencyTemplate, Dependency, DependencyCategory,
  DependencyId, DependencyType, ModuleDependency,
};

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
