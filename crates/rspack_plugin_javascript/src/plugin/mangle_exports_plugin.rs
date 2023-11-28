use async_trait::async_trait;
use once_cell::sync::Lazy;
use regex::Regex;
use rspack_core::{
  BuildMetaExportsType, Compilation, ExportInfo, ExportInfoProvided, ExportsInfoId, ModuleGraph,
  Plugin, UsageState,
};
use rspack_error::Result;
use rustc_hash::FxHashSet;

fn can_mangle(exports_info_id: ExportsInfoId, mg: &ModuleGraph) -> bool {
  let exports_info = exports_info_id.get_exports_info(mg);
  if exports_info
    .other_exports_info
    .get_export_info(mg)
    .get_used(None)
    != UsageState::Unused
  {
    return false;
  }
  let mut has_something_to_mangle = false;
  for (atom, export_info_id) in exports_info.exports.iter() {
    if export_info_id.get_export_info(mg).can_mangle_use == Some(true) {
      has_something_to_mangle = true;
    }
  }
  return has_something_to_mangle;
}

/// Struct to represent the mangle exports plugin.
#[derive(Debug)]
struct MangleExportsPlugin {
  deterministic: bool,
}

#[async_trait]
impl Plugin for MangleExportsPlugin {
  async fn optimize_code_generation(&self, compilation: &mut Compilation) -> Result<Option<()>> {
    // TODO: should bailout if compilation.moduleMemCache is enable, https://github.com/webpack/webpack/blob/1f99ad6367f2b8a6ef17cce0e058f7a67fb7db18/lib/optimize/MangleExportsPlugin.js#L160-L164
    // We don't do that cause we don't have this option
    let mg = &mut compilation.module_graph;
    let module_id_list = mg
      .module_identifier_to_module
      .keys()
      .cloned()
      .collect::<Vec<_>>();
    for identifier in module_id_list {
      let Some(module) = mg.module_graph_module_by_identifier(&identifier) else {
        continue;
      };
      let is_namespace = module
        .build_meta
        .map(|meta| matches!(meta.exports_type, BuildMetaExportsType::Namespace))
        .unwrap_or_default();
      let exports_info_id = module.exports;
    }
    Ok(None)
  }
}

/// Compare function for sorting exports by name.
fn compare_strings_numeric(a: &ExportInfo, b: &ExportInfo) -> std::cmp::Ordering {
  a.name.cmp(&b.name)
}
static MANGLE_NAME_NORMAL_REG: Lazy<Regex> =
  Lazy::new(|| Regex::new("^[a-zA-Z0-9_$]").expect("should construct regex"));
static MANGLE_NAME_DETERMINSTIC_REG: Lazy<Regex> =
  Lazy::new(|| Regex::new("^[a-zA-Z_$][a-zA-Z0-9_$]|^[1-9][0-9]").expect("should construct regex"));

/// Function to mangle exports information.
fn mangle_exports_info(
  mg: &mut ModuleGraph,
  deterministic: bool,
  exports_info_id: ExportsInfoId,
  is_namespace: bool,
) {
  if !can_mangle(exports_info_id, mg) {
    return;
  }

  let mut used_names = FxHashSet::default();
  let mut mangleable_exports = Vec::new();
  let mut avoid_mangle_non_provided = !is_namespace;

  if !avoid_mangle_non_provided && deterministic {
    let exports_info = exports_info_id.get_exports_info(mg);
    for export_info_id in exports_info.exports.values() {
      let export_info = export_info_id.get_export_info(mg);
      if export_info.provided != Some(ExportInfoProvided::False) {
        avoid_mangle_non_provided = true;
        break;
      }
    }
  }

  let exports_info = exports_info_id.get_exports_info(mg);
  for export_info_id in exports_info.exports.values() {
    let export_info = export_info_id.get_export_info(mg);
    if !export_info.has_used_name() {
      let name = export_info
        .name
        .expect("the name of export_info inserted in exprots_info can not be `None`")
        .clone();
      let can_not_mangle = export_info.can_mangle_use != Some(true)
        || (name.len() == 1 && MANGLE_NAME_NORMAL_REG.is_match(name.as_str()))
        || (avoid_mangle_non_provided && export_info.provided != Some(ExportInfoProvided::True));

      let export_info_mut = export_info_id.get_export_info_mut(mg);
      if can_not_mangle {
        export_info_mut.set_used_name(name);
      } else {
        mangleable_exports.push(export_info_mut.id);
      };
    }
    // if let Some(exports_info_owned) = &export_info.exports_info_owned {
    //   let used = export_info.get_used(None);
    //   if used == UsageState::OnlyPropertiesUsed || used == UsageState::Unused {
    //     mangle_exports_info(deterministic, exports_info_owned, false);
    //   }
    // }
  }

  // if deterministic {
  //   assign_deterministic_ids(
  //     &mut mangleable_exports,
  //     |e| &e.name,
  //     compare_strings_numeric,
  //     |e, id| {
  //       let name = number_to_identifier(id);
  //       let size = used_names.len();
  //       used_names.insert(name.clone());
  //       if size == used_names.len() {
  //         false
  //       } else {
  //         e.set_used_name(Some(name));
  //         true
  //       }
  //     },
  //     [
  //       NUMBER_OF_IDENTIFIER_START_CHARS,
  //       NUMBER_OF_IDENTIFIER_START_CHARS * NUMBER_OF_IDENTIFIER_CONTINUATION_CHARS,
  //     ],
  //     NUMBER_OF_IDENTIFIER_CONTINUATION_CHARS,
  //     used_names.len(),
  //   );
  // } else {
  //   let mut used_exports = Vec::new();
  //   let mut unused_exports = Vec::new();
  //
  //   for export_info in mangleable_exports {
  //     if export_info.get_used(None) == UsageState::Unused {
  //       unused_exports.push(export_info);
  //     } else {
  //       used_exports.push(export_info);
  //     }
  //   }
  //
  //   used_exports.sort_by(compare_strings_numeric);
  //   unused_exports.sort_by(compare_strings_numeric);
  //
  //   let mut i = 0;
  //   for list in vec![used_exports, unused_exports] {
  //     for mut export_info in list {
  //       let name;
  //       loop {
  //         name = number_to_identifier(i);
  //         if !used_names.contains(name) {
  //           break;
  //         }
  //         i += 1;
  //       }
  //       export_info.set_used_name(Some(name));
  //     }
  //   }
  // }
}

// Function to assign deterministic IDs.
// fn assign_deterministic_ids<T, F, C>(
//   list: &mut [T],
//   get_name: F,
//   comparator: C,
//   assign_name: impl Fn(&mut T, usize) -> bool,
//   identifier_ranges: [usize; 2],
//   continuation_chars: usize,
//   used_names_size: usize,
// ) where
//   F: Fn(&T) -> &str,
//   C: Fn(&T, &T) -> std::cmp::Ordering,
// {
//   // Implement the assign_deterministic_ids function based on the original JavaScript code.
//   // ...
// }
//
// fn number_to_identifier(number: usize) -> String {
//   // Implement the number_to_identifier function based on the original JavaScript code.
//   // ...
// }
//
// fn compare_select<F, T, K>(selector: F, compare_fn: K) -> impl Fn(&T, &T) -> std::cmp::Ordering
// where
//   F: Fn(&T) -> &K,
//   K: Ord,
// {
//   move |a, b| compare_fn(selector(a), selector(b))
// }
//
// fn main() {
//   // Entry point
// }
