use std::sync::LazyLock;

use regex::Regex;
use rspack_core::{
  ApplyContext, BuildMetaExportsType, Compilation, CompilationOptimizeCodeGeneration,
  CompilerOptions, ExportInfoProvided, ExportsInfo, ModuleGraph, Plugin, PluginContext, UsageState,
};
use rspack_error::Result;
use rspack_hook::{plugin, plugin_hook};
use rspack_ids::id_helpers::assign_deterministic_ids;
use rspack_util::atom::Atom;
use rustc_hash::{FxHashMap, FxHashSet};

use crate::utils::mangle_exports::{
  number_to_identifier, NUMBER_OF_IDENTIFIER_CONTINUATION_CHARS, NUMBER_OF_IDENTIFIER_START_CHARS,
};

fn can_mangle(exports_info: ExportsInfo, mg: &ModuleGraph) -> bool {
  if exports_info.other_exports_info(mg).get_used(mg, None) != UsageState::Unused {
    return false;
  }
  let mut has_something_to_mangle = false;
  for export_info in exports_info.exports(mg) {
    if export_info.can_mangle(mg) == Some(true) {
      has_something_to_mangle = true;
    }
  }
  has_something_to_mangle
}

/// Struct to represent the mangle exports plugin.
#[plugin]
#[derive(Debug)]
pub struct MangleExportsPlugin {
  deterministic: bool,
}

impl MangleExportsPlugin {
  pub fn new(deterministic: bool) -> Self {
    Self::new_inner(deterministic)
  }
}

#[plugin_hook(CompilationOptimizeCodeGeneration for MangleExportsPlugin)]
fn optimize_code_generation(&self, compilation: &mut Compilation) -> Result<()> {
  // TODO: should bailout if compilation.moduleMemCache is enable, https://github.com/webpack/webpack/blob/1f99ad6367f2b8a6ef17cce0e058f7a67fb7db18/lib/optimize/MangleExportsPlugin.js#L160-L164
  // We don't do that cause we don't have this option
  let mut mg = compilation.get_module_graph_mut();
  let module_id_list = mg.modules().keys().cloned().collect::<Vec<_>>();
  for identifier in module_id_list {
    let (Some(mgm), Some(module)) = (
      mg.module_graph_module_by_identifier(&identifier),
      mg.module_by_identifier(&identifier),
    ) else {
      continue;
    };
    let is_namespace = matches!(
      module.build_meta().exports_type,
      BuildMetaExportsType::Namespace
    );
    let exports_info = mgm.exports;
    mangle_exports_info(&mut mg, self.deterministic, exports_info, is_namespace);
  }
  Ok(())
}

impl Plugin for MangleExportsPlugin {
  fn apply(&self, ctx: PluginContext<&mut ApplyContext>, _options: &CompilerOptions) -> Result<()> {
    ctx
      .context
      .compilation_hooks
      .optimize_code_generation
      .tap(optimize_code_generation::new(self));
    Ok(())
  }
}

/// Compare function for sorting exports by name.
fn compare_strings_numeric(a: Option<&Atom>, b: Option<&Atom>) -> std::cmp::Ordering {
  a.cmp(&b)
}
static MANGLE_NAME_NORMAL_REG: LazyLock<Regex> =
  LazyLock::new(|| Regex::new("^[a-zA-Z0-9_$]").expect("should construct regex"));
static MANGLE_NAME_DETERMINISTIC_REG: LazyLock<Regex> = LazyLock::new(|| {
  Regex::new("^[a-zA-Z_$][a-zA-Z0-9_$]|^[1-9][0-9]").expect("should construct regex")
});

/// Function to mangle exports information.
fn mangle_exports_info(
  mg: &mut ModuleGraph,
  deterministic: bool,
  exports_info: ExportsInfo,
  is_namespace: bool,
) {
  if !can_mangle(exports_info, mg) {
    return;
  }

  let mut used_names = FxHashSet::default();
  let mut mangleable_exports = Vec::new();
  let mut avoid_mangle_non_provided = !is_namespace;

  if !avoid_mangle_non_provided && deterministic {
    for export_info in exports_info.owned_exports(mg) {
      if !matches!(export_info.provided(mg), Some(ExportInfoProvided::False)) {
        avoid_mangle_non_provided = true;
        break;
      }
    }
  }

  for export_info in exports_info.owned_exports(mg).collect::<Vec<_>>() {
    if !export_info.has_used_name(mg) {
      let name = export_info
        .name(mg)
        .expect("the name of export_info inserted in exports_info can not be `None`")
        .clone();
      let can_not_mangle = export_info.can_mangle(mg) != Some(true)
        || (name.len() == 1 && MANGLE_NAME_NORMAL_REG.is_match(name.as_str()))
        || (deterministic
          && name.len() == 2
          && MANGLE_NAME_DETERMINISTIC_REG.is_match(name.as_str()))
        || (avoid_mangle_non_provided
          && !matches!(export_info.provided(mg), Some(ExportInfoProvided::True)));

      if can_not_mangle {
        export_info.set_used_name(mg, name.clone());
        used_names.insert(name.to_string());
      } else {
        mangleable_exports.push(export_info);
      };
    }

    // we need to re get export info to avoid extending immutable borrow lifetime
    if export_info.exports_info_owned(mg) {
      let used = export_info.get_used(mg, None);
      if used == UsageState::OnlyPropertiesUsed || used == UsageState::Unused {
        mangle_exports_info(
          mg,
          deterministic,
          export_info
            .exports_info(mg)
            .expect("should have exports info id"),
          false,
        );
      }
    }
  }

  if deterministic {
    let used_names_len = used_names.len();
    let mut export_info_used_name = FxHashMap::default();
    assign_deterministic_ids(
      mangleable_exports,
      |e| e.name(mg).expect("should have name").to_string(),
      |a, b| compare_strings_numeric(a.name(mg), b.name(mg)),
      |e, id| {
        let name = number_to_identifier(id as u32);
        let size = used_names.len();
        used_names.insert(name.clone());
        if size == used_names.len() {
          false
        } else {
          export_info_used_name.insert(e, name);
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
    for (export_info, name) in export_info_used_name {
      export_info.set_used_name(mg, name.into());
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

    used_exports.sort_by(|a, b| compare_strings_numeric(a.name(mg), b.name(mg)));
    unused_exports.sort_by(|a, b| compare_strings_numeric(a.name(mg), b.name(mg)));

    let mut i = 0;
    for list in [used_exports, unused_exports] {
      for export_info in list {
        let mut name;
        loop {
          name = number_to_identifier(i);
          if !used_names.contains(&name) {
            break;
          }
          i += 1;
        }
        export_info.set_used_name(mg, name.into());
      }
    }
  }
}
