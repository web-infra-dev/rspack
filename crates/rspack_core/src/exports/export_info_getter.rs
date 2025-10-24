use std::borrow::Cow;

use itertools::Itertools;
use rspack_util::atom::Atom;
use rustc_hash::FxHashMap as HashMap;

use super::{ExportInfoData, ExportInfoTargetValue, ExportProvided, UsageState, UsedNameItem};
use crate::{CanInlineUse, DependencyId, RuntimeSpec};

impl ExportInfoData {
  pub fn get_used_name(
    &self,
    fallback_name: Option<&Atom>,
    runtime: Option<&RuntimeSpec>,
  ) -> Option<UsedNameItem> {
    if self.has_use_in_runtime_info() {
      if let Some(usage) = self.global_used() {
        if matches!(usage, UsageState::Unused) {
          return None;
        }
      } else if let Some(used_in_runtime) = self.used_in_runtime() {
        if let Some(runtime) = runtime
          && runtime
            .iter()
            .all(|item| !used_in_runtime.contains_key(item))
        {
          return None;
        }
      } else {
        return None;
      }
    }
    if let Some(used_name) = self.used_name() {
      return Some(used_name.clone());
    }
    if let Some(name) = self.name() {
      Some(UsedNameItem::Str(name.clone()))
    } else {
      fallback_name.map(|n| UsedNameItem::Str(n.clone()))
    }
  }

  pub fn get_used(&self, runtime: Option<&RuntimeSpec>) -> UsageState {
    if !self.has_use_in_runtime_info() {
      return UsageState::NoInfo;
    }
    if let Some(global_used) = self.global_used() {
      return global_used;
    }
    if let Some(used_in_runtime) = self.used_in_runtime() {
      let mut max = UsageState::Unused;
      if let Some(runtime) = runtime {
        for item in runtime.iter() {
          let Some(usage) = used_in_runtime.get(item) else {
            continue;
          };
          match usage {
            UsageState::Used => return UsageState::Used,
            _ => {
              max = std::cmp::max(max, *usage);
            }
          }
        }
      } else {
        for usage in used_in_runtime.values() {
          match usage {
            UsageState::Used => return UsageState::Used,
            _ => {
              max = std::cmp::max(max, *usage);
            }
          }
        }
      }
      max
    } else {
      UsageState::Unused
    }
  }

  pub fn get_provided_info(&self) -> &'static str {
    match self.provided() {
      Some(ExportProvided::NotProvided) => "not provided",
      Some(ExportProvided::Unknown) => "maybe provided (runtime-defined)",
      Some(ExportProvided::Provided) => "provided",
      None => "no provided info",
    }
  }

  pub fn get_rename_info(&self) -> Cow<'_, str> {
    match (self.used_name(), self.name()) {
      (Some(UsedNameItem::Inlined(inlined)), _) => {
        return format!("inlined to {}", inlined.render()).into();
      }
      (Some(UsedNameItem::Str(used)), Some(name)) if used != name => {
        return format!("renamed to {used}").into();
      }
      (Some(UsedNameItem::Str(used)), None) => return format!("renamed to {used}").into(),
      _ => {}
    }

    match (self.can_mangle_provide(), self.can_mangle_use()) {
      (None, None) => "missing provision and use info prevents renaming",
      (None, Some(false)) => "usage prevents renaming (no provision info)",
      (None, Some(true)) => "missing provision info prevents renaming",

      (Some(true), None) => "missing usage info prevents renaming",
      (Some(true), Some(false)) => "usage prevents renaming",
      (Some(true), Some(true)) => "could be renamed",

      (Some(false), None) => "provision prevents renaming (no use info)",
      (Some(false), Some(false)) => "usage and provision prevents renaming",
      (Some(false), Some(true)) => "provision prevents renaming",
    }
    .into()
  }

  pub fn get_used_info(&self) -> Cow<'_, str> {
    if let Some(global_used) = self.global_used() {
      return match global_used {
        UsageState::Unused => "unused".into(),
        UsageState::NoInfo => "no usage info".into(),
        UsageState::Unknown => "maybe used (runtime-defined)".into(),
        UsageState::Used => "used".into(),
        UsageState::OnlyPropertiesUsed => "only properties used".into(),
      };
    } else if let Some(used_in_runtime) = self.used_in_runtime() {
      let mut map = HashMap::default();

      for (runtime, used) in used_in_runtime.iter() {
        let list = map.entry(*used).or_insert(vec![]);
        list.push(runtime);
      }

      let specific_info: Vec<String> = map
        .iter()
        .map(|(used, runtimes)| match used {
          UsageState::NoInfo => format!("no usage info in {}", runtimes.iter().join(", ")),
          UsageState::Unknown => format!(
            "maybe used in {} (runtime-defined)",
            runtimes.iter().join(", ")
          ),
          UsageState::Used => format!("used in {}", runtimes.iter().join(", ")),
          UsageState::OnlyPropertiesUsed => {
            format!("only properties used in {}", runtimes.iter().join(", "))
          }
          UsageState::Unused => {
            // https://github.com/webpack/webpack/blob/1f99ad6367f2b8a6ef17cce0e058f7a67fb7db18/lib/ExportsInfo.js#L1470-L1481
            unreachable!()
          }
        })
        .collect();

      if !specific_info.is_empty() {
        return specific_info.join("; ").into();
      }
    }

    if self.has_use_in_runtime_info() {
      "unused".into()
    } else {
      "no usage info".into()
    }
  }

  pub fn is_reexport(&self) -> bool {
    !self.terminal_binding() && self.target_is_set() && !self.target().is_empty()
  }

  pub fn has_info(&self, base_info: &ExportInfoData, runtime: Option<&RuntimeSpec>) -> bool {
    self.used_name().is_some()
      || self.provided().is_some()
      || self.terminal_binding()
      || (self.get_used(runtime) != base_info.get_used(runtime))
  }

  pub fn can_mangle(&self) -> Option<bool> {
    match self.can_mangle_provide() {
      Some(true) => self.can_mangle_use(),
      Some(false) => Some(false),
      None => {
        if self.can_mangle_use() == Some(false) {
          Some(false)
        } else {
          None
        }
      }
    }
  }

  pub fn can_inline(&self) -> Option<bool> {
    match self.can_inline_provide() {
      Some(_) => self
        .can_inline_use()
        .map(|v| matches!(v, CanInlineUse::Yes)),
      None => None,
    }
  }

  pub fn has_used_name(&self) -> bool {
    self.used_name().is_some()
  }

  pub fn get_max_target(&self) -> Cow<'_, HashMap<Option<DependencyId>, ExportInfoTargetValue>> {
    if self.target().len() <= 1 {
      return Cow::Borrowed(self.target());
    }
    let mut max_priority = u8::MIN;
    let mut min_priority = u8::MAX;
    for value in self.target().values() {
      max_priority = max_priority.max(value.priority);
      min_priority = min_priority.min(value.priority);
    }
    if max_priority == min_priority {
      return Cow::Borrowed(self.target());
    }
    let mut map = HashMap::default();
    for (k, v) in self.target().iter() {
      if max_priority == v.priority {
        map.insert(*k, v.clone());
      }
    }
    Cow::Owned(map)
  }
}
