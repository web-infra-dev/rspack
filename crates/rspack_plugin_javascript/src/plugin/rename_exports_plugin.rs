use std::sync::LazyLock;

use itertools::Itertools;
use rayon::prelude::*;
use regex::Regex;
use rspack_core::{
  BuildMetaExportsType, Compilation, CompilationOptimizeCodeGeneration, EvaluatedInlinableValue,
  ExportInfo, ExportProvided, ExportsInfo, ExportsInfoGetter, MangleExportsOptions, ModuleGraph,
  Plugin, PrefetchExportsInfoMode, PrefetchedExportsInfoWrapper, UsageState, UsedNameItem,
  incremental::IncrementalPasses,
};
use rspack_error::Result;
use rspack_hook::{plugin, plugin_hook};
use rspack_ids::id_helpers::assign_deterministic_ids;
use rspack_util::atom::Atom;
use rustc_hash::{FxHashMap, FxHashSet};

use crate::utils::mangle_exports::{
  NUMBER_OF_IDENTIFIER_CONTINUATION_CHARS, NUMBER_OF_IDENTIFIER_START_CHARS, number_to_identifier,
};

fn can_rename(
  exports_info: &PrefetchedExportsInfoWrapper<'_>,
  inline_exports: bool,
  mangle_exports: bool,
) -> bool {
  if exports_info.other_exports_info().get_used(None) != UsageState::Unused {
    return false;
  }
  let mut has_something_to_rename = false;
  for (_, export_info) in exports_info.exports() {
    if inline_exports && export_info.can_inline() == Some(true) {
      has_something_to_rename = true;
      break;
    }
    if mangle_exports && export_info.can_mangle() == Some(true) {
      has_something_to_rename = true;
      break;
    }
  }
  has_something_to_rename
}

#[derive(Debug)]
pub struct RenameExportsPluginOptions {
  pub inline_exports: bool,
  pub mangle_exports: MangleExportsOptions,
}

/// Struct to represent the mangle exports plugin.
#[plugin]
#[derive(Debug)]
pub struct RenameExportsPlugin {
  options: RenameExportsPluginOptions,
}

impl RenameExportsPlugin {
  pub fn new(options: RenameExportsPluginOptions) -> Self {
    Self::new_inner(options)
  }
}

#[derive(Debug)]
enum Renameable {
  CanNotRename(Atom),
  CanInline(EvaluatedInlinableValue),
  CanMangle(Atom),
  Renamed,
}

#[derive(Debug)]
struct ExportInfoCache {
  id: ExportInfo,
  exports_info: Option<ExportsInfo>,
  can_rename: Renameable,
}

#[plugin_hook(CompilationOptimizeCodeGeneration for RenameExportsPlugin)]
async fn optimize_code_generation(&self, compilation: &mut Compilation) -> Result<()> {
  if let Some(diagnostic) = compilation.incremental.disable_passes(
    IncrementalPasses::MODULES_HASHES,
    "RenameExportsPlugin (optimization.mangleExports = true)",
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
  let modules = mg.modules();
  let deterministic = matches!(self.options.mangle_exports, MangleExportsOptions::Enabled { deterministic } if deterministic);

  let mut exports_info_cache = FxHashMap::default();

  let mut q = modules
    .iter()
    .filter_map(|(mid, module)| {
      let mgm = mg.module_graph_module_by_identifier(mid)?;
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
        let exports_info_data =
          ExportsInfoGetter::prefetch(exports_info, &mg, PrefetchExportsInfoMode::Default);
        let export_list = {
          if !can_rename(
            &exports_info_data,
            self.options.inline_exports,
            matches!(
              self.options.mangle_exports,
              MangleExportsOptions::Enabled { .. }
            ),
          ) {
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
            .map(|(_, export_info_data)| {
              let can_rename = if !export_info_data.has_used_name() {
                if self.options.inline_exports && export_info_data.can_inline() == Some(true) {
                  let inlined = export_info_data
                    .can_inline_provide()
                    .expect("should provide inlined value");
                  Renameable::CanInline(inlined.clone())
                } else {
                  let name = export_info_data
                    .name()
                    .expect("the name of export_info inserted in exports_info can not be `None`")
                    .clone();
                  let can_not_mangle = export_info_data.can_mangle() != Some(true)
                    || (name.len() == 1 && MANGLE_NAME_NORMAL_REG.is_match(name.as_str()))
                    || (deterministic
                      && name.len() == 2
                      && MANGLE_NAME_DETERMINISTIC_REG.is_match(name.as_str()))
                    || (avoid_mangle_non_provided
                      && !matches!(export_info_data.provided(), Some(ExportProvided::Provided)));

                  if can_not_mangle {
                    Renameable::CanNotRename(name)
                  } else {
                    Renameable::CanMangle(name)
                  }
                }
              } else {
                Renameable::Renamed
              };

              let nested_exports_info = if export_info_data.exports_info_owned() {
                let used = export_info_data.get_used(None);
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
                exports_info: nested_exports_info,
                can_rename,
              }
            })
            .collect_vec()
        };

        Some((*exports_info, export_list))
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

  let mut queue = modules
    .into_iter()
    .filter_map(|(mid, _)| {
      let mgm = mg.module_graph_module_by_identifier(&mid)?;
      Some(mgm.exports)
    })
    .collect_vec();

  while !queue.is_empty() {
    let tasks = std::mem::take(&mut queue);
    let batch = tasks
      .into_par_iter()
      .map(|exports_info| {
        rename_exports_info(&mg, deterministic, exports_info, &exports_info_cache)
      })
      .collect::<Vec<_>>();

    let mut used_name_tasks = vec![];
    for (changes, nested_exports) in batch {
      used_name_tasks.extend(changes);
      queue.extend(nested_exports);
    }

    mg.batch_set_export_info_used_name(used_name_tasks);
  }

  Ok(())
}

impl Plugin for RenameExportsPlugin {
  fn apply(&self, ctx: &mut rspack_core::ApplyContext<'_>) -> Result<()> {
    if !matches!(
      self.options.mangle_exports,
      MangleExportsOptions::Enabled { .. }
    ) && !self.options.inline_exports
    {
      return Ok(());
    }
    ctx
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
fn rename_exports_info(
  mg: &ModuleGraph,
  deterministic: bool,
  exports_info: ExportsInfo,
  exports_info_cache: &FxHashMap<ExportsInfo, Vec<ExportInfoCache>>,
) -> (Vec<(ExportInfo, UsedNameItem)>, Vec<ExportsInfo>) {
  let mut changes = vec![];
  let mut nested_exports = vec![];
  let mut used_names = FxHashSet::default();
  let mut mangleable_exports = Vec::new();
  let Some(export_list) = exports_info_cache.get(&exports_info) else {
    return (changes, nested_exports);
  };

  let mut mangleable_export_names = FxHashMap::default();

  for export_info in export_list {
    match &export_info.can_rename {
      Renameable::CanNotRename(name) => {
        changes.push((export_info.id.clone(), UsedNameItem::Str(name.clone())));
        used_names.insert(name.to_string());
      }
      Renameable::CanInline(inlined) => {
        changes.push((
          export_info.id.clone(),
          UsedNameItem::Inlined(inlined.clone()),
        ));
      }
      Renameable::CanMangle(name) => {
        mangleable_export_names.insert(export_info.id.clone(), name.clone());
        mangleable_exports.push(export_info.id.clone());
      }
      Renameable::Renamed => {}
    }

    if let Some(nested_exports_info) = export_info.exports_info {
      nested_exports.push(nested_exports_info);
    }
  }

  if deterministic {
    let used_names_len = used_names.len();
    let mut export_info_used_name = FxHashMap::default();
    assign_deterministic_ids(
      mangleable_exports,
      |e| {
        mangleable_export_names
          .get(e)
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
      changes.push((export_info, UsedNameItem::Str(name.into())));
    }
  } else {
    let mut used_exports = Vec::new();
    let mut unused_exports = Vec::new();

    for export_info in mangleable_exports {
      let used = export_info.as_data(mg).get_used(None);
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
        changes.push((export_info, UsedNameItem::Str(name.into())));
      }
    }
  }
  (changes, nested_exports)
}
