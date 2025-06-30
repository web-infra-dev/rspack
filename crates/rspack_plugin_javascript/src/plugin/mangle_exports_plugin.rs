use std::sync::LazyLock;

use itertools::Itertools;
use rayon::prelude::*;
use regex::Regex;
use rspack_core::{
  incremental::IncrementalPasses, ApplyContext, BuildMetaExportsType, Compilation,
  CompilationOptimizeCodeGeneration, CompilerOptions, ExportInfo, ExportInfoGetter, ExportProvided,
  ExportsInfo, ExportsInfoGetter, ModuleGraph, Plugin, PluginContext, PrefetchExportsInfoMode,
  PrefetchedExportsInfoWrapper, UsageState,
};
use rspack_error::Result;
use rspack_hook::{plugin, plugin_hook};
use rspack_ids::id_helpers::assign_deterministic_ids;
use rspack_util::atom::Atom;
use rustc_hash::{FxHashMap, FxHashSet};

use crate::utils::mangle_exports::{
  number_to_identifier, NUMBER_OF_IDENTIFIER_CONTINUATION_CHARS, NUMBER_OF_IDENTIFIER_START_CHARS,
};

fn can_mangle(exports_info: &PrefetchedExportsInfoWrapper<'_>) -> bool {
  if ExportInfoGetter::get_used(exports_info.other_exports_info(), None) != UsageState::Unused {
    return false;
  }
  let mut has_something_to_mangle = false;
  for (_, export_info) in exports_info.exports() {
    if ExportInfoGetter::can_mangle(export_info) == Some(true) {
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

#[derive(Debug)]
enum Manglable {
  CanNotMangle(Atom),
  CanMangle(Atom),
  Mangled,
}

#[derive(Debug)]
struct ExportInfoCache {
  id: ExportInfo,
  deterministic: bool,
  exports_info: Option<ExportsInfo>,
  can_mangle: Manglable,
}

#[plugin_hook(CompilationOptimizeCodeGeneration for MangleExportsPlugin)]
async fn optimize_code_generation(&self, compilation: &mut Compilation) -> Result<()> {
  if let Some(diagnostic) = compilation.incremental.disable_passes(
    IncrementalPasses::MODULES_HASHES,
    "MangleExportsPlugin (optimization.mangleExports = true)",
    "it requires calculating the export names of all the modules, which is a global effect",
  ) {
    if let Some(diagnostic) = diagnostic {
      compilation.push_diagnostic(diagnostic);
    }
    compilation.cgm_hash_artifact.clear();
  }

  // TODO: should bailout if compilation.moduleMemCache is enable, https://github.com/webpack/webpack/blob/1f99ad6367f2b8a6ef17cce0e058f7a67fb7db18/lib/optimize/MangleExportsPlugin.js#L160-L164
  // We don't do that cause we don't have this option
  let mut mg = compilation.get_module_graph_mut();
  let module_id_list = mg.modules().keys().cloned().collect::<Vec<_>>();

  let mut exports_info_cache = FxHashMap::default();

  let mut q = module_id_list
    .iter()
    .filter_map(|id| {
      let (Some(mgm), Some(module)) = (
        mg.module_graph_module_by_identifier(id),
        mg.module_by_identifier(id),
      ) else {
        return None;
      };
      let is_namespace = matches!(
        module.build_meta().exports_type,
        BuildMetaExportsType::Namespace
      );
      Some((mgm.exports, is_namespace))
    })
    .collect_vec();

  while !q.is_empty() {
    let items = std::mem::take(&mut q);
    let batch = items
      .par_iter()
      .filter_map(|(exports_info, is_namespace)| {
        let mut avoid_mangle_non_provided = !is_namespace;
        let deterministic = self.deterministic;
        let exports_info_data =
          ExportsInfoGetter::prefetch(exports_info, &mg, PrefetchExportsInfoMode::AllExports);
        let export_list = {
          if !can_mangle(&exports_info_data) {
            return None;
          }
          if !avoid_mangle_non_provided && deterministic {
            for (_, export_info) in exports_info_data.exports() {
              if !matches!(export_info.provided(), Some(ExportProvided::NotProvided)) {
                avoid_mangle_non_provided = true;
                break;
              }
            }
          }
          exports_info_data
            .exports()
            .map(|(_, export_info)| export_info)
            .collect::<Vec<_>>()
        };

        Some((
          *exports_info,
          export_list
            .iter()
            .map(|export_info_data| {
              let can_mangle = if !ExportInfoGetter::has_used_name(export_info_data) {
                let name = export_info_data
                  .name()
                  .expect("the name of export_info inserted in exports_info can not be `None`")
                  .clone();
                let can_not_mangle = ExportInfoGetter::can_mangle(export_info_data) != Some(true)
                  || (name.len() == 1 && MANGLE_NAME_NORMAL_REG.is_match(name.as_str()))
                  || (deterministic
                    && name.len() == 2
                    && MANGLE_NAME_DETERMINISTIC_REG.is_match(name.as_str()))
                  || (avoid_mangle_non_provided
                    && !matches!(export_info_data.provided(), Some(ExportProvided::Provided)));

                if can_not_mangle {
                  Manglable::CanNotMangle(name)
                } else {
                  Manglable::CanMangle(name)
                }
              } else {
                Manglable::Mangled
              };

              let nested_exports_info = if export_info_data.exports_info_owned() {
                let used = ExportInfoGetter::get_used(export_info_data, None);
                if used == UsageState::OnlyPropertiesUsed || used == UsageState::Unused {
                  export_info_data.exports_info()
                } else {
                  None
                }
              } else {
                None
              };

              ExportInfoCache {
                id: export_info_data.id(),
                deterministic,
                exports_info: nested_exports_info,
                can_mangle,
              }
            })
            .collect_vec(),
        ))
      })
      .collect::<Vec<_>>();

    for (exports_info, export_list) in batch {
      q.extend(
        export_list
          .iter()
          .filter_map(|export_info_cache| export_info_cache.exports_info.map(|e| (e, false)))
          .filter(|(e, _)| !exports_info_cache.contains_key(e)),
      );
      exports_info_cache.insert(exports_info, export_list);
    }
  }

  for identifier in module_id_list {
    let (Some(mgm), Some(_)) = (
      mg.module_graph_module_by_identifier(&identifier),
      mg.module_by_identifier(&identifier),
    ) else {
      continue;
    };
    let exports_info = mgm.exports;
    mangle_exports_info(
      &mut mg,
      self.deterministic,
      exports_info,
      &exports_info_cache,
    );
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
  exports_info_cache: &FxHashMap<ExportsInfo, Vec<ExportInfoCache>>,
) {
  let mut used_names = FxHashSet::default();
  let mut mangleable_exports = Vec::new();
  let Some(export_list) = exports_info_cache.get(&exports_info) else {
    return;
  };

  let mut mangleable_export_names = FxHashMap::default();

  for export_info in export_list {
    match &export_info.can_mangle {
      Manglable::CanNotMangle(name) => {
        export_info.id.as_data_mut(mg).set_used_name(name.clone());
        used_names.insert(name.to_string());
      }
      Manglable::CanMangle(name) => {
        mangleable_export_names.insert(export_info.id.clone(), name.clone());
        mangleable_exports.push(export_info.id.clone());
      }
      Manglable::Mangled => {}
    }

    if let Some(nested_exports_info) = export_info.exports_info {
      mangle_exports_info(
        mg,
        export_info.deterministic,
        nested_exports_info,
        exports_info_cache,
      );
    }
  }

  if deterministic {
    let used_names_len = used_names.len();
    let mut export_info_used_name = FxHashMap::default();
    assign_deterministic_ids(
      mangleable_exports,
      |e| {
        mangleable_export_names
          .get(&e)
          .expect("should have name")
          .to_string()
      },
      |a, b| {
        compare_strings_numeric(
          Some(mangleable_export_names.get(a).expect("should have name")),
          Some(mangleable_export_names.get(b).expect("should have name")),
        )
      },
      |e, id| {
        let name = number_to_identifier(id as u32);
        let size = used_names.len();
        used_names.insert(name.clone());
        if size == used_names.len() {
          false
        } else {
          export_info_used_name.insert(e.clone(), name);
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
      export_info.as_data_mut(mg).set_used_name(name.into());
    }
  } else {
    let mut used_exports = Vec::new();
    let mut unused_exports = Vec::new();

    for export_info in mangleable_exports {
      let used = ExportInfoGetter::get_used(export_info.as_data(mg), None);
      if used == UsageState::Unused {
        unused_exports.push(export_info);
      } else {
        used_exports.push(export_info);
      }
    }

    used_exports.sort_by(|a, b| {
      compare_strings_numeric(
        Some(mangleable_export_names.get(a).expect("should have name")),
        Some(mangleable_export_names.get(b).expect("should have name")),
      )
    });
    unused_exports.sort_by(|a, b| {
      compare_strings_numeric(
        Some(mangleable_export_names.get(a).expect("should have name")),
        Some(mangleable_export_names.get(b).expect("should have name")),
      )
    });

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
        export_info.as_data_mut(mg).set_used_name(name.into());
      }
    }
  }
}
