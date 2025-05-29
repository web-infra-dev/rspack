use std::{
  hash::Hash,
  sync::{atomic::AtomicU32, Arc},
};

use either::Either;
use rspack_cacheable::{
  cacheable,
  with::{AsPreset, AsVec},
};
use rspack_util::atom::Atom;
use rustc_hash::FxHashSet as HashSet;

use crate::{
  ConnectionState, DependencyCondition, DependencyConditionFn, DependencyId, ModuleGraph,
  ModuleGraphConnection, ModuleIdentifier, RuntimeSpec,
};

pub static NEXT_EXPORTS_INFO_UKEY: AtomicU32 = AtomicU32::new(0);
pub static NEXT_EXPORT_INFO_UKEY: AtomicU32 = AtomicU32::new(0);

#[derive(Debug, Clone, Hash)]
pub struct ExportInfoTargetValue {
  pub dependency: Option<DependencyId>,
  pub export: Option<Vec<Atom>>,
  pub priority: u8,
}

pub enum ProvidedExports {
  Unknown,
  ProvidedAll,
  ProvidedNames(Vec<Atom>),
}

pub enum UsedExports {
  Unknown,
  UsedNamespace(bool),
  UsedNames(Vec<Atom>),
}

#[derive(Debug, Clone)]
pub enum UsedName {
  Normal(Vec<Atom>),
}

impl UsedName {
  pub fn to_used_name_vec(self) -> Vec<Atom> {
    match self {
      UsedName::Normal(vec) => vec,
    }
  }
}

impl AsRef<[Atom]> for UsedName {
  fn as_ref(&self) -> &[Atom] {
    match self {
      UsedName::Normal(vec) => vec,
    }
  }
}

#[derive(Debug, Hash, Clone, Copy)]
pub enum ExportProvided {
  /// The export can be statically analyzed, and it is provided
  Provided,
  /// The export can be statically analyzed, and the it is not provided
  NotProvided,
  /// The export is unknown, we don't know if module really has this export, eg. cjs module
  Unknown,
}

#[derive(Clone, Debug)]
pub struct ResolvedExportInfoTarget {
  pub module: ModuleIdentifier,
  pub export: Option<Vec<Atom>>,
  /// using dependency id to retrieve Connection
  pub dependency: DependencyId,
}

#[derive(Clone, Debug)]
pub enum FindTargetRetEnum {
  Undefined,
  False,
  Value(FindTargetRetValue),
}
#[derive(Clone, Debug)]
pub struct FindTargetRetValue {
  pub module: ModuleIdentifier,
  pub export: Option<Vec<Atom>>,
}

#[derive(Debug, Hash, PartialEq, Eq, Default)]
pub struct UsageKey(pub Vec<Either<Box<UsageKey>, UsageState>>);

impl UsageKey {
  pub fn add(&mut self, value: Either<Box<UsageKey>, UsageState>) {
    self.0.push(value);
  }
}

#[derive(Debug, Clone)]
pub struct UnResolvedExportInfoTarget {
  pub dependency: Option<DependencyId>,
  pub export: Option<Vec<Atom>>,
}

#[derive(Debug)]
pub enum ResolvedExportInfoTargetWithCircular {
  Target(ResolvedExportInfoTarget),
  Circular,
}

pub type UsageFilterFnTy<T> = Box<dyn Fn(&T) -> bool>;

#[derive(Debug, PartialEq, Copy, Clone, Default, Hash, PartialOrd, Ord, Eq)]
pub enum UsageState {
  Unused = 0,
  OnlyPropertiesUsed = 1,
  NoInfo = 2,
  #[default]
  Unknown = 3,
  Used = 4,
}

#[cacheable]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UsedByExports {
  Set(#[cacheable(with=AsVec<AsPreset>)] HashSet<Atom>),
  Bool(bool),
}

#[derive(Clone)]
struct UsedByExportsDependencyCondition {
  dependency_id: DependencyId,
  used_by_exports: HashSet<Atom>,
}

impl DependencyConditionFn for UsedByExportsDependencyCondition {
  fn get_connection_state(
    &self,
    _conn: &ModuleGraphConnection,
    runtime: Option<&RuntimeSpec>,
    mg: &ModuleGraph,
  ) -> ConnectionState {
    let module_identifier = mg
      .get_parent_module(&self.dependency_id)
      .expect("should have parent module");
    let exports_info = mg.get_exports_info(module_identifier);
    for export_name in self.used_by_exports.iter() {
      if exports_info.get_used(mg, &[export_name.clone()], runtime) != UsageState::Unused {
        return ConnectionState::Active(true);
      }
    }
    ConnectionState::Active(false)
  }
}

// https://github.com/webpack/webpack/blob/1f99ad6367f2b8a6ef17cce0e058f7a67fb7db18/lib/optimize/InnerGraph.js#L319-L338
pub fn get_dependency_used_by_exports_condition(
  dependency_id: DependencyId,
  used_by_exports: Option<&UsedByExports>,
) -> Option<DependencyCondition> {
  match used_by_exports {
    Some(UsedByExports::Set(used_by_exports)) => Some(DependencyCondition::Fn(Arc::new(
      UsedByExportsDependencyCondition {
        dependency_id,
        used_by_exports: used_by_exports.clone(),
      },
    ))),
    Some(UsedByExports::Bool(bool)) => {
      if *bool {
        None
      } else {
        Some(DependencyCondition::False)
      }
    }
    None => None,
  }
}

/// refer https://github.com/webpack/webpack/blob/d15c73469fd71cf98734685225250148b68ddc79/lib/FlagDependencyUsagePlugin.js#L64
#[derive(Clone, Debug)]
pub enum ExtendedReferencedExport {
  Array(Vec<Atom>),
  Export(ReferencedExport),
}

pub fn is_no_exports_referenced(exports: &[ExtendedReferencedExport]) -> bool {
  exports.is_empty()
}

pub fn is_exports_object_referenced(exports: &[ExtendedReferencedExport]) -> bool {
  matches!(exports[..], [ExtendedReferencedExport::Array(ref arr)] if arr.is_empty())
}

pub fn create_no_exports_referenced() -> Vec<ExtendedReferencedExport> {
  vec![]
}

pub fn create_exports_object_referenced() -> Vec<ExtendedReferencedExport> {
  vec![ExtendedReferencedExport::Array(vec![])]
}

impl From<Vec<Atom>> for ExtendedReferencedExport {
  fn from(value: Vec<Atom>) -> Self {
    ExtendedReferencedExport::Array(value)
  }
}
impl From<ReferencedExport> for ExtendedReferencedExport {
  fn from(value: ReferencedExport) -> Self {
    ExtendedReferencedExport::Export(value)
  }
}

#[derive(Clone, Debug)]
pub struct ReferencedExport {
  pub name: Vec<Atom>,
  pub can_mangle: bool,
}

impl ReferencedExport {
  pub fn new(_name: Vec<Atom>, _can_mangle: bool) -> Self {
    Self {
      name: _name,
      can_mangle: _can_mangle,
    }
  }
}

impl Default for ReferencedExport {
  fn default() -> Self {
    Self {
      name: vec![],
      can_mangle: true,
    }
  }
}
