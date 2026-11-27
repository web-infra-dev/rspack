use rspack_cacheable::{cacheable, cacheable_dyn};
use rspack_core::{
  AsContextDependency, AsDependencyCodeGeneration, Dependency, DependencyCategory, DependencyId,
  DependencyType, ModuleDependency, ModuleLayer,
};

#[cacheable]
#[derive(Debug)]
pub struct ConsumeSharedFallbackDependency {
  id: DependencyId,
  request: String,
  layer: Option<ModuleLayer>,
}

impl ConsumeSharedFallbackDependency {
  pub fn new(request: String, layer: Option<ModuleLayer>) -> Self {
    Self {
      id: DependencyId::new(),
      request,
      layer,
    }
  }
}

#[cacheable_dyn]
impl Dependency for ConsumeSharedFallbackDependency {
  fn id(&self) -> &DependencyId {
    &self.id
  }

  fn dependency_type(&self) -> &DependencyType {
    &DependencyType::ConsumeSharedFallback
  }

  fn category(&self) -> &DependencyCategory {
    &DependencyCategory::Esm
  }

  fn get_layer(&self) -> Option<&ModuleLayer> {
    self.layer.as_ref()
  }

  fn could_affect_referencing_module(&self) -> rspack_core::AffectType {
    rspack_core::AffectType::True
  }
}

#[cacheable_dyn]
impl ModuleDependency for ConsumeSharedFallbackDependency {
  fn request(&self) -> &str {
    &self.request
  }
}

impl AsContextDependency for ConsumeSharedFallbackDependency {}
impl AsDependencyCodeGeneration for ConsumeSharedFallbackDependency {}
