use std::collections::hash_map::Entry;

use rspack_util::atom::Atom;

use super::{ExportInfoData, ExportInfoTargetValue, UsageFilterFnTy, UsageState};
use crate::{CanInlineUse, DependencyId, Nullable, RuntimeSpec};

impl ExportInfoData {
  pub fn reset_provide_info(&mut self) {
    self.set_provided(None);
    self.set_can_mangle_provide(None);
    self.set_can_inline_provide(None);
    self.set_exports_info(None);
    self.set_exports_info_owned(false);
    self.set_target_is_set(false);
    self.target_mut().clear();
    self.set_terminal_binding(false);
  }

  pub fn unset_target(&mut self, key: &DependencyId) -> bool {
    if !self.target_is_set() {
      false
    } else {
      self.target_mut().remove(&Some(*key)).is_some()
    }
  }

  pub fn set_target(
    &mut self,
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
    if !self.target_is_set() {
      self.target_mut().insert(
        key,
        ExportInfoTargetValue {
          dependency,
          export: export_name.cloned(),
          priority: normalized_priority,
        },
      );
      self.set_target_is_set(true);
      return true;
    }
    let Some(old_target) = self.target_mut().get_mut(&key) else {
      if dependency.is_none() {
        return false;
      }

      self.target_mut().insert(
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

  pub fn do_move_target(&mut self, dependency: DependencyId, target_export: Option<Vec<Atom>>) {
    self.target_mut().clear();
    self.target_mut().insert(
      None,
      ExportInfoTargetValue {
        dependency: Some(dependency),
        export: target_export,
        priority: 0,
      },
    );
    self.set_target_is_set(true);
  }

  pub fn set_used(&mut self, new_value: UsageState, runtime: Option<&RuntimeSpec>) -> bool {
    if let Some(runtime) = runtime {
      let used_in_runtime = self.used_in_runtime_mut();
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
        self.set_used_in_runtime(None);
        changed = true;
      }
      return changed;
    } else if self.global_used() != Some(new_value) {
      self.set_global_used(Some(new_value));
      return true;
    }
    false
  }

  pub fn set_used_conditionally(
    &mut self,
    condition: &UsageFilterFnTy<UsageState>,
    new_value: UsageState,
    runtime: Option<&RuntimeSpec>,
  ) -> bool {
    if let Some(runtime) = runtime {
      let used_in_runtime = self.used_in_runtime_mut();
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
        self.set_used_in_runtime(None);
        changed = true;
      }
      return changed;
    } else if let Some(global_used) = self.global_used() {
      if global_used != new_value && condition(&global_used) {
        self.set_global_used(Some(new_value));
        return true;
      }
    } else {
      self.set_global_used(Some(new_value));
      return true;
    }
    false
  }

  pub fn set_used_in_unknown_way(&mut self, runtime: Option<&RuntimeSpec>) -> bool {
    let mut changed = false;

    let condition: UsageFilterFnTy<UsageState> = Box::new(|value| value < &UsageState::Unknown);
    if self.set_used_conditionally(&condition, UsageState::Unknown, runtime) {
      changed = true;
    }
    if self.can_mangle_use() != Some(false) {
      self.set_can_mangle_use(Some(false));
      changed = true;
    }
    if self.can_inline_use() != Some(CanInlineUse::No) {
      self.set_can_inline_use(Some(CanInlineUse::No));
      changed = true;
    }
    changed
  }

  pub fn set_used_without_info(&mut self, runtime: Option<&RuntimeSpec>) -> bool {
    let mut changed = false;
    let flag = self.set_used(UsageState::NoInfo, runtime);
    changed |= flag;
    if self.can_mangle_use() != Some(false) {
      self.set_can_mangle_use(Some(false));
      changed = true;
    }
    if self.can_inline_use() != Some(CanInlineUse::No) {
      self.set_can_inline_use(Some(CanInlineUse::No));
      changed = true;
    }
    changed
  }

  pub fn set_has_use_info(&mut self) {
    if !self.has_use_in_runtime_info() {
      self.set_has_use_in_runtime_info(true);
    }
    if self.can_mangle_use().is_none() {
      self.set_can_mangle_use(Some(true));
    }
    if self.can_inline_use().is_none() {
      self.set_can_inline_use(Some(CanInlineUse::HasInfo));
    }
  }
}
