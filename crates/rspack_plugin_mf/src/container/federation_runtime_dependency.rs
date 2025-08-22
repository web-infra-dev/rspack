use std::sync::Arc;

use rspack_cacheable::{cacheable, cacheable_dyn};
use rspack_core::{
  AsContextDependency, AsDependencyCodeGeneration, Dependency, DependencyCategory, DependencyId,
  DependencyType, FactorizeInfo, ModuleDependency,
};

//TODO: consider adding a new variant to DependencyType enum for 'federation runtime dependency'
// For now, using a related existing type or a placeholder.
const FEDERATION_RUNTIME_DEPENDENCY_TYPE: DependencyType = DependencyType::EsmImport;

#[cacheable]
#[derive(Debug, Clone)]
pub struct FederationRuntimeDependency {
  pub id: DependencyId,
  request: String,
  factorize_info: Arc<FactorizeInfo>,
}

impl FederationRuntimeDependency {
  pub fn new(request: String) -> Self {
    Self {
      id: DependencyId::new(),
      request,
      factorize_info: Default::default(),
    }
  }
}

#[cacheable_dyn]
impl Dependency for FederationRuntimeDependency {
  fn id(&self) -> &DependencyId {
    &self.id
  }

  fn category(&self) -> &DependencyCategory {
    &DependencyCategory::Esm // Or another appropriate category
  }

  fn dependency_type(&self) -> &DependencyType {
    &FEDERATION_RUNTIME_DEPENDENCY_TYPE
  }

  fn could_affect_referencing_module(&self) -> rspack_core::AffectType {
    rspack_core::AffectType::False // Runtime dependencies usually don't affect the referencing module's content
  }
}

#[cacheable_dyn]
impl ModuleDependency for FederationRuntimeDependency {
  fn request(&self) -> &str {
    &self.request
  }

  fn factorize_info(&self) -> &Arc<FactorizeInfo> {
    &self.factorize_info
  }

  fn factorize_info_mut(&mut self) -> &mut Arc<FactorizeInfo> {
    &mut self.factorize_info
  }
  // Spawning_effect is not directly translatable, Rust's ownership and borrowing rules apply.
  // Side effects are generally handled by the module's build and code generation logic.
}

impl AsContextDependency for FederationRuntimeDependency {}
impl AsDependencyCodeGeneration for FederationRuntimeDependency {}
