use rspack_cacheable::{cacheable, cacheable_dyn};
use rspack_core::{
  AsContextDependency, AsDependencyCodeGeneration, Dependency, DependencyCategory, DependencyId,
  DependencyType, ModuleDependency, ModuleLayer, ResourceIdentifier,
};

use super::provide_shared_plugin::ProvideVersion;
use crate::{ConsumeVersion, ShareScope, SharedIdentity, push_identifier_component};

#[cacheable]
#[derive(Debug)]
pub struct ProvideSharedDependency {
  id: DependencyId,
  request: String,
  pub(crate) original_request: String,
  pub(crate) version_inferred: bool,
  pub share_scope: ShareScope,
  pub layer: Option<ModuleLayer>,
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
    layer: Option<ModuleLayer>,
    tree_shaking_mode: Option<String>,
  ) -> Self {
    let mut resource_identifier = String::from("provide module ");
    push_identifier_component(
      &mut resource_identifier,
      &SharedIdentity::new(&share_scope, &name, layer.as_deref()).identifier_key(),
    );
    push_identifier_component(&mut resource_identifier, &request);
    push_identifier_component(&mut resource_identifier, &version.to_string());
    resource_identifier.push(if eager { '1' } else { '0' });
    let resource_identifier = resource_identifier.into();
    Self {
      id: DependencyId::new(),
      original_request: request.clone(),
      version_inferred: false,
      request,
      share_scope,
      layer,
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
  pub(crate) fn with_request_origin(mut self, request: String, version_inferred: bool) -> Self {
    let mut resource_identifier = self.resource_identifier.to_string();
    push_identifier_component(&mut resource_identifier, &request);
    resource_identifier.push(if version_inferred { '1' } else { '0' });
    self.resource_identifier = resource_identifier.into();
    self.original_request = request;
    self.version_inferred = version_inferred;
    self
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

  fn get_layer(&self) -> Option<&ModuleLayer> {
    self.layer.as_ref()
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
