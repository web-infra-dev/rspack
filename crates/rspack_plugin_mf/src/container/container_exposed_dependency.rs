use rspack_cacheable::{cacheable, cacheable_dyn};
use rspack_core::{
  AsContextDependency, AsDependencyCodeGeneration, Dependency, DependencyCategory, DependencyId,
  DependencyType, FactorizeInfo, ModuleDependency, ModuleLayer, ResourceIdentifier,
};

#[cacheable]
#[derive(Debug, Clone)]
pub struct ContainerExposedDependency {
  id: DependencyId,
  request: String,
  pub exposed_name: String,
  layer: Option<ModuleLayer>,
  resource_identifier: ResourceIdentifier,
  dependency_type: DependencyType,
  factorize_info: FactorizeInfo,
}

impl ContainerExposedDependency {
  pub fn new(exposed_name: String, request: String, layer: Option<ModuleLayer>) -> Self {
    let resource_identifier = if let Some(layer) = &layer {
      format!("exposed dependency {exposed_name}={request}|layer={layer}").into()
    } else {
      format!("exposed dependency {exposed_name}={request}").into()
    };
    Self {
      id: DependencyId::new(),
      request,
      exposed_name,
      layer,
      resource_identifier,
      dependency_type: DependencyType::ContainerExposed,
      factorize_info: Default::default(),
    }
  }

  pub fn new_shared_fallback(request: String) -> Self {
    let resource_identifier = format!("share-container-fallback:{request}").into();
    Self {
      id: DependencyId::new(),
      request,
      exposed_name: String::new(),
      layer: None,
      resource_identifier,
      dependency_type: DependencyType::ShareContainerFallback,
      factorize_info: Default::default(),
    }
  }
}

#[cacheable_dyn]
impl Dependency for ContainerExposedDependency {
  fn id(&self) -> &DependencyId {
    &self.id
  }

  fn category(&self) -> &DependencyCategory {
    &DependencyCategory::Esm
  }

  fn dependency_type(&self) -> &DependencyType {
    &self.dependency_type
  }

  fn get_layer(&self) -> Option<&ModuleLayer> {
    self.layer.as_ref()
  }

  fn resource_identifier(&self) -> Option<&str> {
    Some(&self.resource_identifier)
  }

  fn could_affect_referencing_module(&self) -> rspack_core::AffectType {
    rspack_core::AffectType::True
  }
}

#[cacheable_dyn]
impl ModuleDependency for ContainerExposedDependency {
  fn request(&self) -> &str {
    &self.request
  }

  fn user_request(&self) -> &str {
    &self.request
  }

  fn factorize_info(&self) -> &FactorizeInfo {
    &self.factorize_info
  }

  fn factorize_info_mut(&mut self) -> &mut FactorizeInfo {
    &mut self.factorize_info
  }
}

impl AsContextDependency for ContainerExposedDependency {}
impl AsDependencyCodeGeneration for ContainerExposedDependency {}
