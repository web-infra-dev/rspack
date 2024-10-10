use rspack_cacheable::{cacheable, cacheable_dyn};

use super::AffectType;
use crate::{
  AsContextDependency, AsDependencyTemplate, Context, Dependency, DependencyCategory, DependencyId,
  DependencyType, ModuleDependency, ModuleLayer,
};

#[cacheable]
#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct EntryDependency {
  id: DependencyId,
  request: String,
  context: Context,
  layer: Option<ModuleLayer>,
  is_global: bool,
}

impl EntryDependency {
  pub fn new(
    request: String,
    context: Context,
    layer: Option<ModuleLayer>,
    is_global: bool,
  ) -> Self {
    Self {
      request,
      context,
      layer,
      id: DependencyId::new(),
      is_global,
    }
  }

  pub fn is_global(&self) -> bool {
    self.is_global
  }
}

#[cacheable_dyn]
impl Dependency for EntryDependency {
  fn id(&self) -> &DependencyId {
    &self.id
  }

  fn category(&self) -> &DependencyCategory {
    &DependencyCategory::Esm
  }

  fn dependency_type(&self) -> &DependencyType {
    &DependencyType::Entry
  }

  fn get_context(&self) -> Option<&Context> {
    Some(&self.context)
  }

  fn get_layer(&self) -> Option<&ModuleLayer> {
    self.layer.as_ref()
  }

  fn could_affect_referencing_module(&self) -> AffectType {
    AffectType::True
  }
}

#[cacheable_dyn]
impl ModuleDependency for EntryDependency {
  fn request(&self) -> &str {
    &self.request
  }

  fn user_request(&self) -> &str {
    &self.request
  }

  fn set_request(&mut self, request: String) {
    self.request = request;
  }
}

impl AsDependencyTemplate for EntryDependency {}
impl AsContextDependency for EntryDependency {}
