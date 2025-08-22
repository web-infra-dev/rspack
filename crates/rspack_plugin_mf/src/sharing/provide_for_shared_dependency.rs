use std::sync::Arc;

use rspack_cacheable::{cacheable, cacheable_dyn};
use rspack_core::{
  AsContextDependency, AsDependencyCodeGeneration, Dependency, DependencyCategory, DependencyId,
  DependencyType, FactorizeInfo, ModuleDependency,
};

#[cacheable]
#[derive(Debug, Clone)]
pub struct ProvideForSharedDependency {
  id: DependencyId,
  request: String,
  factorize_info: Arc<FactorizeInfo>,
}

impl ProvideForSharedDependency {
  pub fn new(request: String) -> Self {
    Self {
      id: DependencyId::new(),
      request,
      factorize_info: Default::default(),
    }
  }
}

#[cacheable_dyn]
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

  fn could_affect_referencing_module(&self) -> rspack_core::AffectType {
    rspack_core::AffectType::True
  }
}

#[cacheable_dyn]
impl ModuleDependency for ProvideForSharedDependency {
  fn request(&self) -> &str {
    &self.request
  }

  fn factorize_info(&self) -> &Arc<FactorizeInfo> {
    &self.factorize_info
  }

  fn factorize_info_mut(&mut self) -> &mut Arc<FactorizeInfo> {
    &mut self.factorize_info
  }
}

impl AsContextDependency for ProvideForSharedDependency {}
impl AsDependencyCodeGeneration for ProvideForSharedDependency {}
