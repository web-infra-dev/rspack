use rspack_cacheable::{cacheable, cacheable_dyn};
use rspack_core::{
  AsContextDependency, AsDependencyCodeGeneration, Dependency, DependencyCategory, DependencyId,
  DependencyType, FactorizeInfo, ModuleDependency, ResourceIdentifier,
};

use crate::plugin_state::ClientModuleImport;

#[cacheable]
#[derive(Debug, Clone)]
pub struct RscEntryDependency {
  id: DependencyId,
  pub name: String,
  pub client_modules: Vec<ClientModuleImport>,
  resource_identifier: ResourceIdentifier,
  #[cacheable(with=rspack_cacheable::with::As<FactorizeInfo>)]
  factorize_info: std::sync::Arc<std::sync::Mutex<FactorizeInfo>>,
}

impl RscEntryDependency {
  pub fn new(name: String, client_modules: Vec<ClientModuleImport>) -> Self {
    let resource_identifier = format!("rsc-client-entry-{}", &name).into();
    Self {
      id: DependencyId::new(),
      name,
      client_modules,
      resource_identifier,
      factorize_info: Default::default(),
    }
  }
}

#[cacheable_dyn]
impl Dependency for RscEntryDependency {
  fn id(&self) -> &DependencyId {
    &self.id
  }

  fn category(&self) -> &DependencyCategory {
    &DependencyCategory::Esm
  }

  fn dependency_type(&self) -> &DependencyType {
    &DependencyType::RscEntry
  }

  fn resource_identifier(&self) -> Option<&str> {
    Some(&self.resource_identifier)
  }

  fn could_affect_referencing_module(&self) -> rspack_core::AffectType {
    rspack_core::AffectType::Transitive
  }
}

#[cacheable_dyn]
impl ModuleDependency for RscEntryDependency {
  fn request(&self) -> &str {
    &self.resource_identifier
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

impl AsContextDependency for RscEntryDependency {}
impl AsDependencyCodeGeneration for RscEntryDependency {}
