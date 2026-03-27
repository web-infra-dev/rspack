mod cached_const_dependency;
mod const_dependency;
mod context_dependency;
mod context_element_dependency;
mod dependency_category;
mod dependency_id;
mod dependency_location;
mod dependency_template;
mod dependency_trait;
mod dependency_type;
mod entry;
mod factorize_info;
mod loader_import;
mod module_dependency;
mod runtime_requirements_dependency;
mod static_exports_dependency;

use std::sync::Arc;

pub use cached_const_dependency::{CachedConstDependency, CachedConstDependencyTemplate};
pub use const_dependency::{ConstDependency, ConstDependencyTemplate};
pub use context_dependency::{AsContextDependency, ContextDependency};
pub use context_element_dependency::ContextElementDependency;
pub use dependency_category::DependencyCategory;
pub use dependency_id::*;
pub use dependency_location::*;
pub use dependency_template::*;
pub use dependency_trait::*;
pub use dependency_type::DependencyType;
pub use entry::*;
pub use factorize_info::FactorizeInfo;
pub use loader_import::*;
pub use module_dependency::*;
use rspack_cacheable::{
  cacheable,
  with::{AsPreset, AsVec},
};
pub use runtime_requirements_dependency::{
  RuntimeRequirementsDependency, RuntimeRequirementsDependencyTemplate,
};
use rustc_hash::{FxHashMap, FxHashSet};
use serde::Serialize;
pub use static_exports_dependency::{StaticExportsDependency, StaticExportsSpec};
use swc_core::ecma::atoms::Atom;

use crate::{
  ConnectionState, EvaluatedInlinableValue, ExportsInfoArtifact, ExportsType,
  ExtendedReferencedExport, ModuleGraph, ModuleGraphCacheArtifact, ModuleGraphConnection,
  ModuleIdentifier, ReferencedExport, RuntimeSpec, SideEffectsStateArtifact,
  create_exports_object_referenced,
};

#[derive(Debug, Clone)]
pub enum ProcessModuleReferencedExports {
  Map(FxHashMap<String, ExtendedReferencedExport>),
  ExtendRef(Vec<ExtendedReferencedExport>),
}

#[derive(Debug, Default)]
pub struct ExportSpec {
  pub name: Atom,
  pub export: Option<Nullable<Vec<Atom>>>,
  pub exports: Option<Vec<ExportNameOrSpec>>,
  pub can_mangle: Option<bool>,
  pub terminal_binding: Option<bool>,
  pub priority: Option<u8>,
  pub hidden: Option<bool>,
  pub from: Option<ModuleGraphConnection>,
  pub from_export: Option<ModuleGraphConnection>,
  pub inlinable: Option<EvaluatedInlinableValue>,
}

#[derive(Debug)]
pub enum Nullable<T> {
  Null,
  Value(T),
}

impl ExportSpec {
  pub fn new(name: String) -> Self {
    Self {
      name: Atom::from(name),
      ..Default::default()
    }
  }
}

#[derive(Debug)]
pub enum ExportNameOrSpec {
  String(Atom),
  ExportSpec(ExportSpec),
}

impl Default for ExportNameOrSpec {
  fn default() -> Self {
    Self::String(Atom::default())
  }
}

#[derive(Debug, Default)]
pub enum ExportsOfExportsSpec {
  UnknownExports,
  #[default]
  NoExports,
  Names(Vec<ExportNameOrSpec>),
}

#[derive(Debug, Default)]
#[allow(unused)]
pub struct ExportsSpec {
  pub exports: ExportsOfExportsSpec,
  pub priority: Option<u8>,
  pub can_mangle: Option<bool>,
  pub terminal_binding: Option<bool>,
  pub from: Option<ModuleGraphConnection>,
  pub dependencies: Option<Vec<ModuleIdentifier>>,
  pub hide_export: Option<FxHashSet<Atom>>,
  pub exclude_exports: Option<FxHashSet<Atom>>,
}

impl ExportsSpec {
  pub fn has_nested_exports(&self) -> bool {
    match &self.exports {
      ExportsOfExportsSpec::UnknownExports => false,
      ExportsOfExportsSpec::NoExports => false,
      ExportsOfExportsSpec::Names(exports) => exports.iter().any(|name| match name {
        ExportNameOrSpec::String(_) => false,
        ExportNameOrSpec::ExportSpec(spec) => spec.exports.is_some(),
      }),
    }
  }
}

pub trait DependencyConditionFn: Sync + Send {
  fn get_connection_state(
    &self,
    conn: &ModuleGraphConnection,
    runtime: Option<&RuntimeSpec>,
    module_graph: &ModuleGraph,
    module_graph_cache: &ModuleGraphCacheArtifact,
    side_effects_state_artifact: &SideEffectsStateArtifact,
    exports_info_artifact: &ExportsInfoArtifact,
  ) -> ConnectionState;

  fn is_connection_active(
    &self,
    conn: &ModuleGraphConnection,
    runtime: Option<&RuntimeSpec>,
    module_graph: &ModuleGraph,
    module_graph_cache: &ModuleGraphCacheArtifact,
    side_effects_state_artifact: &SideEffectsStateArtifact,
    exports_info_artifact: &ExportsInfoArtifact,
  ) -> bool {
    self
      .get_connection_state(
        conn,
        runtime,
        module_graph,
        module_graph_cache,
        side_effects_state_artifact,
        exports_info_artifact,
      )
      .is_true()
  }
}

#[derive(Clone)]
pub struct DependencyCondition(Arc<dyn DependencyConditionFn>);

impl DependencyCondition {
  pub fn new(f: impl DependencyConditionFn + 'static) -> Self {
    Self(Arc::new(f))
  }

  pub fn get_connection_state(
    &self,
    connection: &ModuleGraphConnection,
    runtime: Option<&RuntimeSpec>,
    mg: &ModuleGraph,
    module_graph_cache: &ModuleGraphCacheArtifact,
    side_effects_state_artifact: &SideEffectsStateArtifact,
    exports_info_artifact: &ExportsInfoArtifact,
  ) -> ConnectionState {
    self.0.get_connection_state(
      connection,
      runtime,
      mg,
      module_graph_cache,
      side_effects_state_artifact,
      exports_info_artifact,
    )
  }

  pub fn is_connection_active(
    &self,
    connection: &ModuleGraphConnection,
    runtime: Option<&RuntimeSpec>,
    mg: &ModuleGraph,
    module_graph_cache: &ModuleGraphCacheArtifact,
    side_effects_state_artifact: &SideEffectsStateArtifact,
    exports_info_artifact: &ExportsInfoArtifact,
  ) -> bool {
    self.0.is_connection_active(
      connection,
      runtime,
      mg,
      module_graph_cache,
      side_effects_state_artifact,
      exports_info_artifact,
    )
  }
}

impl std::fmt::Debug for DependencyCondition {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "DependencyCondition(...)")
  }
}

#[rspack_cacheable::cacheable]
#[derive(Debug, Clone, Serialize, Default, PartialEq, Eq)]
pub struct ImportAttributes(FxHashMap<String, String>);

impl FromIterator<(String, String)> for ImportAttributes {
  fn from_iter<T: IntoIterator<Item = (String, String)>>(iter: T) -> Self {
    Self(FxHashMap::from_iter(iter))
  }
}

impl ImportAttributes {
  pub fn get(&self, k: &str) -> Option<&str> {
    self.0.get(k).map(|v| v.as_str())
  }

  pub fn insert(&mut self, k: String, v: String) -> Option<String> {
    self.0.insert(k, v)
  }
}

#[rspack_cacheable::cacheable]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum ImportPhase {
  #[default]
  Evaluation,
  Source,
  Defer,
}

impl ImportPhase {
  pub fn is_defer(&self) -> bool {
    matches!(self, ImportPhase::Defer)
  }

  pub fn as_str(&self) -> &'static str {
    match self {
      ImportPhase::Evaluation => "evaluation",
      ImportPhase::Source => "source",
      ImportPhase::Defer => "defer",
    }
  }
}

impl From<swc_core::ecma::ast::ImportPhase> for ImportPhase {
  fn from(phase: swc_core::ecma::ast::ImportPhase) -> Self {
    match phase {
      swc_core::ecma::ast::ImportPhase::Evaluation => Self::Evaluation,
      swc_core::ecma::ast::ImportPhase::Source => Self::Source,
      swc_core::ecma::ast::ImportPhase::Defer => Self::Defer,
    }
  }
}

#[cacheable]
#[derive(Debug, Clone)]
pub struct ReferencedSpecifier {
  #[cacheable(with=AsVec<AsPreset>)]
  pub names: Vec<Atom>,
  pub is_call: bool,
  pub namespace_object_as_context: bool,
}

impl ReferencedSpecifier {
  pub fn new(names: Vec<Atom>) -> Self {
    Self {
      names,
      is_call: false,
      namespace_object_as_context: false,
    }
  }

  pub fn new_call(names: Vec<Atom>, namespace_object_as_context: bool) -> Self {
    Self {
      names,
      is_call: true,
      namespace_object_as_context,
    }
  }
}

pub fn create_referenced_exports_by_referenced_specifiers(
  referenced_specifiers: &[ReferencedSpecifier],
  exports_type: ExportsType,
) -> Vec<ExtendedReferencedExport> {
  let mut refs = vec![];
  for ReferencedSpecifier {
    names,
    is_call,
    namespace_object_as_context,
  } in referenced_specifiers
  {
    let mut names = names.as_slice();
    let mut namespace_object_as_context = *namespace_object_as_context;

    // Force enable namespace object as context for DefaultOnly and DefaultWithNamed
    // because it's more common in cjs and json
    if matches!(
      exports_type,
      ExportsType::DefaultOnly | ExportsType::DefaultWithNamed
    ) {
      namespace_object_as_context = true;
    }

    if let Some(id) = names.first()
      && id == "default"
    {
      match exports_type {
        ExportsType::DefaultOnly | ExportsType::DefaultWithNamed => {
          if names.len() == 1 {
            return create_exports_object_referenced();
          }
          names = &names[1..];
        }
        ExportsType::Dynamic => {
          return create_exports_object_referenced();
        }
        _ => {}
      }
    }

    if namespace_object_as_context && *is_call {
      if names.len() == 1 {
        return create_exports_object_referenced();
      }
      // remove last one
      names = &names[..names.len().saturating_sub(1)];
    }
    refs.push(ExtendedReferencedExport::Export(ReferencedExport::new(
      names.to_vec(),
      false,
      false,
    )));
  }
  refs
}
