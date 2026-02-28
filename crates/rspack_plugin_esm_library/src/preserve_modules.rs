use std::{borrow::Cow, path::Path, sync::LazyLock};

use regex::Regex;
use rspack_collections::{IdentifierMap, IdentifierSet};
use rspack_core::Compilation;
use rspack_util::fx_hash::{FxHashMap, FxHashSet};
use sugar_path::SugarPath;

use crate::EsmLibraryPlugin;

pub fn entry_modules(compilation: &Compilation) -> FxHashMap<String, IdentifierSet> {
  let module_graph = compilation.get_module_graph();
  compilation
    .entries
    .iter()
    .map(|(name, item)| {
      (
        name.clone(),
        item
          .all_dependencies()
          .chain(compilation.global_entry.all_dependencies())
          .filter_map(|dep_id| {
            // some entry dependencies may not find module because of resolve failed
            // so use filter_map to ignore them
            module_graph
              .module_identifier_by_dependency_id(dep_id)
              .copied()
          })
          .collect::<IdentifierSet>(),
      )
    })
    .collect()
}

pub fn entry_name_for_module(
  entry_modules: &FxHashMap<String, IdentifierSet>,
) -> IdentifierMap<FxHashSet<String>> {
  let mut entry_name_for_module: IdentifierMap<FxHashSet<String>> = IdentifierMap::default();
  for (entry_name, modules) in entry_modules {
    for module_id in modules {
      entry_name_for_module
        .entry(*module_id)
        .or_default()
        .insert(entry_name.clone());
    }
  }
  entry_name_for_module
}

pub async fn preserve_modules(
  root: &Path,
  compilation: &mut Compilation,
) -> Vec<rspack_error::Diagnostic> {
  let mut errors = vec![];
  let modules = compilation
    .get_module_graph()
    .modules_keys()
    .copied()
    .collect::<Vec<_>>();

  let entry_modules = entry_modules(compilation);
  let entry_name_for_module = entry_name_for_module(&entry_modules);

  for module_id in modules {
    if compilation
      .build_chunk_graph_artifact
      .chunk_graph
      .get_module_chunks(module_id)
      .is_empty()
    {
      // ignore orphan
      continue;
    }

    let module_graph = compilation.get_module_graph();
    let Some(normal_module) = module_graph
      .module_by_identifier(&module_id)
      .expect("should have module")
      .as_normal_module()
    else {
      continue;
    };

    let Some(abs_path) = normal_module
      .resource_resolved_data()
      .path()
      .map(|p| p.as_std_path())
      .map(|p| p.to_path_buf())
    else {
      continue;
    };
    let chunk = EsmLibraryPlugin::get_module_chunk(module_id, compilation);
    let old_chunk = compilation
      .build_chunk_graph_artifact
      .chunk_by_ukey
      .expect_get_mut(&chunk);

    if abs_path.starts_with(root) {
      // split module into single chunk named root
      let file_path = abs_path.relative(root);
      let extension = file_path.extension();

      let new_extension = old_chunk
        .filename_template()
        .unwrap_or(&compilation.options.output.filename)
        .template()
        .map_or(Cow::Borrowed(".js"), |tpl| {
          static EXTENSION_JS: LazyLock<Regex> =
            LazyLock::new(|| Regex::new(r".+(\..+)$").expect("failed to compile EXTENSION_REGEXP"));

          if let Some(captures) = EXTENSION_JS.captures(tpl) {
            Cow::Owned(captures[1].to_string())
          } else {
            Cow::Borrowed(".js")
          }
        });
      let new_filename = if let Some(extension) = extension {
        let ext_lossy = extension.to_string_lossy();
        let file_path_lossy = file_path.to_string_lossy();
        let suffix = format!(".{ext_lossy}");
        let base = file_path_lossy
          .strip_suffix(&suffix)
          .unwrap_or(&file_path_lossy);
        format!("{base}{new_extension}").into()
      } else {
        file_path.to_string_lossy().to_string().into()
      };

      if compilation
        .build_chunk_graph_artifact
        .chunk_graph
        .get_chunk_modules_identifier(&chunk)
        .len()
        == 1
      {
        // this is last module in chunk, we can keep this chunk, just rename it
        old_chunk.set_filename_template(Some(new_filename));
        continue;
      }

      let new_chunk_ukey =
        Compilation::add_chunk(&mut compilation.build_chunk_graph_artifact.chunk_by_ukey);
      compilation
        .build_chunk_graph_artifact
        .chunk_graph
        .add_chunk(new_chunk_ukey);
      let [Some(new_chunk), Some(old_chunk)] = compilation
        .build_chunk_graph_artifact
        .chunk_by_ukey
        .get_many_mut([&new_chunk_ukey, &chunk])
      else {
        unreachable!("new_chunk and old_chunk should be inserted already")
      };

      new_chunk.set_filename_template(Some(new_filename));
      old_chunk.split(
        new_chunk,
        &mut compilation.build_chunk_graph_artifact.chunk_group_by_ukey,
      );
      // disconnect module from other chunks
      compilation
        .build_chunk_graph_artifact
        .chunk_graph
        .disconnect_chunk_and_module(&chunk, module_id);

      compilation
        .build_chunk_graph_artifact
        .chunk_graph
        .connect_chunk_and_module(new_chunk_ukey, module_id);

      if let Some(entry_names) = entry_name_for_module.get(&module_id) {
        if entry_names.len() > 1 {
          errors.push(
            rspack_error::error!(
              "{} is used in multiple entries: [{}], this is not allowed in preserveModules",
              module_id,
              entry_names
                .iter()
                .map(|s| s.as_str())
                .collect::<Vec<_>>()
                .join(", ")
            )
            .into(),
          );
        }

        compilation
          .build_chunk_graph_artifact
          .chunk_graph
          .disconnect_chunk_and_entry_module(&chunk, module_id);

        let entrypoint = compilation.entrypoint_by_name_mut(
          entry_names
            .iter()
            .next()
            .expect("entry_names should not be empty"),
        );
        let ukey = entrypoint.ukey;
        entrypoint.set_entrypoint_chunk(new_chunk_ukey);

        compilation
          .build_chunk_graph_artifact
          .chunk_graph
          .connect_chunk_and_entry_module(new_chunk_ukey, module_id, ukey);
      }
    }
  }

  errors
}
