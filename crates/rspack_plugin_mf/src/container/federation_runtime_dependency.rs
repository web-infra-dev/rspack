use rspack_cacheable::{cacheable, cacheable_dyn};
use rspack_core::{
  AsContextDependency, AsDependencyCodeGeneration, Dependency, DependencyCategory, DependencyId,
  DependencyType, ModuleDependency,
};

#[cacheable]
#[derive(Debug)]
pub struct FederationRuntimeDependency {
  pub id: DependencyId,
  request: String,
}

impl FederationRuntimeDependency {
  pub fn new(request: String) -> Self {
    Self {
      id: DependencyId::new(),
      request,
    }
  }
}

#[cacheable_dyn]
impl Dependency for FederationRuntimeDependency {
  fn id(&self) -> &DependencyId {
    &self.id
  }

  fn category(&self) -> &DependencyCategory {
    &DependencyCategory::Esm
  }

  fn dependency_type(&self) -> &DependencyType {
    &DependencyType::FederationRuntime
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

  // Spawning_effect is not directly translatable, Rust's ownership and borrowing rules apply.
  // Side effects are generally handled by the module's build and code generation logic.
}

impl AsContextDependency for FederationRuntimeDependency {}
impl AsDependencyCodeGeneration for FederationRuntimeDependency {}
