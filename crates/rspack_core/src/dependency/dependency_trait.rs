use std::{any::Any, fmt::Debug};

use dyn_clone::{DynClone, clone_trait_object};
use rspack_cacheable::cacheable_dyn;
use rspack_collections::{IdentifierMap, IdentifierSet};
use rspack_error::Diagnostic;
use rspack_location::DependencyLocation;
use rspack_util::ext::AsAny;

use super::{
  DependencyCategory, DependencyId, DependencyRange, DependencyType, ExportsSpec,
  dependency_template::AsDependencyCodeGeneration, module_dependency::*,
};
use crate::{
  AsContextDependency, ConnectionState, Context, ExtendedReferencedExport, ForwardId,
  ImportAttributes, ImportPhase, LazyUntil, ModuleGraph, ModuleGraphCacheArtifact, ModuleLayer,
  RuntimeSpec, create_exports_object_referenced,
};

#[derive(Debug, Clone, Copy)]
pub enum AffectType {
  True,
  False,
  Transitive,
}

#[cacheable_dyn]
pub trait Dependency:
  AsDependencyCodeGeneration
  + AsContextDependency
  + AsModuleDependency
  + AsAny
  + DynClone
  + Send
  + Sync
  + Debug
{
  fn id(&self) -> &DependencyId;

  fn category(&self) -> &DependencyCategory {
    &DependencyCategory::Unknown
  }

  fn dependency_type(&self) -> &DependencyType {
    &DependencyType::Unknown
  }

  // get issuer context
  fn get_context(&self) -> Option<&Context> {
    None
  }

  // get issuer layer
  fn get_layer(&self) -> Option<&ModuleLayer> {
    None
  }

  fn get_phase(&self) -> ImportPhase {
    ImportPhase::Evaluation
  }

  fn get_attributes(&self) -> Option<&ImportAttributes> {
    None
  }

  fn get_exports(
    &self,
    _mg: &ModuleGraph,
    _module_graph_cache: &ModuleGraphCacheArtifact,
  ) -> Option<ExportsSpec> {
    None
  }

  fn get_module_evaluation_side_effects_state(
    &self,
    _module_graph: &ModuleGraph,
    _module_graph_cache: &ModuleGraphCacheArtifact,
    _module_chain: &mut IdentifierSet,
    _connection_state_cache: &mut IdentifierMap<ConnectionState>,
  ) -> ConnectionState {
    ConnectionState::Active(true)
  }

  fn loc(&self) -> Option<DependencyLocation> {
    None
  }

  fn range(&self) -> Option<DependencyRange> {
    None
  }

  fn source_order(&self) -> Option<i32> {
    None
  }

  fn resource_identifier(&self) -> Option<&str> {
    None
  }

  fn get_diagnostics(
    &self,
    _module_graph: &ModuleGraph,
    _module_graph_cache: &ModuleGraphCacheArtifact,
  ) -> Option<Vec<Diagnostic>> {
    None
  }

  fn get_referenced_exports(
    &self,
    _module_graph: &ModuleGraph,
    _module_graph_cache: &ModuleGraphCacheArtifact,
    _runtime: Option<&RuntimeSpec>,
  ) -> Vec<ExtendedReferencedExport> {
    create_exports_object_referenced()
  }

  fn could_affect_referencing_module(&self) -> AffectType;

  fn forward_id(&self) -> ForwardId {
    ForwardId::All
  }

  fn lazy(&self) -> Option<LazyUntil> {
    None
  }

  fn unset_lazy(&mut self) -> bool {
    false
  }
}

impl dyn Dependency + '_ {
  pub fn downcast_ref<D: Any>(&self) -> Option<&D> {
    self.as_any().downcast_ref::<D>()
  }

  pub fn downcast_mut<D: Any>(&mut self) -> Option<&mut D> {
    self.as_any_mut().downcast_mut::<D>()
  }

  pub fn is<D: Any>(&self) -> bool {
    self.downcast_ref::<D>().is_some()
  }
}

clone_trait_object!(Dependency);

pub type BoxDependency = Box<dyn Dependency>;
