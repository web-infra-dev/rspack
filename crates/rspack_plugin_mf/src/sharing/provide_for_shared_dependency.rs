use rspack_core::{
  AsContextDependency, AsDependencyTemplate, Dependency, DependencyCategory, DependencyId,
  DependencyType, ModuleDependency,
};

#[derive(Debug, Clone)]
pub struct ProvideForSharedDependency {
  id: DependencyId,
  request: String,
}

impl ProvideForSharedDependency {
  pub fn new(request: String) -> Self {
    Self {
      id: DependencyId::new(),
      request,
    }
  }
}

impl Dependency for ProvideForSharedDependency {
  fn id(&self) -> &DependencyId {
    &self.id
  }

  fn dependency_type(&self) -> &DependencyType {
    &DependencyType::ProvideModuleForShared
  }

  fn category(&self) -> &DependencyCategory {
    &DependencyCategory::Esm
  }
}

impl ModuleDependency for ProvideForSharedDependency {
  fn request(&self) -> &str {
    &self.request
  }
}

impl AsContextDependency for ProvideForSharedDependency {}
impl AsDependencyTemplate for ProvideForSharedDependency {}
