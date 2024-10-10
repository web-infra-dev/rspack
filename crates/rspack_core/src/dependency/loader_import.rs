use rspack_cacheable::{cacheable, cacheable_dyn};

use super::AffectType;
use crate::{
  AsContextDependency, AsDependencyTemplate, Context, Dependency, DependencyCategory, DependencyId,
  DependencyType, ModuleDependency,
};

#[cacheable]
#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct LoaderImportDependency {
  id: DependencyId,
  context: Context,
  request: String,
}

impl LoaderImportDependency {
  pub fn new(request: String, context: Context) -> Self {
    Self {
      request,
      context,
      id: DependencyId::new(),
    }
  }
}

impl AsDependencyTemplate for LoaderImportDependency {}
impl AsContextDependency for LoaderImportDependency {}

#[cacheable_dyn]
impl Dependency for LoaderImportDependency {
  fn id(&self) -> &DependencyId {
    &self.id
  }

  fn get_context(&self) -> Option<&Context> {
    Some(&self.context)
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

#[cacheable_dyn]
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
