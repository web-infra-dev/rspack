use async_trait::async_trait;
use once_cell::sync::Lazy;
use regex::Regex;
use rspack_core::{
  BuildMetaExportsType, Compilation, ExportInfo, ExportInfoProvided, ExportsInfoId, ModuleGraph,
  Plugin, UsageState,
};
use rspack_error::Result;
use rspack_ids::id_helpers::assign_deterministic_ids;
use rustc_hash::{FxHashMap, FxHashSet};

use crate::utils::mangle_exports::{
  number_to_identifier, NUMBER_OF_IDENTIFIER_CONTINUATION_CHARS, NUMBER_OF_IDENTIFIER_START_CHARS,
};

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
  for export_info_id in exports_info.exports.values() {
    if export_info_id.get_export_info(mg).can_mangle() == Some(true) {
      has_something_to_mangle = true;
    }
  }
  has_something_to_mangle
}

/// Struct to represent the mangle exports plugin.
#[derive(Debug)]
pub struct MangleExportsPlugin {
  deterministic: bool,
}

impl MangleExportsPlugin {
  pub fn new(deterministic: bool) -> Self {
    Self { deterministic }
  }
}

#[async_trait]
impl Plugin for MangleExportsPlugin {
  async fn optimize_code_generation(&self, compilation: &mut Compilation) -> Result<Option<()>> {
    // TODO: should bailout if compilation.moduleMemCache is enable, https://github.com/webpack/webpack/blob/1f99ad6367f2b8a6ef17cce0e058f7a67fb7db18/lib/optimize/MangleExportsPlugin.js#L160-L164
    // We don't do that cause we don't have this option
    let mg = compilation.get_module_graph_mut();
    let module_id_list = mg
      .module_identifier_to_module
      .keys()
      .cloned()
      .collect::<Vec<_>>();
    for identifier in module_id_list {
      let (Some(mgm), Some(module)) = (
        mg.module_graph_module_by_identifier(&identifier),
        mg.module_by_identifier(&identifier),
      ) else {
        continue;
      };
      let is_namespace = module
        .build_meta()
        .as_ref()
        .map(|meta| matches!(meta.exports_type, BuildMetaExportsType::Namespace))
        .unwrap_or_default();
      let exports_info_id = mgm.exports;
      mangle_exports_info(mg, self.deterministic, exports_info_id, is_namespace);
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
static MANGLE_NAME_DETERMINISTIC_REG: Lazy<Regex> =
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
      if !matches!(export_info.provided, Some(ExportInfoProvided::False)) {
        avoid_mangle_non_provided = true;
        break;
      }
    }
  }

  let export_info_id_list = exports_info_id
    .get_exports_info(mg)
    .exports
    .values()
    .cloned()
    .collect::<Vec<_>>();
  for export_info_id in export_info_id_list {
    let export_info = export_info_id.get_export_info(mg);
    if !export_info.has_used_name() {
      let name = export_info
        .name
        .as_ref()
        .expect("the name of export_info inserted in exports_info can not be `None`")
        .clone();
      let can_not_mangle = export_info.can_mangle_use != Some(true)
        || (name.len() == 1 && MANGLE_NAME_NORMAL_REG.is_match(name.as_str()))
        || (deterministic
          && name.len() == 2
          && MANGLE_NAME_DETERMINISTIC_REG.is_match(name.as_str()))
        || (avoid_mangle_non_provided
          && !matches!(export_info.provided, Some(ExportInfoProvided::True)));

      let export_info_mut = export_info_id.get_export_info_mut(mg);
      if can_not_mangle {
        export_info_mut.set_used_name(name.clone());
        used_names.insert(name.to_string());
      } else {
        mangleable_exports.push(export_info_mut.id);
      };
    }

    // we need to re get export info to avoid extending immutable borrow lifetime
    let export_info = export_info_id.get_export_info(mg);
    if export_info.exports_info_owned {
      let used = export_info.get_used(None);
      if used == UsageState::OnlyPropertiesUsed || used == UsageState::Unused {
        mangle_exports_info(
          mg,
          deterministic,
          export_info
            .exports_info
            .expect("should have exports info id"),
          false,
        );
      }
    }
  }

  if deterministic {
    let used_names_len = used_names.len();
    let mut export_info_id_used_name = FxHashMap::default();
    assign_deterministic_ids(
      mangleable_exports,
      |e| {
        let export_info = e.get_export_info(mg);
        export_info
          .name
          .as_ref()
          .expect("should have name")
          .to_string()
      },
      |a, b| {
        let a_info = a.get_export_info(mg);
        let b_info = b.get_export_info(mg);
        compare_strings_numeric(a_info, b_info)
      },
      |e, id| {
        let name = number_to_identifier(id as u32);
        let size = used_names.len();
        used_names.insert(name.clone());
        if size == used_names.len() {
          false
        } else {
          export_info_id_used_name.insert(e, name);
          true
        }
      },
      &[
        NUMBER_OF_IDENTIFIER_START_CHARS as usize,
        (NUMBER_OF_IDENTIFIER_START_CHARS * NUMBER_OF_IDENTIFIER_CONTINUATION_CHARS) as usize,
      ],
      NUMBER_OF_IDENTIFIER_CONTINUATION_CHARS as usize,
      used_names_len,
      0,
    );
    for (export_info_id, name) in export_info_id_used_name {
      export_info_id
        .get_export_info_mut(mg)
        .set_used_name(name.into());
    }
  } else {
    let mut used_exports = Vec::new();
    let mut unused_exports = Vec::new();

    for export_info in mangleable_exports {
      if export_info.get_used(mg, None) == UsageState::Unused {
        unused_exports.push(export_info);
      } else {
        used_exports.push(export_info);
      }
    }

    used_exports.sort_by(|a, b| {
      let export_info_a = a.get_export_info(mg);
      let export_info_b = b.get_export_info(mg);
      compare_strings_numeric(export_info_a, export_info_b)
    });
    unused_exports.sort_by(|a, b| {
      let export_info_a = a.get_export_info(mg);
      let export_info_b = b.get_export_info(mg);
      compare_strings_numeric(export_info_a, export_info_b)
    });

    let mut i = 0;
    for list in [used_exports, unused_exports] {
      for export_info_id in list {
        let mut name;
        loop {
          name = number_to_identifier(i);
          if !used_names.contains(&name) {
            break;
          }
          i += 1;
        }
        export_info_id.set_used_name(mg, name.into());
      }
    }
  }
}
