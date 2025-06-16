use std::collections::hash_map::Entry;

use rspack_util::atom::Atom;

use super::{ExportInfoData, ExportInfoTargetValue, Inlinable, UsageFilterFnTy, UsageState};
use crate::{DependencyId, Nullable, RuntimeSpec};

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
}
