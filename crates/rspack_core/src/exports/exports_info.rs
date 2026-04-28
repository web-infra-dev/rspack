use std::{collections::BTreeMap, hash::Hash, sync::atomic::Ordering::Relaxed};

use rspack_cacheable::cacheable;
use rspack_util::{atom::Atom, ext::DynHash};
use rustc_hash::FxHashSet;
use serde::Serialize;

use super::{ExportInfoData, NEXT_EXPORTS_INFO_UKEY};
use crate::{ExportsInfoArtifact, RuntimeSpec};

#[cacheable]
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, Ord, PartialOrd, Serialize)]
pub struct ExportsInfo(u32);

impl ExportsInfo {
  #[allow(clippy::new_without_default)]
  pub fn new() -> Self {
    Self(NEXT_EXPORTS_INFO_UKEY.fetch_add(1, Relaxed))
  }

  pub fn as_data<'a>(&self, exports_info_artifact: &'a ExportsInfoArtifact) -> &'a ExportsInfoData {
    exports_info_artifact.get_exports_info_by_id(self)
  }

  pub fn as_data_mut<'a>(
    &self,
    exports_info_artifact: &'a mut ExportsInfoArtifact,
  ) -> &'a mut ExportsInfoData {
    exports_info_artifact.get_exports_info_mut_by_id(self)
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

  pub fn update_hash(
    &self,
    exports_info_artifact: &ExportsInfoArtifact,
    hasher: &mut dyn std::hash::Hasher,
    runtime: Option<&RuntimeSpec>,
  ) {
    fn export_info_update_hash(
      export_info: &ExportInfoData,
      exports_info_artifact: &ExportsInfoArtifact,
      hasher: &mut dyn std::hash::Hasher,
      runtime: Option<&RuntimeSpec>,
      visited: &mut FxHashSet<ExportsInfo>,
    ) {
      if let Some(used_name) = export_info.used_name() {
        used_name.dyn_hash(hasher);
      } else {
        export_info.name().dyn_hash(hasher);
      }
      export_info.get_used(runtime).dyn_hash(hasher);
      export_info.provided().dyn_hash(hasher);
      export_info.terminal_binding().dyn_hash(hasher);
      export_info.ns_access().dyn_hash(hasher);
      if let Some(exports_info) = export_info.exports_info()
        && !visited.contains(&exports_info)
      {
        exports_info_update_hash(
          exports_info.as_data(exports_info_artifact),
          exports_info_artifact,
          hasher,
          runtime,
          visited,
        );
      }
    }

    fn exports_info_update_hash(
      exports_info: &ExportsInfoData,
      exports_info_artifact: &ExportsInfoArtifact,
      hasher: &mut dyn std::hash::Hasher,
      runtime: Option<&RuntimeSpec>,
      visited: &mut FxHashSet<ExportsInfo>,
    ) {
      visited.insert(exports_info.id());
      let other_export_info = exports_info.other_exports_info();
      let side_effects_only_info = exports_info.side_effects_only_info();

      for export_info in exports_info.exports().values() {
        if export_info.has_info(other_export_info, runtime) {
          export_info_update_hash(export_info, exports_info_artifact, hasher, runtime, visited);
        }
      }

      export_info_update_hash(
        side_effects_only_info,
        exports_info_artifact,
        hasher,
        runtime,
        visited,
      );
      export_info_update_hash(
        other_export_info,
        exports_info_artifact,
        hasher,
        runtime,
        visited,
      );
    }

    let mut visited = FxHashSet::default();
    exports_info_update_hash(self, exports_info_artifact, hasher, runtime, &mut visited);
  }
}
