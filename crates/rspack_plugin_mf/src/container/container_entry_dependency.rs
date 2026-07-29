use rspack_cacheable::{cacheable, cacheable_dyn};
use rspack_core::{
  AsContextDependency, AsDependencyCodeGeneration, Dependency, DependencyCategory, DependencyId,
  DependencyType, FactorizeInfo, ModuleDependency, ResourceIdentifier,
};

use crate::{ExposeOptions, ShareScope, SharedIdentity};

#[cacheable]
#[derive(Debug, Clone)]
pub struct ContainerEntryDependency {
  id: DependencyId,
  pub name: String,
  pub exposes: Vec<(String, ExposeOptions)>,
  pub(crate) expose_layers: Vec<Option<rspack_core::ModuleLayer>>,
  pub share_scope: ShareScope,
  pub request: Option<String>,
  pub version: Option<String>,
  pub(crate) share_key: Option<String>,
  pub(crate) layer: Option<String>,
  resource_identifier: ResourceIdentifier,
  pub(crate) enhanced: bool,
  dependency_type: DependencyType,
  factorize_info: FactorizeInfo,
}

impl ContainerEntryDependency {
  pub fn new(
    name: String,
    exposes: Vec<(String, ExposeOptions)>,
    share_scope: ShareScope,
    enhanced: bool,
  ) -> Self {
    let expose_layers = vec![None; exposes.len()];
    Self::new_with_expose_layers(name, exposes, expose_layers, share_scope, enhanced)
  }

  pub(crate) fn new_with_expose_layers(
    name: String,
    exposes: Vec<(String, ExposeOptions)>,
    expose_layers: Vec<Option<rspack_core::ModuleLayer>>,
    share_scope: ShareScope,
    enhanced: bool,
  ) -> Self {
    let resource_identifier = format!("container-entry-{}", &name).into();
    Self {
      id: DependencyId::new(),
      name,
      exposes,
      expose_layers,
      share_scope,
      request: None,
      version: None,
      share_key: None,
      layer: None,
      resource_identifier,
      enhanced,
      dependency_type: DependencyType::ContainerEntry,
      factorize_info: Default::default(),
    }
  }

  pub(crate) fn new_share_container_entry(
    name: String,
    request: String,
    version: String,
    share_scope: ShareScope,
    share_key: String,
    layer: Option<String>,
  ) -> Self {
    let shared_identity = SharedIdentity::new(&share_scope, &share_key, layer.as_deref());
    let resource_identifier = format!(
      "share-container-entry-{}-{}",
      &name,
      shared_identity.identifier_key()
    )
    .into();
    Self {
      id: DependencyId::new(),
      name,
      exposes: vec![],
      expose_layers: vec![],
      share_scope,
      request: Some(request),
      version: Some(version),
      share_key: Some(share_key),
      layer,
      resource_identifier,
      enhanced: false,
      dependency_type: DependencyType::ShareContainerEntry,
      factorize_info: Default::default(),
    }
  }
}

#[cacheable_dyn]
impl Dependency for ContainerEntryDependency {
  fn id(&self) -> &DependencyId {
    &self.id
  }

  fn category(&self) -> &DependencyCategory {
    &DependencyCategory::Esm
  }

  fn dependency_type(&self) -> &DependencyType {
    &self.dependency_type
  }

  fn get_layer(&self) -> Option<&rspack_core::ModuleLayer> {
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
impl ModuleDependency for ContainerEntryDependency {
  fn request(&self) -> &str {
    if self.dependency_type == DependencyType::ShareContainerEntry {
      self.request.as_deref().unwrap_or_default()
    } else {
      &self.resource_identifier
    }
  }

  fn factorize_info(&self) -> &FactorizeInfo {
    &self.factorize_info
  }

  fn factorize_info_mut(&mut self) -> &mut FactorizeInfo {
    &mut self.factorize_info
  }
}

impl AsContextDependency for ContainerEntryDependency {}
impl AsDependencyCodeGeneration for ContainerEntryDependency {}
