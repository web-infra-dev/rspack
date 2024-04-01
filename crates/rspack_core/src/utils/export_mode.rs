use rustc_hash::FxHashSet as HashSet;
use swc_core::atoms::Atom;

use crate::{DependencyId, ExportInfoId, RuntimeSpec};

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct GetModeCacheKey {
  pub name: Option<Atom>,
  pub dep_id: DependencyId,
  pub runtime: Option<RuntimeSpec>,
}

impl std::hash::Hash for GetModeCacheKey {
  fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
    self.name.hash(state);
    self.dep_id.hash(state);
  }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NormalReexportItem {
  pub name: Atom,
  pub ids: Vec<Atom>,
  pub hidden: bool,
  pub checked: bool,
  pub export_info: ExportInfoId,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ExportMode {
  /// corresponding to `type` field in webpack's `ExportMode`
  pub ty: ExportModeType,
  pub items: Option<Vec<NormalReexportItem>>,
  pub name: Option<Atom>,
  pub fake_type: u8,
  pub partial_namespace_export_info: Option<ExportInfoId>,
  pub ignored: Option<HashSet<Atom>>,
  pub hidden: Option<HashSet<Atom>>,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ExportModeType {
  Missing,
  Unused,
  EmptyStar,
  ReexportDynamicDefault,
  ReexportNamedDefault,
  ReexportNamespaceObject,
  ReexportFakeNamespaceObject,
  ReexportUndefined,
  NormalReexport,
  DynamicReexport,
}

impl ExportMode {
  pub fn new(ty: ExportModeType) -> Self {
    Self {
      ty,
      items: None,
      name: None,
      fake_type: 0,
      partial_namespace_export_info: None,
      ignored: None,
      hidden: None,
    }
  }
}
