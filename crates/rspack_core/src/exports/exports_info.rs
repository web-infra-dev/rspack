use std::{collections::BTreeMap, hash::Hash, sync::atomic::Ordering::Relaxed};

use rspack_cacheable::cacheable;
use rspack_collections::{impl_item_ukey, Ukey};
use rspack_util::atom::Atom;
use rustc_hash::FxHashSet;
use serde::Serialize;

use super::{ExportInfo, ExportInfoData, ExportProvided, UsageState, NEXT_EXPORTS_INFO_UKEY};
use crate::{DependencyId, ModuleGraph, Nullable, RuntimeSpec};

#[cacheable]
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, Ord, PartialOrd, Serialize)]
pub struct ExportsInfo(Ukey);

impl_item_ukey!(ExportsInfo);

impl ExportsInfo {
  #[allow(clippy::new_without_default)]
  pub fn new() -> Self {
    Self(NEXT_EXPORTS_INFO_UKEY.fetch_add(1, Relaxed).into())
  }

  pub fn as_data<'a>(&self, mg: &'a ModuleGraph) -> &'a ExportsInfoData {
    mg.get_exports_info_by_id(self)
  }

  pub fn as_data_mut<'a>(&self, mg: &'a mut ModuleGraph) -> &'a mut ExportsInfoData {
    mg.get_exports_info_mut_by_id(self)
  }

  pub fn set_has_use_info(&self, mg: &mut ModuleGraph) {
    let mut nested_exports_info = vec![];
    let exports_info = self.as_data_mut(mg);
    for export_info in exports_info.exports_mut().values_mut() {
      export_info.set_has_use_info(&mut nested_exports_info);
    }
    exports_info
      .side_effects_only_info_mut()
      .set_has_use_info(&mut nested_exports_info);
    let other_exports_info = exports_info.other_exports_info_mut();
    other_exports_info.set_has_use_info(&mut nested_exports_info);
    if other_exports_info.can_mangle_use().is_none() {
      other_exports_info.set_can_mangle_use(Some(true));
    }

    for nested_exports_info in nested_exports_info {
      nested_exports_info.set_has_use_info(mg);
    }
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
