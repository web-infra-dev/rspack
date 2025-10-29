use rspack_cacheable::{cacheable, cacheable_dyn};
use rspack_core::{
  AsContextDependency, AsDependencyCodeGeneration, Dependency, DependencyCategory, DependencyId,
  DependencyType, FactorizeInfo, ModuleDependency, ResourceIdentifier,
};

use super::provide_shared_plugin::ProvideVersion;
use crate::ConsumeVersion;

#[cacheable]
#[derive(Debug, Clone)]
pub struct ProvideSharedDependency {
  id: DependencyId,
  request: String,
  pub share_scope: String,
  pub name: String,
  pub version: ProvideVersion,
  pub eager: bool,
  pub singleton: Option<bool>,
  pub required_version: Option<ConsumeVersion>,
  pub strict_version: Option<bool>,
  resource_identifier: ResourceIdentifier,
  factorize_info: FactorizeInfo,
}

impl ProvideSharedDependency {
  #[allow(clippy::too_many_arguments)]
  pub fn new(
    share_scope: String,
    name: String,
    version: ProvideVersion,
    request: String,
    eager: bool,
    singleton: Option<bool>,
    required_version: Option<ConsumeVersion>,
    strict_version: Option<bool>,
  ) -> Self {
    let resource_identifier = format!(
      "provide module ({}) {} as {} @ {} {}",
      &share_scope,
      &request,
      &name,
      &version,
      if eager { "eager" } else { Default::default() },
    )
    .into();
    Self {
      id: DependencyId::new(),
      request,
      share_scope,
      name,
      version,
      eager,
      singleton,
      required_version,
      strict_version,
      resource_identifier,
      factorize_info: Default::default(),
    }
  }
}

#[cacheable_dyn]
impl Dependency for ProvideSharedDependency {
  fn id(&self) -> &DependencyId {
    &self.id
  }

  fn dependency_type(&self) -> &DependencyType {
    &DependencyType::ProvideSharedModule
  }

  fn category(&self) -> &DependencyCategory {
    &DependencyCategory::Esm
  }

  fn resource_identifier(&self) -> Option<&str> {
    Some(&self.resource_identifier)
  }

  fn could_affect_referencing_module(&self) -> rspack_core::AffectType {
    rspack_core::AffectType::Transitive
  }
}

#[cacheable_dyn]
impl ModuleDependency for ProvideSharedDependency {
  fn request(&self) -> &str {
    &self.request
  }

  fn factorize_info(&self) -> &FactorizeInfo {
    &self.factorize_info
  }

  fn factorize_info_mut(&mut self) -> &mut FactorizeInfo {
    &mut self.factorize_info
  }
}

impl AsContextDependency for ProvideSharedDependency {}
impl AsDependencyCodeGeneration for ProvideSharedDependency {}
