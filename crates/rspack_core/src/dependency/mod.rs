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
mod runtime_template;
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
pub use runtime_requirements_dependency::{
  RuntimeRequirementsDependency, RuntimeRequirementsDependencyTemplate,
};
pub use runtime_template::*;
use rustc_hash::{FxHashMap, FxHashSet};
use serde::Serialize;
pub use static_exports_dependency::{StaticExportsDependency, StaticExportsSpec};
use swc_core::ecma::atoms::Atom;

use crate::{
  ConnectionState, EvaluatedInlinableValue, ModuleGraph, ModuleGraphCacheArtifact,
  ModuleGraphConnection, ModuleIdentifier, RuntimeSpec,
};

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
      ExportsOfExportsSpec::Names(names) => names.iter().any(|name| match name {
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
  ) -> ConnectionState;
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
  ) -> ConnectionState {
    self
      .0
      .get_connection_state(connection, runtime, mg, module_graph_cache)
  }
}

impl std::fmt::Debug for DependencyCondition {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "DependencyCondition(...)")
  }
}

#[rspack_cacheable::cacheable]
#[derive(Debug, Clone, Serialize, Default)]
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
