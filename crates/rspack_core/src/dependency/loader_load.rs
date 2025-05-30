use rspack_cacheable::{cacheable, cacheable_dyn};

use super::{AffectType, FactorizeInfo};
use crate::{
  AsContextDependency, AsDependencyCodeGeneration, Context, Dependency, DependencyCategory,
  DependencyId, DependencyType, ModuleDependency,
};

#[cacheable]
#[derive(Debug, Clone)]
pub struct LoaderLoadDependency {
  id: DependencyId,
  context: Context,
  request: String,
  factorize_info: FactorizeInfo,
}

impl LoaderLoadDependency {
  pub fn new(request: String, context: Context) -> Self {
    Self {
      request,
      context,
      id: DependencyId::new(),
      factorize_info: Default::default(),
    }
  }
}

impl PartialEq for LoaderLoadDependency {
  fn eq(&self, other: &Self) -> bool {
    self.id == other.id && self.context == other.context && self.request == other.request
  }
}

impl Eq for LoaderLoadDependency {}

impl std::hash::Hash for LoaderLoadDependency {
  fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
    self.id.hash(state);
    self.context.hash(state);
    self.request.hash(state);
  }
}

impl AsDependencyCodeGeneration for LoaderLoadDependency {}
impl AsContextDependency for LoaderLoadDependency {}

#[cacheable_dyn]
impl Dependency for LoaderLoadDependency {
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
impl ModuleDependency for LoaderLoadDependency {
  fn request(&self) -> &str {
    &self.request
  }

  fn user_request(&self) -> &str {
    &self.request
  }

  fn set_request(&mut self, request: String) {
    self.request = request;
  }

  fn factorize_info(&self) -> &FactorizeInfo {
    &self.factorize_info
  }

  fn factorize_info_mut(&mut self) -> &mut FactorizeInfo {
    &mut self.factorize_info
  }
}
