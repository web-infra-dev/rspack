mod cached_const_dependency;
mod const_dependency;
mod context_dependency;
mod context_element_dependency;
mod dependency_category;
mod dependency_id;
mod dependency_macro;
mod dependency_template;
mod dependency_trait;
mod dependency_type;
mod entry;
mod import_dependency_trait;
mod loader_import;
mod module_dependency;
mod runtime_requirements_dependency;
mod runtime_template;
mod span;
mod static_exports_dependency;

use std::sync::Arc;

pub use cached_const_dependency::CachedConstDependency;
pub use const_dependency::ConstDependency;
pub use context_dependency::{AsContextDependency, ContextDependency};
pub use context_element_dependency::ContextElementDependency;
pub use dependency_category::DependencyCategory;
pub use dependency_id::*;
pub use dependency_template::*;
pub use dependency_trait::*;
pub use dependency_type::DependencyType;
pub use entry::*;
pub use import_dependency_trait::ImportDependencyTrait;
pub use loader_import::*;
pub use module_dependency::*;
pub use runtime_requirements_dependency::RuntimeRequirementsDependency;
pub use runtime_template::*;
pub use span::SpanExt;
pub use static_exports_dependency::{StaticExportsDependency, StaticExportsSpec};
use swc_core::ecma::atoms::Atom;

use crate::{
  ConnectionState, ModuleGraph, ModuleGraphConnection, ModuleIdentifier, ReferencedExport,
  RuntimeSpec,
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
  True,
  #[default]
  Null,
  Array(Vec<ExportNameOrSpec>),
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
  pub hide_export: Option<Vec<Atom>>,
  pub exclude_exports: Option<Vec<Atom>>,
}

pub enum ExportsReferencedType {
  No,     // NO_EXPORTS_REFERENCED
  Object, // EXPORTS_OBJECT_REFERENCED
  String(Box<Vec<Vec<Atom>>>),
  Value(Box<Vec<ReferencedExport>>),
}

impl From<Atom> for ExportsReferencedType {
  fn from(value: Atom) -> Self {
    ExportsReferencedType::String(Box::new(vec![vec![value]]))
  }
}

impl From<Vec<Vec<Atom>>> for ExportsReferencedType {
  fn from(value: Vec<Vec<Atom>>) -> Self {
    ExportsReferencedType::String(Box::new(value))
  }
}

impl From<Vec<Atom>> for ExportsReferencedType {
  fn from(value: Vec<Atom>) -> Self {
    ExportsReferencedType::String(Box::new(vec![value]))
  }
}

impl From<Vec<ReferencedExport>> for ExportsReferencedType {
  fn from(value: Vec<ReferencedExport>) -> Self {
    ExportsReferencedType::Value(Box::new(value))
  }
}

pub type DependencyConditionFn = Arc<
  dyn Fn(&ModuleGraphConnection, Option<&RuntimeSpec>, &ModuleGraph) -> ConnectionState
    + Send
    + Sync,
>;

#[derive(Clone)]
pub enum DependencyCondition {
  False,
  Fn(DependencyConditionFn),
}

impl std::fmt::Debug for DependencyCondition {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      // Self::Nil => write!(f, "Nil"),
      Self::False => write!(f, "False"),
      Self::Fn(_) => write!(f, "Fn"),
    }
  }
}
