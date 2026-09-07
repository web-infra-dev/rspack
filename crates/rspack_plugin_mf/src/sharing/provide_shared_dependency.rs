use rspack_cacheable::{cacheable, cacheable_dyn};
use rspack_core::{
  AsContextDependency, AsDependencyCodeGeneration, Dependency, DependencyCategory, DependencyId,
  DependencyType, ModuleDependency, ResourceIdentifier,
};

use super::provide_shared_plugin::ProvideVersion;
use crate::{ConsumeVersion, ShareScope};

#[cacheable]
#[derive(Debug)]
pub struct ProvideSharedDependency {
  id: DependencyId,
  request: String,
  pub share_scope: ShareScope,
  pub name: String,
  pub version: ProvideVersion,
  pub eager: bool,
  pub singleton: Option<bool>,
  pub required_version: Option<ConsumeVersion>,
  pub strict_version: Option<bool>,
  pub tree_shaking_mode: Option<String>,
  resource_identifier: ResourceIdentifier,
}

impl ProvideSharedDependency {
  #[allow(clippy::too_many_arguments)]
  pub fn new(
    share_scope: ShareScope,
    name: String,
    version: ProvideVersion,
    request: String,
    eager: bool,
    singleton: Option<bool>,
    required_version: Option<ConsumeVersion>,
    strict_version: Option<bool>,
    tree_shaking_mode: Option<String>,
  ) -> Self {
    let resource_identifier = format!(
      "provide module ({}) {} as {} @ {} {}",
      share_scope.key(),
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
      tree_shaking_mode,
      resource_identifier,
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

  // Match webpack: global shared providers are applied to initial entrypoints only.
  fn skip_async_entrypoints(&self) -> bool {
    true
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
}

impl AsContextDependency for ProvideSharedDependency {}
impl AsDependencyCodeGeneration for ProvideSharedDependency {}
