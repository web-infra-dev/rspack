use rspack_core::{ExportInfo, ExportsInfoId, ModuleGraph, UsageState};

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

use std::collections::HashSet;

#[derive(Debug)]
struct Compiler {}

/// Struct to represent the mangle exports plugin.
#[derive(Debug)]
struct MangleExportsPlugin {
  deterministic: bool,
}

// impl MangleExportsPlugin {
//   /// Constructor for MangleExportsPlugin.
//   fn new(deterministic: bool) -> Self {
//     MangleExportsPlugin { deterministic }
//   }
//
//   /// Apply the plugin.
//   fn apply(&self, compiler: &Compiler) {
//     let deterministic = self.deterministic;
//     compiler
//       .hooks
//       .compilation
//       .tap("MangleExportsPlugin", |compilation| {
//         let module_graph = compilation.module_graph;
//         compilation
//           .hooks
//           .optimize_code_generation
//           .tap("MangleExportsPlugin", |modules| {
//             for module in modules {
//               let is_namespace = module
//                 .build_meta
//                 .map_or(false, |meta| meta.exports_type == "namespace");
//               let exports_info = module_graph.get_exports_info(module);
//               mangle_exports_info(deterministic, exports_info, is_namespace);
//             }
//           });
//       });
//   }
// }

/// Compare function for sorting exports by name.
fn compare_strings_numeric(a: &ExportInfo, b: &ExportInfo) -> std::cmp::Ordering {
  a.name.cmp(&b.name)
}

/// Function to check if mangling is possible.
fn can_mangle(exports_info: &ExportsInfo) -> bool {
  if let Some(usage_state) = exports_info.other_exports_info.get(&None) {
    if usage_state != &UsageState::Unused {
      return false;
    }
  }

  let mut has_something_to_mangle = false;
  for export_info in &exports_info.exports {
    if export_info.can_mangle == true {
      has_something_to_mangle = true;
    }
  }
  has_something_to_mangle
}

/// Function to mangle exports information.
fn mangle_exports_info(deterministic: bool, exports_info: &mut ExportsInfo, is_namespace: bool) {
  if !can_mangle(exports_info) {
    return;
  }

  let mut used_names = HashSet::new();
  let mut mangleable_exports = Vec::new();
  let mut avoid_mangle_non_provided = !is_namespace;

  if !avoid_mangle_non_provided && deterministic {
    for export_info in &exports_info.owned_exports {
      if export_info.provided != Some(false) {
        avoid_mangle_non_provided = true;
        break;
      }
    }
  }

  for export_info in &exports_info.owned_exports {
    let name = &export_info.name;
    if !export_info.has_used_name() {
      if !export_info.can_mangle == true
        || (name.len() == 1 && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_'))
        || (deterministic
          && name.len() == 2
          && (name.starts_with(|c| c.is_ascii_alphabetic() || c == '_')
            || (name.starts_with(|c| c.is_ascii_digit())
              && name.chars().skip(1).all(|c| c.is_ascii_digit()))))
        || (avoid_mangle_non_provided && export_info.provided != Some(true))
      {
        export_info.set_used_name(name.clone());
        used_names.insert(name.clone());
      } else {
        mangleable_exports.push(export_info.clone());
      }
    }

    if let Some(exports_info_owned) = &export_info.exports_info_owned {
      let used = export_info.get_used(None);
      if used == UsageState::OnlyPropertiesUsed || used == UsageState::Unused {
        mangle_exports_info(deterministic, exports_info_owned, false);
      }
    }
  }

  if deterministic {
    assign_deterministic_ids(
      &mut mangleable_exports,
      |e| &e.name,
      compare_strings_numeric,
      |e, id| {
        let name = number_to_identifier(id);
        let size = used_names.len();
        used_names.insert(name.clone());
        if size == used_names.len() {
          false
        } else {
          e.set_used_name(Some(name));
          true
        }
      },
      [
        NUMBER_OF_IDENTIFIER_START_CHARS,
        NUMBER_OF_IDENTIFIER_START_CHARS * NUMBER_OF_IDENTIFIER_CONTINUATION_CHARS,
      ],
      NUMBER_OF_IDENTIFIER_CONTINUATION_CHARS,
      used_names.len(),
    );
  } else {
    let mut used_exports = Vec::new();
    let mut unused_exports = Vec::new();

    for export_info in mangleable_exports {
      if export_info.get_used(None) == UsageState::Unused {
        unused_exports.push(export_info);
      } else {
        used_exports.push(export_info);
      }
    }

    used_exports.sort_by(compare_strings_numeric);
    unused_exports.sort_by(compare_strings_numeric);

    let mut i = 0;
    for list in vec![used_exports, unused_exports] {
      for mut export_info in list {
        let name;
        loop {
          name = number_to_identifier(i);
          if !used_names.contains(name) {
            break;
          }
          i += 1;
        }
        export_info.set_used_name(Some(name));
      }
    }
  }
}

/// Function to assign deterministic IDs.
fn assign_deterministic_ids<T, F, C>(
  list: &mut [T],
  get_name: F,
  comparator: C,
  assign_name: impl Fn(&mut T, usize) -> bool,
  identifier_ranges: [usize; 2],
  continuation_chars: usize,
  used_names_size: usize,
) where
  F: Fn(&T) -> &str,
  C: Fn(&T, &T) -> std::cmp::Ordering,
{
  // Implement the assign_deterministic_ids function based on the original JavaScript code.
  // ...
}

fn number_to_identifier(number: usize) -> String {
  // Implement the number_to_identifier function based on the original JavaScript code.
  // ...
}

fn compare_select<F, T, K>(selector: F, compare_fn: K) -> impl Fn(&T, &T) -> std::cmp::Ordering
where
  F: Fn(&T) -> &K,
  K: Ord,
{
  move |a, b| compare_fn(selector(a), selector(b))
}

fn main() {
  // Entry point
}
