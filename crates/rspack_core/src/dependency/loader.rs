use super::AffectType;
use crate::{
  AsContextDependency, AsDependencyTemplate, Context, Dependency, DependencyCategory, DependencyId,
  DependencyType, ModuleDependency,
};

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct LoaderDependency {
  id: DependencyId,
  context: Context,
  request: String,
}

impl LoaderDependency {
  pub fn new(request: String, context: Context) -> Self {
    Self {
      request,
      context,
      id: DependencyId::new(),
    }
  }
}

impl AsDependencyTemplate for LoaderDependency {}
impl AsContextDependency for LoaderDependency {}

impl Dependency for LoaderDependency {
  fn id(&self) -> &DependencyId {
    &self.id
  }

  fn category(&self) -> &DependencyCategory {
    &DependencyCategory::Loader
  }

  fn dependency_type(&self) -> &DependencyType {
    &DependencyType::Loader
  }

  fn could_affect_referencing_module(&self) -> AffectType {
    AffectType::True
  }
}

impl ModuleDependency for LoaderDependency {
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
