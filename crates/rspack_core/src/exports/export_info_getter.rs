use std::borrow::Cow;

use itertools::Itertools;
use rspack_util::atom::Atom;
use rustc_hash::FxHashMap as HashMap;

use super::{
  ExportInfoData, ExportInfoTargetValue, ExportProvided, ExportsInfo, Inlinable, UsageState,
  UsedNameItem,
};
use crate::{DependencyId, RuntimeSpec};

pub struct ExportInfoGetter;

impl ExportInfoGetter {
  pub fn name(info: &ExportInfoData) -> Option<&Atom> {
    info.name.as_ref()
  }

  pub fn provided(info: &ExportInfoData) -> Option<&ExportProvided> {
    info.provided.as_ref()
  }

  pub fn can_mangle_provide(info: &ExportInfoData) -> Option<bool> {
    info.can_mangle_provide
  }

  pub fn can_mangle_use(info: &ExportInfoData) -> Option<bool> {
    info.can_mangle_use
  }

  pub fn terminal_binding(info: &ExportInfoData) -> bool {
    info.terminal_binding
  }

  pub fn exports_info_owned(info: &ExportInfoData) -> bool {
    info.exports_info_owned
  }

  pub fn exports_info(info: &ExportInfoData) -> Option<ExportsInfo> {
    info.exports_info
  }

  pub fn inlinable(info: &ExportInfoData) -> &Inlinable {
    &info.inlinable
  }

  /// Webpack returns `false | string`, we use `Option<Atom>` to avoid declare a redundant enum
  /// type
  pub fn get_used_name(
    info: &ExportInfoData,
    fallback_name: Option<&Atom>,
    runtime: Option<&RuntimeSpec>,
  ) -> Option<UsedNameItem> {
    if let Inlinable::Inlined(inlined) = &info.inlinable {
      return Some(UsedNameItem::Inlined(*inlined));
    }
    if info.has_use_in_runtime_info {
      if let Some(usage) = info.global_used {
        if matches!(usage, UsageState::Unused) {
          return None;
        }
      } else if let Some(used_in_runtime) = info.used_in_runtime.as_ref() {
        if let Some(runtime) = runtime {
          if runtime
            .iter()
            .all(|item| !used_in_runtime.contains_key(item))
          {
            return None;
          }
        }
      } else {
        return None;
      }
    }
    if let Some(used_name) = info.used_name.as_ref() {
      return Some(UsedNameItem::Str(used_name.clone()));
    }
    if let Some(name) = info.name.as_ref() {
      Some(UsedNameItem::Str(name.clone()))
    } else {
      fallback_name.map(|n| UsedNameItem::Str(n.clone()))
    }
  }

  pub fn get_used(info: &ExportInfoData, runtime: Option<&RuntimeSpec>) -> UsageState {
    if !info.has_use_in_runtime_info {
      return UsageState::NoInfo;
    }
    if let Some(global_used) = info.global_used {
      return global_used;
    }
    if let Some(used_in_runtime) = info.used_in_runtime.as_ref() {
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

  pub fn get_provided_info(info: &ExportInfoData) -> &'static str {
    match info.provided {
      Some(ExportProvided::NotProvided) => "not provided",
      Some(ExportProvided::Unknown) => "maybe provided (runtime-defined)",
      Some(ExportProvided::Provided) => "provided",
      None => "no provided info",
    }
  }

  pub fn get_rename_info(info: &ExportInfoData) -> Cow<str> {
    match (&info.used_name, &info.name) {
      (Some(used), Some(name)) if used != name => return format!("renamed to {used}").into(),
      (Some(used), None) => return format!("renamed to {used}").into(),
      _ => {}
    }

    match (Self::can_mangle_provide(info), Self::can_mangle_use(info)) {
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

  pub fn get_used_info(info: &ExportInfoData) -> Cow<str> {
    if let Some(global_used) = info.global_used {
      return match global_used {
        UsageState::Unused => "unused".into(),
        UsageState::NoInfo => "no usage info".into(),
        UsageState::Unknown => "maybe used (runtime-defined)".into(),
        UsageState::Used => "used".into(),
        UsageState::OnlyPropertiesUsed => "only properties used".into(),
      };
    } else if let Some(used_in_runtime) = &info.used_in_runtime {
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

    if info.has_use_in_runtime_info {
      "unused".into()
    } else {
      "no usage info".into()
    }
  }

  pub fn is_reexport(info: &ExportInfoData) -> bool {
    !info.terminal_binding && info.target_is_set && !info.target.is_empty()
  }

  pub fn has_info(
    info: &ExportInfoData,
    base_info: &ExportInfoData,
    runtime: Option<&RuntimeSpec>,
  ) -> bool {
    info.used_name.is_some()
      || info.provided.is_some()
      || info.terminal_binding
      || (Self::get_used(info, runtime) != Self::get_used(base_info, runtime))
  }

  pub fn can_mangle(info: &ExportInfoData) -> Option<bool> {
    match info.can_mangle_provide {
      Some(true) => info.can_mangle_use,
      Some(false) => Some(false),
      None => {
        if info.can_mangle_use == Some(false) {
          Some(false)
        } else {
          None
        }
      }
    }
  }

  pub fn has_used_name(info: &ExportInfoData) -> bool {
    info.used_name.is_some()
  }

  pub fn get_max_target(
    info: &ExportInfoData,
  ) -> Cow<HashMap<Option<DependencyId>, ExportInfoTargetValue>> {
    if info.target.len() <= 1 {
      return Cow::Borrowed(&info.target);
    }
    let mut max_priority = u8::MIN;
    let mut min_priority = u8::MAX;
    for value in info.target.values() {
      max_priority = max_priority.max(value.priority);
      min_priority = min_priority.min(value.priority);
    }
    if max_priority == min_priority {
      return Cow::Borrowed(&info.target);
    }
    let mut map = HashMap::default();
    for (k, v) in info.target.iter() {
      if max_priority == v.priority {
        map.insert(*k, v.clone());
      }
    }
    Cow::Owned(map)
  }
}
