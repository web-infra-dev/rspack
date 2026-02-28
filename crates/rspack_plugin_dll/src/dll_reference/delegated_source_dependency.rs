use rspack_cacheable::{cacheable, cacheable_dyn};
use rspack_core::{
  AffectType, AsContextDependency, AsDependencyCodeGeneration, Dependency, DependencyCategory,
  DependencyId, DependencyType, FactorizeInfo, ModuleDependency,
};

#[cacheable]
#[derive(Debug, Clone)]
pub struct DelegatedSourceDependency {
  id: DependencyId,
  request: String,
  #[cacheable(with=rspack_cacheable::with::Skip)]
  factorize_info: std::sync::Arc<std::sync::Mutex<FactorizeInfo>>,
}

impl DelegatedSourceDependency {
  pub fn new(request: String) -> Self {
    Self {
      id: DependencyId::new(),
      request,
      factorize_info: Default::default(),
    }
  }
}

#[cacheable_dyn]
impl Dependency for DelegatedSourceDependency {
  fn id(&self) -> &DependencyId {
    &self.id
  }

  fn could_affect_referencing_module(&self) -> AffectType {
    AffectType::True
  }

  fn category(&self) -> &DependencyCategory {
    &DependencyCategory::Esm
  }

  fn dependency_type(&self) -> &DependencyType {
    &DependencyType::DelegatedSource
  }
}

#[cacheable_dyn]
impl ModuleDependency for DelegatedSourceDependency {
  fn request(&self) -> &str {
    &self.request
  }

  fn factorize_info(&self) -> FactorizeInfo {
    self
      .factorize_info
      .lock()
      .expect("dependency factorize_info poisoned")
      .clone()
  }

  fn set_factorize_info(&self, info: FactorizeInfo) {
    *self
      .factorize_info
      .lock()
      .expect("dependency factorize_info poisoned") = info;
  }
}

impl AsContextDependency for DelegatedSourceDependency {}

impl AsDependencyCodeGeneration for DelegatedSourceDependency {}
