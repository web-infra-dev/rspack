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

  // TODO: remove this, we should refactor ExportInfo into ExportName and ExportProvideInfo and ExportUsedInfo
  // ExportProvideInfo is created by FlagDependencyExportsPlugin, and should not mutate after create
  // ExportUsedInfo is created by FlagDependencyUsagePlugin or Plugin::finish_modules, and should not mutate after create
  pub fn reset_provide_info(&self, mg: &mut ModuleGraph) {
    let exports_info = self.as_data_mut(mg);
    for export_info in exports_info.exports_mut().values_mut() {
      export_info.reset_provide_info();
    }
    exports_info
      .side_effects_only_info_mut()
      .reset_provide_info();
    exports_info.other_exports_info_mut().reset_provide_info();
    if let Some(redirect_to) = exports_info.redirect_to() {
      redirect_to.reset_provide_info(mg);
    }
  }

  /// # Panic
  /// it will panic if you provide a export info that does not exists in the module graph
  pub fn set_has_provide_info(&self, mg: &mut ModuleGraph) {
    let exports_info = self.as_data_mut(mg);

    for export_info in exports_info.exports_mut().values_mut() {
      if export_info.provided().is_none() {
        export_info.set_provided(Some(ExportProvided::NotProvided));
      }
      if export_info.can_mangle_provide().is_none() {
        export_info.set_can_mangle_provide(Some(true));
      }
    }
    if let Some(redirect) = exports_info.redirect_to() {
      redirect.set_has_provide_info(mg);
    } else {
      let other_exports_info = exports_info.other_exports_info_mut();
      if other_exports_info.provided().is_none() {
        other_exports_info.set_provided(Some(ExportProvided::NotProvided));
      }
      if other_exports_info.can_mangle_provide().is_none() {
        other_exports_info.set_can_mangle_provide(Some(true));
      }
    }
  }

  pub fn set_unknown_exports_provided(
    &self,
    mg: &mut ModuleGraph,
    can_mangle: bool,
    exclude_exports: Option<&FxHashSet<Atom>>,
    target_key: Option<DependencyId>,
    target_module: Option<DependencyId>,
    priority: Option<u8>,
  ) -> bool {
    let mut changed = false;

    if let Some(exclude_exports) = &exclude_exports {
      for name in exclude_exports.iter() {
        self.get_export_info(mg, name);
      }
    }

    let exports_info = self.as_data_mut(mg);
    for export_info in exports_info.exports_mut().values_mut() {
      if !can_mangle && export_info.can_mangle_provide() != Some(false) {
        export_info.set_can_mangle_provide(Some(false));
        changed = true;
      }
      if let Some(exclude_exports) = &exclude_exports {
        if let Some(export_name) = export_info.name()
          && exclude_exports.contains(export_name)
        {
          continue;
        }
      }
      if !matches!(
        export_info.provided(),
        Some(ExportProvided::Provided | ExportProvided::Unknown)
      ) {
        export_info.set_provided(Some(ExportProvided::Unknown));
        changed = true;
      }
      if let Some(target_key) = target_key {
        let name = export_info
          .name()
          .map(|name| Nullable::Value(vec![name.clone()]));
        export_info.set_target(Some(target_key), target_module, name.as_ref(), priority);
      }
    }

    if let Some(redirect_to) = exports_info.redirect_to() {
      changed |= redirect_to.set_unknown_exports_provided(
        mg,
        can_mangle,
        exclude_exports,
        target_key,
        target_module,
        priority,
      );
    } else {
      let other_exports_info_data = exports_info.other_exports_info_mut();
      if !matches!(
        other_exports_info_data.provided(),
        Some(ExportProvided::Provided | ExportProvided::Unknown)
      ) {
        other_exports_info_data.set_provided(Some(ExportProvided::Unknown));
        changed = true;
      }

      if let Some(target_key) = target_key {
        other_exports_info_data.set_target(Some(target_key), target_module, None, priority);
      }

      if !can_mangle && other_exports_info_data.can_mangle_provide() != Some(false) {
        other_exports_info_data.set_can_mangle_provide(Some(false));
        changed = true;
      }
    }
    changed
  }

  pub fn get_export_info(&self, mg: &mut ModuleGraph, name: &Atom) -> ExportInfo {
    let exports_info = self.as_data_mut(mg);
    if let Some(export_info) = exports_info.named_exports(name) {
      return export_info.id();
    }
    if let Some(redirect) = exports_info.redirect_to() {
      return redirect.get_export_info(mg, name);
    }

    let other_export_info = exports_info.other_exports_info();
    let new_info = ExportInfoData::new(*self, Some(name.clone()), Some(other_export_info));
    let new_info_id = new_info.id();
    exports_info.exports_mut().insert(name.clone(), new_info);
    new_info_id
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
    if let Some(redirect) = exports_info.redirect_to() {
      redirect.set_has_use_info(mg);
    } else {
      let other_exports_info = exports_info.other_exports_info_mut();
      other_exports_info.set_has_use_info(&mut nested_exports_info);
      if other_exports_info.can_mangle_use().is_none() {
        other_exports_info.set_can_mangle_use(Some(true));
      }
    }

    for nested_exports_info in nested_exports_info {
      nested_exports_info.set_has_use_info(mg);
    }
  }

  pub fn set_used_without_info(&self, mg: &mut ModuleGraph, runtime: Option<&RuntimeSpec>) -> bool {
    let mut changed = false;
    let exports_info = self.as_data_mut(mg);
    for export_info in exports_info.exports_mut().values_mut() {
      let flag = export_info.set_used_without_info(runtime);
      changed |= flag;
    }
    if let Some(redirect_to) = exports_info.redirect_to() {
      let flag = redirect_to.set_used_without_info(mg, runtime);
      changed |= flag;
    } else {
      let other_exports_info = exports_info.other_exports_info_mut();
      let flag = other_exports_info.set_used(UsageState::NoInfo, None);
      changed |= flag;
      if other_exports_info.can_mangle_use() != Some(false) {
        other_exports_info.set_can_mangle_use(Some(false));
        changed = true;
      }
    }
    changed
  }

  pub fn set_all_known_exports_used(
    &self,
    mg: &mut ModuleGraph,
    runtime: Option<&RuntimeSpec>,
  ) -> bool {
    let mut changed = false;
    let exports_info = self.as_data_mut(mg);
    for export_info in exports_info.exports_mut().values_mut() {
      if !matches!(export_info.provided(), Some(ExportProvided::Provided)) {
        continue;
      }
      changed |= export_info.set_used(UsageState::Used, runtime);
    }
    changed
  }

  pub fn set_used_in_unknown_way(
    &self,
    mg: &mut ModuleGraph,
    runtime: Option<&RuntimeSpec>,
  ) -> bool {
    let mut changed = false;
    let exports_info = self.as_data_mut(mg);
    for export_info in exports_info.exports_mut().values_mut() {
      if export_info.set_used_in_unknown_way(runtime) {
        changed = true;
      }
    }
    if let Some(redirect_to) = exports_info.redirect_to() {
      if redirect_to.set_used_in_unknown_way(mg, runtime) {
        changed = true;
      }
    } else {
      let other_exports_info = exports_info.other_exports_info_mut();
      if other_exports_info.set_used_conditionally(
        Box::new(|value| value < &UsageState::Unknown),
        UsageState::Unknown,
        runtime,
      ) {
        changed = true;
      }
      if other_exports_info.can_mangle_use() != Some(false) {
        other_exports_info.set_can_mangle_use(Some(false));
        changed = true;
      }
    }
    changed
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
  redirect_to: Option<ExportsInfo>,
  id: ExportsInfo,
}

impl Default for ExportsInfoData {
  fn default() -> Self {
    let id = ExportsInfo::new();
    Self {
      exports: BTreeMap::default(),
      other_exports_info: ExportInfoData::new(id, None, None),
      side_effects_only_info: ExportInfoData::new(id, Some("*side effects only*".into()), None),
      redirect_to: None,
      id,
    }
  }
}

impl ExportsInfoData {
  pub fn id(&self) -> ExportsInfo {
    self.id
  }

  pub fn redirect_to(&self) -> Option<ExportsInfo> {
    self.redirect_to
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

  pub fn set_redirect_to(&mut self, id: Option<ExportsInfo>) {
    self.redirect_to = id;
  }
}
