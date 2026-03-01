use rspack_cacheable::{cacheable, cacheable_dyn};
use rspack_core::{
  AsContextDependency, AsDependencyCodeGeneration, Dependency, DependencyCategory, DependencyId,
  DependencyType, FactorizeInfo, ModuleDependency,
};

#[cacheable]
#[derive(Debug, Clone)]
pub struct FallbackItemDependency {
  id: DependencyId,
  request: String,
  #[cacheable(with=rspack_cacheable::with::As<FactorizeInfo>)]
  factorize_info: std::sync::Arc<std::sync::Mutex<FactorizeInfo>>,
}

impl FallbackItemDependency {
  pub fn new(request: String) -> Self {
    Self {
      id: DependencyId::new(),
      request,
      factorize_info: Default::default(),
    }
  }
}

#[cacheable_dyn]
impl Dependency for FallbackItemDependency {
  fn id(&self) -> &DependencyId {
    &self.id
  }

  fn category(&self) -> &DependencyCategory {
    &DependencyCategory::Esm
  }

  fn dependency_type(&self) -> &DependencyType {
    &DependencyType::RemoteToFallbackItem
  }

  fn could_affect_referencing_module(&self) -> rspack_core::AffectType {
    rspack_core::AffectType::True
  }
}

#[cacheable_dyn]
impl ModuleDependency for FallbackItemDependency {
  fn request(&self) -> &str {
    &self.request
  }

  fn factorize_info(&self) -> std::sync::MutexGuard<'_, FactorizeInfo> {
    self
      .factorize_info
      .lock()
      .expect("dependency factorize_info poisoned")
  }

  fn set_factorize_info(&self, info: FactorizeInfo) {
    *self
      .factorize_info
      .lock()
      .expect("dependency factorize_info poisoned") = info;
  }
}

impl AsContextDependency for FallbackItemDependency {}
impl AsDependencyCodeGeneration for FallbackItemDependency {}
