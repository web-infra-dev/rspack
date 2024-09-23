use super::AffectType;
use crate::{
  AsContextDependency, AsDependencyTemplate, Context, Dependency, DependencyCategory, DependencyId,
  DependencyType, ModuleDependency, ModuleIdentifier,
};

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct LoaderImportDependency {
  id: DependencyId,
  pub context: Context,
  pub original_module: Option<ModuleIdentifier>,
  request: String,
}

impl LoaderImportDependency {
  pub fn new(request: String, context: Context, original_module: Option<ModuleIdentifier>) -> Self {
    Self {
      request,
      context,
      original_module,
      id: DependencyId::new(),
    }
  }
}

impl AsDependencyTemplate for LoaderImportDependency {}
impl AsContextDependency for LoaderImportDependency {}

impl Dependency for LoaderImportDependency {
  fn id(&self) -> &DependencyId {
    &self.id
  }

  fn category(&self) -> &DependencyCategory {
    &DependencyCategory::LoaderImport
  }

  fn dependency_type(&self) -> &DependencyType {
    &DependencyType::LoaderImport
  }

  fn could_affect_referencing_module(&self) -> AffectType {
    AffectType::True
  }
}

impl ModuleDependency for LoaderImportDependency {
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
