use rspack_core::{
  AsContextDependency, AsDependencyTemplate, Dependency, DependencyCategory, DependencyId,
  DependencyType, ModuleDependency, RealDependencyLocation,
};

#[derive(Debug, Clone)]
pub struct CssComposeDependency {
  id: DependencyId,
  request: String,
  range: RealDependencyLocation,
}

impl CssComposeDependency {
  pub fn new(request: String, range: RealDependencyLocation) -> Self {
    Self {
      id: DependencyId::new(),
      request,
      range,
    }
  }
}

impl Dependency for CssComposeDependency {
  fn id(&self) -> &DependencyId {
    &self.id
  }

  fn category(&self) -> &DependencyCategory {
    &DependencyCategory::CssCompose
  }

  fn dependency_type(&self) -> &DependencyType {
    &DependencyType::CssCompose
  }

  fn range(&self) -> Option<&RealDependencyLocation> {
    Some(&self.range)
  }

  fn could_affect_referencing_module(&self) -> rspack_core::AffectType {
    rspack_core::AffectType::True
  }
}

impl ModuleDependency for CssComposeDependency {
  fn request(&self) -> &str {
    &self.request
  }

  fn user_request(&self) -> &str {
    &self.request
  }

  fn set_request(&mut self, request: String) {
    self.request = request;
  }
}

impl AsDependencyTemplate for CssComposeDependency {}
impl AsContextDependency for CssComposeDependency {}
