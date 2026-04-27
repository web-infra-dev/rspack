use std::sync::Arc;

use rspack_cacheable::{
  cacheable, cacheable_dyn,
  with::{AsCacheable, AsMap, AsVec},
};
use rspack_core::{
  AsContextDependency, AsDependencyCodeGeneration, Dependency, DependencyCategory, DependencyId,
  DependencyType, FactorizeInfo, ModuleDependency, ResourceIdentifier,
};

use crate::plugin_state::{ClientModuleImport, CssImportsByServerEntry};

#[cacheable]
#[derive(Debug, Clone)]
pub struct RscEntryDependency {
  id: DependencyId,
  pub name: Arc<str>,
  pub client_modules: Vec<ClientModuleImport>,
  #[cacheable(with=AsMap<AsCacheable, AsVec>)]
  pub css_imports_by_server_entry: CssImportsByServerEntry,
  /// When true, client modules are loaded eagerly (not as code-split points).
  /// When false, client modules are dynamic imports (code-split points).
  pub is_server_side_rendering: bool,
  resource_identifier: ResourceIdentifier,
  factorize_info: FactorizeInfo,
}

impl RscEntryDependency {
  pub fn new(
    name: Arc<str>,
    client_modules: Vec<ClientModuleImport>,
    css_imports_by_server_entry: CssImportsByServerEntry,
    is_server_side_rendering: bool,
  ) -> Self {
    let resource_identifier = format!("rsc-client-entry-{}", &name).into();
    Self {
      id: DependencyId::new(),
      name,
      client_modules,
      css_imports_by_server_entry,
      is_server_side_rendering,
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

  fn factorize_info(&self) -> &FactorizeInfo {
    &self.factorize_info
  }

  fn factorize_info_mut(&mut self) -> &mut FactorizeInfo {
    &mut self.factorize_info
  }
}

impl AsContextDependency for RscEntryDependency {}
impl AsDependencyCodeGeneration for RscEntryDependency {}
