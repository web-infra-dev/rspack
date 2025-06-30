use std::collections::hash_map::Entry;

use rspack_util::atom::Atom;

use super::{ExportInfoData, ExportInfoTargetValue, Inlinable, UsageFilterFnTy, UsageState};
use crate::{
  DependencyId, ExportInfo, ExportsInfo, ExportsInfoData, ModuleGraph, Nullable, RuntimeSpec,
};

pub struct ExportInfoSetter;

impl ExportInfoSetter {
  pub fn reset_provide_info(info: &mut ExportInfoData) {
    info.set_provided(None);
    info.set_can_mangle_provide(None);
    info.set_inlinable(Inlinable::NoByProvide);
    info.set_exports_info(None);
    info.set_exports_info_owned(false);
    info.set_target_is_set(false);
    info.target_mut().clear();
    info.set_terminal_binding(false);
  }

  pub fn unset_target(info: &mut ExportInfoData, key: &DependencyId) -> bool {
    if !info.target_is_set() {
      false
    } else {
      info.target_mut().remove(&Some(*key)).is_some()
    }
  }

  pub fn set_target(
    info: &mut ExportInfoData,
    key: Option<DependencyId>,
    dependency: Option<DependencyId>,
    export_name: Option<&Nullable<Vec<Atom>>>,
    priority: Option<u8>,
  ) -> bool {
    let export_name = match export_name {
      Some(Nullable::Null) => None,
      Some(Nullable::Value(vec)) => Some(vec),
      None => None,
    };
    let normalized_priority = priority.unwrap_or(0);
    if !info.target_is_set() {
      info.target_mut().insert(
        key,
        ExportInfoTargetValue {
          dependency,
          export: export_name.cloned(),
          priority: normalized_priority,
        },
      );
      info.set_target_is_set(true);
      return true;
    }
    let Some(old_target) = info.target_mut().get_mut(&key) else {
      if dependency.is_none() {
        return false;
      }

      info.target_mut().insert(
        key,
        ExportInfoTargetValue {
          dependency,
          export: export_name.cloned(),
          priority: normalized_priority,
        },
      );
      return true;
    };
    if old_target.dependency != dependency
      || old_target.priority != normalized_priority
      || old_target.export.as_ref() != export_name
    {
      old_target.export = export_name.cloned();
      old_target.priority = normalized_priority;
      old_target.dependency = dependency;
      return true;
    }

    false
  }

  pub fn do_move_target(
    export_info: &mut ExportInfoData,
    dependency: DependencyId,
    target_export: Option<Vec<Atom>>,
  ) {
    export_info.target_mut().clear();
    export_info.target_mut().insert(
      None,
      ExportInfoTargetValue {
        dependency: Some(dependency),
        export: target_export,
        priority: 0,
      },
    );
    export_info.set_target_is_set(true);
  }

  pub fn set_used(
    info: &mut ExportInfoData,
    new_value: UsageState,
    runtime: Option<&RuntimeSpec>,
  ) -> bool {
    if let Some(runtime) = runtime {
      let used_in_runtime = info.used_in_runtime_mut();
      let mut changed = false;
      for &k in runtime.iter() {
        match used_in_runtime.entry(k) {
          Entry::Occupied(mut occ) => match (&new_value, occ.get()) {
            (new, _) if new == &UsageState::Unused => {
              occ.remove();
              changed = true;
            }
            (new, old) if new != old => {
              occ.insert(new_value);
              changed = true;
            }
            (_new, _old) => {}
          },
          Entry::Vacant(vac) => {
            if new_value != UsageState::Unused {
              vac.insert(new_value);
              changed = true;
            }
          }
        }
      }
      if used_in_runtime.is_empty() {
        info.set_used_in_runtime(None);
        changed = true;
      }
      return changed;
    } else if info.global_used() != Some(new_value) {
      info.set_global_used(Some(new_value));
      return true;
    }
    false
  }

  pub fn set_used_conditionally(
    info: &mut ExportInfoData,
    condition: UsageFilterFnTy<UsageState>,
    new_value: UsageState,
    runtime: Option<&RuntimeSpec>,
  ) -> bool {
    if let Some(runtime) = runtime {
      let used_in_runtime = info.used_in_runtime_mut();
      let mut changed = false;

      for &k in runtime.iter() {
        match used_in_runtime.entry(k) {
          Entry::Occupied(mut occ) => match (&new_value, occ.get()) {
            (new, old) if condition(old) && new == &UsageState::Unused => {
              occ.remove();
              changed = true;
            }
            (new, old) if condition(old) && new != old => {
              occ.insert(new_value);
              changed = true;
            }
            _ => {}
          },
          Entry::Vacant(vac) => {
            if new_value != UsageState::Unused && condition(&UsageState::Unused) {
              vac.insert(new_value);
              changed = true;
            }
          }
        }
      }
      if used_in_runtime.is_empty() {
        info.set_used_in_runtime(None);
        changed = true;
      }
      return changed;
    } else if let Some(global_used) = info.global_used() {
      if global_used != new_value && condition(&global_used) {
        info.set_global_used(Some(new_value));
        return true;
      }
    } else {
      info.set_global_used(Some(new_value));
      return true;
    }
    false
  }

  pub fn set_used_in_unknown_way(info: &mut ExportInfoData, runtime: Option<&RuntimeSpec>) -> bool {
    let mut changed = false;

    if ExportInfoSetter::set_used_conditionally(
      info,
      Box::new(|value| value < &UsageState::Unknown),
      UsageState::Unknown,
      runtime,
    ) {
      changed = true;
    }
    if info.can_mangle_use() != Some(false) {
      info.set_can_mangle_use(Some(false));
      changed = true;
    }
    if info.inlinable().can_inline() {
      info.set_inlinable(Inlinable::NoByUse);
      changed = true;
    }
    changed
  }

  pub fn set_used_without_info(info: &mut ExportInfoData, runtime: Option<&RuntimeSpec>) -> bool {
    let mut changed = false;
    let flag = ExportInfoSetter::set_used(info, UsageState::NoInfo, runtime);
    changed |= flag;
    if info.can_mangle_use() != Some(false) {
      info.set_can_mangle_use(Some(false));
      changed = true;
    }
    if info.inlinable().can_inline() {
      info.set_inlinable(Inlinable::NoByUse);
      changed = true;
    }
    changed
  }

  pub fn create_nested_exports_info(info: &ExportInfo, mg: &mut ModuleGraph) -> ExportsInfo {
    let info = info.as_data_mut(mg);
    if info.exports_info_owned() {
      return info
        .exports_info()
        .expect("should have exports_info when exports_info is true");
    }

    info.set_exports_info_owned(true);
    let new_exports_info = ExportsInfoData::new();
    let new_exports_info_id = new_exports_info.id();

    let old_exports_info = info.exports_info();
    info.set_exports_info_owned(true);
    info.set_exports_info(Some(new_exports_info_id));

    mg.set_exports_info(new_exports_info_id, new_exports_info);

    new_exports_info_id.set_has_provide_info(mg);
    if let Some(exports_info) = old_exports_info {
      exports_info.set_redirect_name_to(mg, Some(new_exports_info_id));
    }
    new_exports_info_id
  }

  pub fn set_has_use_info(info: &mut ExportInfoData, nested_exports_info: &mut Vec<ExportsInfo>) {
    if !info.has_use_in_runtime_info() {
      info.set_has_use_in_runtime_info(true);
    }
    if info.can_mangle_use().is_none() {
      info.set_can_mangle_use(Some(true));
    }
    if info.exports_info_owned()
      && let Some(exports_info) = info.exports_info()
    {
      nested_exports_info.push(exports_info);
    }
  }
}
