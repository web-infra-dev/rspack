use std::{collections::BTreeMap, hash::Hash, sync::atomic::Ordering::Relaxed, u32};

use rspack_cacheable::cacheable;
use rspack_collections::{Ukey, impl_item_ukey};
use rspack_util::atom::Atom;
use serde::Serialize;

use super::{ExportInfoData, NEXT_EXPORTS_INFO_UKEY};
use crate::ModuleGraph;

#[cacheable]
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, Ord, PartialOrd, Serialize)]
pub struct ExportsInfo(Ukey);

impl_item_ukey!(ExportsInfo);

impl ExportsInfo {
  #[allow(clippy::new_without_default)]
  pub fn new() -> Self {
    Self(NEXT_EXPORTS_INFO_UKEY.fetch_add(1, Relaxed).into())
  }
  pub fn uninit() -> Self {
    Self(Ukey::new(u32::MAX))
  }
  pub fn as_data<'a>(&self, mg: &'a ModuleGraph) -> &'a ExportsInfoData {
    mg.get_exports_info_by_id(self)
  }

  pub fn as_data_mut<'a>(&self, mg: &'a mut ModuleGraph) -> &'a mut ExportsInfoData {
    mg.get_exports_info_mut_by_id(self)
  }
}

#[derive(Debug, Clone)]
pub struct ExportsInfoData {
  exports: BTreeMap<Atom, ExportInfoData>,

  /// other export info is a strange name and hard to understand
  /// it has 2 meanings:
  /// 1. it is used as factory template, so that we can set one property in one exportsInfo,
  ///    then export info created by it can extends those property
  /// 2. it is used to flag if the whole exportsInfo can be statically analyzed. In many commonjs
  ///    case, we can not statically analyze the exportsInfo, its other_export_info.provided will
  ///    be ExportProvided::Unknown
  other_exports_info: ExportInfoData,

  side_effects_only_info: ExportInfoData,
  id: ExportsInfo,
}

impl Default for ExportsInfoData {
  fn default() -> Self {
    let id = ExportsInfo::new();
    Self {
      exports: BTreeMap::default(),
      other_exports_info: ExportInfoData::new(id, None, None),
      side_effects_only_info: ExportInfoData::new(id, Some("*side effects only*".into()), None),
      id,
    }
  }
}

impl ExportsInfoData {
  pub fn reset(&mut self) {
    let id = self.id;
    *self = ExportsInfoData::default();
    self.id = id;
  }
  pub fn id(&self) -> ExportsInfo {
    self.id
  }

  pub fn other_exports_info(&self) -> &ExportInfoData {
    &self.other_exports_info
  }

  pub fn other_exports_info_mut(&mut self) -> &mut ExportInfoData {
    &mut self.other_exports_info
  }

  pub fn side_effects_only_info(&self) -> &ExportInfoData {
    &self.side_effects_only_info
  }

  pub fn side_effects_only_info_mut(&mut self) -> &mut ExportInfoData {
    &mut self.side_effects_only_info
  }

  pub fn named_exports(&self, name: &Atom) -> Option<&ExportInfoData> {
    self.exports.get(name)
  }

  pub fn named_exports_mut(&mut self, name: &Atom) -> Option<&mut ExportInfoData> {
    self.exports.get_mut(name)
  }

  pub fn exports(&self) -> &BTreeMap<Atom, ExportInfoData> {
    &self.exports
  }

  pub fn exports_mut(&mut self) -> &mut BTreeMap<Atom, ExportInfoData> {
    &mut self.exports
  }
}
