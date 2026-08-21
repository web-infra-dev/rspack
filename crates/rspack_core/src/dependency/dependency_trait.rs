use std::{
  any::Any,
  fmt::Debug,
  sync::{Arc, OnceLock},
};

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
  AsContextDependency, ConnectionState, Context, ExportsInfoArtifact, ForwardId, ImportAttributes,
  ImportPhase, JavascriptParserUrl, LazyUntil, Module, ModuleGraph, ModuleGraphCacheArtifact,
  ModuleLayer, ReferencedExport, RuntimeSpec, SideEffectsStateArtifact,
  create_exports_object_referenced,
};

#[derive(Debug, Clone, Copy)]
pub enum AffectType {
  True,
  False,
  Transitive,
}

/// Module-scoped state shared while collecting diagnostics from its dependencies.
#[derive(Debug, Default)]
pub struct DependencyDiagnosticsContext {
  module_source: OnceLock<Option<Arc<str>>>,
}

impl DependencyDiagnosticsContext {
  fn get_or_init_module_source(&self, init: impl FnOnce() -> Option<Arc<str>>) -> Option<Arc<str>> {
    self.module_source.get_or_init(init).clone()
  }

  /// Lazily materialize the module source once and share it across its diagnostics.
  pub fn module_source(&self, module: &dyn Module) -> Option<Arc<str>> {
    self.get_or_init_module_source(|| {
      module
        .source()
        .map(|source| source.source().into_string_lossy().into())
    })
  }
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

  /// Whether this dependency should be excluded when a global entry include is applied to an
  /// async entrypoint.
  fn skip_async_entrypoints(&self) -> bool {
    false
  }

  fn url_mode(&self) -> Option<JavascriptParserUrl> {
    None
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
    _exports_info_artifact: &ExportsInfoArtifact,
  ) -> Option<ExportsSpec> {
    None
  }

  fn get_module_evaluation_side_effects_state(
    &self,
    _module_graph: &ModuleGraph,
    _module_graph_cache: &ModuleGraphCacheArtifact,
    _side_effects_state_artifact: &SideEffectsStateArtifact,
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
    _exports_info_artifact: &ExportsInfoArtifact,
  ) -> Option<Vec<Diagnostic>> {
    None
  }

  fn get_diagnostics_with_context(
    &self,
    module_graph: &ModuleGraph,
    module_graph_cache: &ModuleGraphCacheArtifact,
    exports_info_artifact: &ExportsInfoArtifact,
    _context: &DependencyDiagnosticsContext,
  ) -> Option<Vec<Diagnostic>> {
    self.get_diagnostics(module_graph, module_graph_cache, exports_info_artifact)
  }

  fn get_referenced_exports(
    &self,
    _module_graph: &ModuleGraph,
    _module_graph_cache: &ModuleGraphCacheArtifact,
    _exports_info_artifact: &ExportsInfoArtifact,
    _runtime: Option<&RuntimeSpec>,
  ) -> Vec<ReferencedExport> {
    create_exports_object_referenced()
  }

  fn could_affect_referencing_module(&self) -> AffectType;

  fn forward_id(&self) -> ForwardId {
    ForwardId::All
  }

  fn lazy(&self) -> Option<LazyUntil> {
    None
  }

  fn set_lazy(&self) {}

  fn unset_lazy(&self) -> bool {
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
