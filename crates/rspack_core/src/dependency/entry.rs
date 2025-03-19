use rspack_cacheable::{cacheable, cacheable_dyn};

use super::{AffectType, FactorizeInfo};
use crate::{
  AsContextDependency, AsDependencyTemplate, Context, Dependency, DependencyCategory, DependencyId,
  DependencyType, ModuleDependency, ModuleLayer,
};

#[cacheable]
#[derive(Debug, Clone)]
pub struct EntryDependency {
  id: DependencyId,
  request: String,
  context: Context,
  layer: Option<ModuleLayer>,
  is_global: bool,
  factorize_info: FactorizeInfo,
}

impl PartialEq for EntryDependency {
  fn eq(&self, other: &Self) -> bool {
    self.id == other.id
      && self.context == other.context
      && self.request == other.request
      && self.layer == other.layer
      && self.is_global == other.is_global
  }
}

impl Eq for EntryDependency {}

impl std::hash::Hash for EntryDependency {
  fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
    self.id.hash(state);
    self.context.hash(state);
    self.request.hash(state);
    self.layer.hash(state);
    self.is_global.hash(state);
  }
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
      factorize_info: Default::default(),
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

  fn factorize_info(&self) -> &FactorizeInfo {
    &self.factorize_info
  }

  fn factorize_info_mut(&mut self) -> &mut FactorizeInfo {
    &mut self.factorize_info
  }
}

impl AsDependencyTemplate for EntryDependency {}
impl AsContextDependency for EntryDependency {}
