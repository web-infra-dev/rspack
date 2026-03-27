use std::sync::Arc;

use atomic_refcell::AtomicRefCell;
use rayon::prelude::*;
use rspack_collections::{IdentifierMap, IdentifierSet};
use rspack_core::{
  ChunkGroupUkey, ChunkUkey, Compilation, DependenciesBlock, DependencyType, ExportProvided,
  ModuleIdentifier, UsageState, find_new_name, get_cached_readable_identifier,
  incremental::Mutation, split_readable_identifier,
};
use rspack_util::{
  atom::Atom,
  fx_hash::{FxDashSet, FxHashMap, FxHashSet},
};

use crate::EsmLibraryPlugin;

/// Ensure that all entry chunks only export the exports used by other chunks,
/// this requires no other chunks depend on the entry chunk to get exports
///
/// for example entryA -> a -> b => c -> a
/// entry chunk: a, b
/// async chunk: c
/// c depends on a, so entry chunk needs to re-export symbols from a
pub(crate) fn ensure_entry_exports(compilation: &mut Compilation) {
  let module_graph = compilation.get_module_graph();
  let mut entrypoint_chunks = FxHashMap::<ChunkUkey, ChunkGroupUkey>::default();
  let mut entry_module_belongs: IdentifierMap<FxHashSet<ChunkUkey>> = IdentifierMap::default();

  for entrypoint_ukey in compilation.entrypoints().values() {
    let entrypoint = compilation
      .build_chunk_graph_artifact
      .chunk_group_by_ukey
      .expect_get(entrypoint_ukey);
    let entrypoint_chunk = entrypoint.get_entrypoint_chunk();

    // we should use get_chunk_modules instead of entrydata.modules
    // because entry modules may be moved to another chunk
    // we only care if the real entry chunk is a dependency of other chunks
    let entry_modules = compilation
      .build_chunk_graph_artifact
      .chunk_graph
      .get_chunk_modules_identifier(&entrypoint_chunk);

    entrypoint_chunks.insert(entrypoint_chunk, *entrypoint_ukey);

    for m in entry_modules {
      entry_module_belongs
        .entry(*m)
        .or_default()
        .insert(entrypoint_chunk);
    }
  }

  let dirty_chunks = FxDashSet::default();

  compilation
    .build_chunk_graph_artifact
    .chunk_by_ukey
    .iter()
    .par_bridge()
    .filter(|(ukey, _)| !entrypoint_chunks.contains_key(ukey))
    .for_each(|(ukey, _)| {
      let modules = compilation
        .build_chunk_graph_artifact
        .chunk_graph
        .get_chunk_modules_identifier(ukey);

      for m in modules {
        // check if module has used entry chunk exports
        let outgoings = module_graph.get_active_outcoming_connections_by_module(
          m,
          None,
          module_graph,
          &compilation.module_graph_cache_artifact,
          &compilation
            .build_module_graph_artifact
            .side_effects_state_artifact
            .read()
            .expect("should lock side effects state artifact"),
          &compilation.exports_info_artifact,
        );
        for outgoing in outgoings.keys() {
          if let Some(entry_chunks) = entry_module_belongs.get(outgoing) {
            for entry_chunk in entry_chunks {
              dirty_chunks.insert(*entry_chunk);
            }
            break;
          }
        }
      }
    });

  // the dirty chunks need to re-exports the needed exports,
  // so move all modules inside it into another chunk.
  // And we should split a runtime chunk as well, to make sure the new chunk depends on runtime
  // will not cause circular dependency
  for entry_chunk_ukey in dirty_chunks {
    let entry_modules = compilation
      .build_chunk_graph_artifact
      .chunk_graph
      .get_chunk_modules_identifier(&entry_chunk_ukey)
      .iter()
      .copied()
      .collect::<Vec<_>>();

    let new_chunk_ukey =
      Compilation::add_chunk(&mut compilation.build_chunk_graph_artifact.chunk_by_ukey);
    if let Some(mut mutation) = compilation.incremental.mutations_write() {
      mutation.add(Mutation::ChunkAdd {
        chunk: new_chunk_ukey,
      });
    }
    compilation
      .build_chunk_graph_artifact
      .chunk_graph
      .add_chunk(new_chunk_ukey);

    // move entrypoint runtime as well
    let entrypoint_ukey = entrypoint_chunks[&entry_chunk_ukey];
    let entrypoint = compilation
      .build_chunk_graph_artifact
      .chunk_group_by_ukey
      .expect_get(&entrypoint_ukey);
    if entrypoint.get_runtime_chunk(&compilation.build_chunk_graph_artifact.chunk_group_by_ukey)
      == entry_chunk_ukey
    {
      let entrypoint = compilation
        .build_chunk_graph_artifact
        .chunk_group_by_ukey
        .expect_get_mut(&entrypoint_ukey);
      entrypoint.set_runtime_chunk(new_chunk_ukey);
    }

    for m in entry_modules {
      compilation
        .build_chunk_graph_artifact
        .chunk_graph
        .disconnect_chunk_and_entry_module(&entry_chunk_ukey, m);
      compilation
        .build_chunk_graph_artifact
        .chunk_graph
        .disconnect_chunk_and_module(&entry_chunk_ukey, m);

      compilation
        .build_chunk_graph_artifact
        .chunk_graph
        .connect_chunk_and_module(new_chunk_ukey, m);

      let entrypoint = entrypoint_chunks[&entry_chunk_ukey];
      compilation
        .build_chunk_graph_artifact
        .chunk_graph
        .connect_chunk_and_entry_module(new_chunk_ukey, m, entrypoint);
    }

    let [Some(entry_chunk), Some(new_chunk)] = compilation
      .build_chunk_graph_artifact
      .chunk_by_ukey
      .get_many_mut([&entry_chunk_ukey, &new_chunk_ukey])
    else {
      unreachable!()
    };

    entry_chunk.split(
      new_chunk,
      &mut compilation.build_chunk_graph_artifact.chunk_group_by_ukey,
    );
  }
}

/// For each entrypoint, if the runtime chunk is the same as the entry chunk
/// and any initial ChunkGroup containing this chunk has multiple chunks,
/// split the runtime into a separate runtime chunk.
///
/// This must run AFTER SplitChunksPlugin and RemoveDuplicateModulesPlugin
/// to inspect the final chunk graph topology.
pub(crate) fn optimize_runtime_chunks(compilation: &mut Compilation) {
  // Phase 1: Collect entrypoints that need runtime splitting
  let entrypoints_to_split: Vec<ChunkGroupUkey> = compilation
    .entrypoints()
    .values()
    .copied()
    .filter(|entrypoint_ukey| {
      let entrypoint = compilation
        .build_chunk_graph_artifact
        .chunk_group_by_ukey
        .expect_get(entrypoint_ukey);

      let runtime_chunk_ukey =
        entrypoint.get_runtime_chunk(&compilation.build_chunk_graph_artifact.chunk_group_by_ukey);
      let entry_chunk_ukey = entrypoint.get_entrypoint_chunk();

      // Skip if runtime is already a separate chunk
      if runtime_chunk_ukey != entry_chunk_ukey {
        return false;
      }

      // Check if any initial ChunkGroup containing this chunk has multiple chunks
      let chunk = compilation
        .build_chunk_graph_artifact
        .chunk_by_ukey
        .expect_get(&runtime_chunk_ukey);

      chunk.groups().iter().any(|group_ukey| {
        let group = compilation
          .build_chunk_graph_artifact
          .chunk_group_by_ukey
          .expect_get(group_ukey);
        group.is_initial() && group.chunks.len() > 1
      })
    })
    .collect();

  // Phase 2: For each identified entrypoint, create a new runtime chunk
  for entrypoint_ukey in entrypoints_to_split {
    let entry_chunk_ukey = {
      let entrypoint = compilation
        .build_chunk_graph_artifact
        .chunk_group_by_ukey
        .expect_get(&entrypoint_ukey);
      entrypoint.get_entrypoint_chunk()
    };

    // Create a new chunk
    let new_chunk_ukey =
      Compilation::add_chunk(&mut compilation.build_chunk_graph_artifact.chunk_by_ukey);

    // Record mutation for incremental compilation
    if let Some(mut mutation) = compilation.incremental.mutations_write() {
      mutation.add(Mutation::ChunkAdd {
        chunk: new_chunk_ukey,
      });
    }

    // Register the chunk in the chunk graph
    compilation
      .build_chunk_graph_artifact
      .chunk_graph
      .add_chunk(new_chunk_ukey);

    // Set the entrypoint's runtime chunk to the new chunk
    let entrypoint = compilation
      .build_chunk_graph_artifact
      .chunk_group_by_ukey
      .expect_get_mut(&entrypoint_ukey);
    entrypoint.set_runtime_chunk(new_chunk_ukey);
    entrypoint.unshift_chunk(new_chunk_ukey);

    // Configure the new chunk
    let [Some(entry_chunk), Some(new_chunk)] = compilation
      .build_chunk_graph_artifact
      .chunk_by_ukey
      .get_many_mut([&entry_chunk_ukey, &new_chunk_ukey])
    else {
      unreachable!("entry_chunk and new_chunk should both exist")
    };

    new_chunk.set_runtime(entry_chunk.runtime().clone());
    new_chunk.add_id_name_hints("runtime".to_string());
    new_chunk.set_prevent_integration(true);
    new_chunk.add_group(entrypoint_ukey);
  }
}

/// Analyze dynamic import targets to identify:
/// - all_dyn_targets: all scope-hoisted modules that are dynamically imported
/// - namespace_targets: subset that are imported as namespace
/// - strict_chunks: single-module or entry chunks where exports are guaranteed correct
///
/// Also pre-assigns namespace object names in `dyn_import_ns_map` for scope-hoisted
/// modules in multi-module non-strict chunks. These names are used both by the
/// dynamic import template (during code generation) and the linker (after code generation).
pub(crate) fn analyze_dyn_import_targets(
  compilation: &Compilation,
  concatenated_modules: &IdentifierSet,
  dyn_import_ns_map: &Arc<AtomicRefCell<IdentifierMap<Atom>>>,
) -> (FxHashSet<ChunkUkey>, IdentifierSet, IdentifierSet) {
  let module_graph = compilation.get_module_graph();
  let mut all_dyn_targets = IdentifierSet::default();
  let mut namespace_targets = IdentifierSet::default();

  for (module_id, module) in module_graph.modules() {
    if !concatenated_modules.contains(module_id) {
      continue;
    }
    for dep_id in module
      .get_blocks()
      .iter()
      .filter_map(|block| module_graph.block_by_id(block))
      .flat_map(|block| block.get_dependencies())
    {
      let dep = module_graph.dependency_by_id(dep_id);
      if dep.dependency_type() != &DependencyType::DynamicImport {
        continue;
      }
      let exports_info_artifact = &compilation.exports_info_artifact;

      let Some(conn) = module_graph.connection_by_dependency_id(dep_id) else {
        continue;
      };
      if !conn.is_target_active(
        module_graph,
        None,
        &compilation.module_graph_cache_artifact,
        &compilation
          .build_module_graph_artifact
          .side_effects_state_artifact
          .read()
          .expect("should lock side effects state artifact"),
        exports_info_artifact,
      ) {
        continue;
      }
      let target = conn.module_identifier();
      // Skip orphan modules — they are not in any chunk (e.g. tree-shaken or worker entries)
      if compilation
        .build_chunk_graph_artifact
        .chunk_graph
        .get_module_chunks(*target)
        .is_empty()
      {
        continue;
      }
      all_dyn_targets.insert(*target);

      if !concatenated_modules.contains(target) {
        continue;
      }

      let exports_info = exports_info_artifact
        .get_exports_info(target)
        .as_data(exports_info_artifact);

      if exports_info.other_exports_info().is_used(None) {
        namespace_targets.insert(*target);
      }
    }
  }

  // Classify chunks: single-module or entry chunks get strict exports
  let mut strict_chunks = FxHashSet::default();

  let entrypoint_chunks: FxHashSet<ChunkUkey> = compilation
    .build_chunk_graph_artifact
    .entrypoints
    .values()
    .map(|entrypoint_ukey| {
      compilation
        .build_chunk_graph_artifact
        .chunk_group_by_ukey
        .expect_get(entrypoint_ukey)
        .get_entrypoint_chunk()
    })
    .collect();

  for module_id in &all_dyn_targets {
    let Some(module) = module_graph.module_by_identifier(module_id) else {
      continue;
    };
    if module.as_external_module().is_some() {
      continue;
    }

    let chunk_ukey = match EsmLibraryPlugin::get_module_chunk(*module_id, compilation) {
      Ok(c) => c,
      Err(e) => {
        tracing::warn!(error = %e, "failed to resolve module chunk during optimize_chunks");
        continue;
      }
    };
    let chunk_modules = compilation
      .build_chunk_graph_artifact
      .chunk_graph
      .get_chunk_modules_identifier(&chunk_ukey);

    // Count only non-external modules: external modules don't contribute code to the chunk,
    // so a chunk with 1 scope-hoisted module + N externals is effectively single-module.
    let non_external_count = chunk_modules
      .iter()
      .filter(|m| {
        module_graph
          .module_by_identifier(m)
          .is_some_and(|mod_| mod_.as_external_module().is_none())
      })
      .count();

    if non_external_count <= 1 || entrypoint_chunks.contains(&chunk_ukey) {
      // Single-module or entry chunk: exports are already correct.
      strict_chunks.insert(chunk_ukey);
    }
  }

  // Pre-assign namespace object names for scope-hoisted dyn targets in non-strict chunks.
  // Use the same naming scheme as regular namespace objects (find_new_name("namespaceObject", ...))
  // so the name matches what deconflict_symbols would produce.
  // These names must be determined before code generation so the dynamic import template
  // can emit `.then(m => m.<ns_name>)`.
  {
    let mut ns_map = dyn_import_ns_map.borrow_mut();
    let mut sorted_targets: Vec<_> = all_dyn_targets.iter().copied().collect();
    sorted_targets.sort();

    // Step 1: Collect export names per module per chunk (for non-strict, non-external,
    // concatenated modules) to detect export name conflicts between modules sharing a chunk.
    let exports_info_artifact = &compilation.exports_info_artifact;
    let mut chunk_module_exports: FxHashMap<ChunkUkey, Vec<(_, FxHashSet<Atom>)>> =
      FxHashMap::default();
    for module_id in &sorted_targets {
      if !concatenated_modules.contains(module_id) {
        continue;
      }
      let Some(module) = module_graph.module_by_identifier(module_id) else {
        continue;
      };
      if module.as_external_module().is_some() {
        continue;
      }
      let chunk_ukey = match EsmLibraryPlugin::get_module_chunk(*module_id, compilation) {
        Ok(c) => c,
        Err(e) => {
          tracing::warn!(error = %e, "failed to resolve module chunk during optimize_chunks");
          continue;
        }
      };
      if strict_chunks.contains(&chunk_ukey) {
        continue;
      }
      let exports_info = exports_info_artifact
        .get_exports_info(module_id)
        .as_data(exports_info_artifact);
      let export_names: FxHashSet<Atom> = exports_info
        .exports()
        .iter()
        .filter(|(_, ei)| {
          !matches!(ei.provided(), Some(ExportProvided::NotProvided))
            && !matches!(ei.get_used(None), UsageState::Unused)
        })
        .map(|(name, _)| name.clone())
        .collect();
      chunk_module_exports
        .entry(chunk_ukey)
        .or_default()
        .push((*module_id, export_names));
    }

    // Step 2: Find modules with conflicting export names (same name in multiple modules)
    let mut modules_with_conflicts = IdentifierSet::default();
    for modules in chunk_module_exports.values() {
      let mut name_count: FxHashMap<&Atom, usize> = FxHashMap::default();
      for (_, exports) in modules {
        for name in exports {
          *name_count.entry(name).or_default() += 1;
        }
      }
      let conflicting_names: FxHashSet<&Atom> = name_count
        .iter()
        .filter(|(_, count)| **count > 1)
        .map(|(name, _)| *name)
        .collect();
      if !conflicting_names.is_empty() {
        for (module_id, exports) in modules {
          if exports.iter().any(|n| conflicting_names.contains(n)) {
            modules_with_conflicts.insert(*module_id);
          }
        }
      }
    }

    // Step 3: Only assign namespace names when needed (namespace used as a whole or has conflicts)
    // Track used names per chunk to avoid collisions between multiple dyn targets
    let mut chunk_used_names: FxHashMap<ChunkUkey, FxHashSet<Atom>> = FxHashMap::default();

    for module_id in &sorted_targets {
      if !concatenated_modules.contains(module_id) {
        continue;
      }
      let Some(module) = module_graph.module_by_identifier(module_id) else {
        continue;
      };
      if module.as_external_module().is_some() {
        continue;
      }
      let chunk_ukey = match EsmLibraryPlugin::get_module_chunk(*module_id, compilation) {
        Ok(c) => c,
        Err(e) => {
          tracing::warn!(error = %e, "failed to resolve module chunk during optimize_chunks");
          continue;
        }
      };
      if strict_chunks.contains(&chunk_ukey) {
        continue;
      }
      // Skip namespace object for modules that don't need it:
      // only needed if namespace is used as a whole or has export name conflicts
      if !namespace_targets.contains(module_id) && !modules_with_conflicts.contains(module_id) {
        continue;
      }
      // Compute namespace_object_name using the same logic as deconflict_symbols
      let readable_identifier = get_cached_readable_identifier(
        module_id,
        module_graph,
        &compilation.module_static_cache,
        &compilation.options.context,
      );
      let escaped_idents = split_readable_identifier(&readable_identifier);
      let used_names = chunk_used_names.entry(chunk_ukey).or_default();
      let ns_name = find_new_name("namespaceObject", used_names, &escaped_idents);
      used_names.insert(ns_name.clone());
      ns_map.insert(*module_id, ns_name);
    }
  }

  (strict_chunks, all_dyn_targets, namespace_targets)
}

/// Compute a short name from a module identifier.
///
/// Identifiers may carry a module-type prefix (`css|…`), trailing metadata
/// separated by `|`, or a `?query` suffix.  These are stripped first so that
/// only the clean file path is used for name derivation.
///
/// Rules (applied to the cleaned path):
/// - If the filename stem is "index" and the path is inside `node_modules`,
///   use the package name (the segment right after `node_modules/`,
///   or `@scope/pkg` for scoped packages).
/// - If the filename stem is "index" otherwise, use the parent directory name.
/// - Otherwise, use the filename stem (without extension).
///
/// Examples:
/// - `node_modules/lib/dist/index.js` → `lib`
/// - `node_modules/@scope/pkg/dist/index.js` → `@scope/pkg`
/// - `/path/to/src/app.js` → `app`
/// - `css|./node_modules/lib/dist/index.css|0||||}` → `lib`
/// - `/path/to/src/index.js?query=1` → `src`
fn short_name_from_identifier(identifier: &str) -> Option<String> {
  // Strip ?query suffix.
  let s = identifier
    .split_once('?')
    .map_or(identifier, |(path, _)| path);

  // Strip module-type prefix and trailing metadata.
  // e.g. "css|./path/to/file.css|0||||}" → "./path/to/file.css"
  let s = if let Some((_, rest)) = s.split_once('|') {
    rest.split('|').next().unwrap_or(rest)
  } else {
    s
  };

  // Normalize Windows backslashes to forward slashes so that all subsequent
  // string operations work uniformly regardless of platform.
  use cow_utils::CowUtils;
  let s = s.cow_replace('\\', "/");
  let s = s.as_ref();

  let last_slash = s.rfind('/');
  let filename = last_slash.map_or(s, |p| &s[p + 1..]);
  let stem = filename.rsplit_once('.').map_or(filename, |(name, _)| name);

  if stem != "index" {
    return Some(stem.to_owned());
  }

  // For index files inside node_modules, extract the package name.
  if let Some(nm_pos) = s.rfind("node_modules/") {
    let after_nm = &s[nm_pos + "node_modules/".len()..];
    let pkg_end = if after_nm.starts_with('@') {
      // Scoped package (@scope/pkg): find the second '/'.
      let first = after_nm.find('/')?;
      first + 1 + after_nm[first + 1..].find('/')?
    } else {
      after_nm.find('/')?
    };
    return Some(after_nm[..pkg_end].to_owned());
  }

  // Fallback: parent directory name.
  let dir = &s[..last_slash?];
  let start = dir.rfind('/').map_or(0, |p| p + 1);
  Some(dir[start..].to_owned())
}

/// For unnamed dynamic-import chunks with exactly one root module,
/// assign a short name derived from the root module's identifier.
///
/// Names are deduplicated: if a name conflicts with any existing named chunk
/// or another computed name, a `~N` suffix is appended (deterministic index).
pub(crate) fn assign_dyn_import_chunk_short_names(compilation: &mut Compilation) {
  let module_graph = compilation.get_module_graph();

  // Collect all existing named chunks
  let mut used_names: FxHashMap<String, usize> = FxHashMap::default();
  for name in compilation.build_chunk_graph_artifact.named_chunks.keys() {
    used_names.insert(name.clone(), 1);
  }

  // Collect candidates: (chunk_ukey, root_module_identifier) for unnamed non-initial chunks
  // with exactly one root module
  let mut candidates: Vec<(ChunkUkey, ModuleIdentifier)> = Vec::new();

  for (chunk_ukey, chunk) in compilation.build_chunk_graph_artifact.chunk_by_ukey.iter() {
    // Skip chunks that already have a name
    if chunk.name().is_some() {
      continue;
    }
    // Only target non-initial chunks (dynamic import chunks)
    if chunk.can_be_initial(&compilation.build_chunk_graph_artifact.chunk_group_by_ukey) {
      continue;
    }
    let root_modules = compilation
      .build_chunk_graph_artifact
      .chunk_graph
      .get_chunk_root_modules(
        chunk_ukey,
        module_graph,
        &compilation.module_graph_cache_artifact,
        &compilation
          .build_module_graph_artifact
          .side_effects_state_artifact
          .read()
          .expect("should lock side effects state artifact"),
        &compilation.exports_info_artifact,
      );
    if root_modules.len() == 1 {
      candidates.push((*chunk_ukey, root_modules[0]));
    }
  }

  // Sort by module identifier for deterministic ordering
  candidates.sort_by(|a, b| a.1.cmp(&b.1));

  // Compute short names and track duplicates
  // name_to_chunks: maps base_name → list of (chunk_ukey, module_identifier) in sorted order
  let mut name_to_chunks: Vec<(String, Vec<(ChunkUkey, ModuleIdentifier)>)> = Vec::new();
  let mut name_index_map: FxHashMap<String, usize> = FxHashMap::default();

  for (chunk_ukey, module_id) in &candidates {
    let Some(module_path) = module_graph
      .module_by_identifier(module_id)
      .expect("should have module")
      .name_for_condition()
    else {
      continue;
    };
    let Some(base_name) = short_name_from_identifier(module_path.as_ref()) else {
      continue;
    };
    if let Some(&idx) = name_index_map.get(&base_name) {
      name_to_chunks[idx].1.push((*chunk_ukey, *module_id));
    } else {
      let idx = name_to_chunks.len();
      name_index_map.insert(base_name.clone(), idx);
      name_to_chunks.push((base_name, vec![(*chunk_ukey, *module_id)]));
    }
  }

  // Assign names, handling deduplication
  let mut assignments: Vec<(ChunkUkey, String)> = Vec::new();

  for (base_name, chunks) in &name_to_chunks {
    if chunks.len() == 1 && !used_names.contains_key(base_name) {
      // Unique name, no conflict
      assignments.push((chunks[0].0, base_name.clone()));
      used_names.insert(base_name.clone(), 1);
    } else {
      // Need dedup suffixes: chunks are already sorted by module identifier
      for (chunk_ukey, _module_id) in chunks {
        let mut index = used_names.get(base_name).copied().unwrap_or(0);
        let name = loop {
          let candidate = if index == 0 {
            base_name.clone()
          } else {
            format!("{base_name}~{index}")
          };
          if !used_names.contains_key(&candidate) {
            break candidate;
          }
          index += 1;
        };
        // Record the next index to try for this base_name
        used_names.insert(base_name.clone(), index + 1);
        used_names.insert(name.clone(), 1);
        assignments.push((*chunk_ukey, name));
      }
    }
  }

  // Apply assignments
  for (chunk_ukey, name) in assignments {
    let chunk = compilation
      .build_chunk_graph_artifact
      .chunk_by_ukey
      .expect_get_mut(&chunk_ukey);
    chunk.set_name(Some(name.clone()));
    compilation
      .build_chunk_graph_artifact
      .named_chunks
      .insert(name, chunk_ukey);
  }
}

#[cfg(test)]
mod tests {
  use super::short_name_from_identifier;

  #[test]
  fn short_name_non_index_file() {
    assert_eq!(
      short_name_from_identifier("/path/to/src/app.js"),
      Some("app".into())
    );
  }

  #[test]
  fn short_name_index_uses_parent_dir() {
    assert_eq!(
      short_name_from_identifier("/path/to/src/index.js"),
      Some("src".into())
    );
  }

  #[test]
  fn short_name_node_modules_flat() {
    assert_eq!(
      short_name_from_identifier("node_modules/lib/index.js"),
      Some("lib".into())
    );
  }

  #[test]
  fn short_name_node_modules_nested() {
    // The main bug: node_modules/lib/dist/index.js should return "lib", not "dist"
    assert_eq!(
      short_name_from_identifier("node_modules/lib/dist/index.js"),
      Some("lib".into())
    );
    assert_eq!(
      short_name_from_identifier("/abs/path/node_modules/my-pkg/lib/index.js"),
      Some("my-pkg".into())
    );
  }

  #[test]
  fn short_name_node_modules_scoped() {
    assert_eq!(
      short_name_from_identifier("node_modules/@scope/pkg/dist/index.js"),
      Some("@scope/pkg".into())
    );
    assert_eq!(
      short_name_from_identifier("/abs/node_modules/@org/lib/src/index.js"),
      Some("@org/lib".into())
    );
  }

  #[test]
  fn short_name_node_modules_non_index() {
    // Non-index files in node_modules should use the stem, not the package name
    assert_eq!(
      short_name_from_identifier("node_modules/lib/dist/utils.js"),
      Some("utils".into())
    );
  }

  #[test]
  fn short_name_strips_query() {
    assert_eq!(
      short_name_from_identifier("/path/to/src/app.js?query=1"),
      Some("app".into())
    );
    assert_eq!(
      short_name_from_identifier("/path/to/src/index.js?v=2&hash=abc"),
      Some("src".into())
    );
    assert_eq!(
      short_name_from_identifier("node_modules/lib/dist/index.js?query"),
      Some("lib".into())
    );
  }

  #[test]
  fn short_name_strips_module_type_prefix() {
    assert_eq!(
      short_name_from_identifier("css|./src/style.css|0||||}"),
      Some("style".into())
    );
    assert_eq!(
      short_name_from_identifier("css|./node_modules/lib/dist/index.css|0||||}"),
      Some("lib".into())
    );
    assert_eq!(
      short_name_from_identifier("css|./node_modules/@scope/pkg/dist/index.css|0||||}"),
      Some("@scope/pkg".into())
    );
    // type prefix without trailing metadata
    assert_eq!(
      short_name_from_identifier("css-module|./src/app.module.css"),
      Some("app.module".into())
    );
  }

  #[test]
  fn short_name_strips_both_prefix_and_query() {
    assert_eq!(
      short_name_from_identifier("css|./src/style.css?inline|0||||}"),
      Some("style".into())
    );
  }

  #[test]
  fn short_name_windows_backslash_paths() {
    assert_eq!(
      short_name_from_identifier(r"C:\repo\src\app.js"),
      Some("app".into())
    );
    assert_eq!(
      short_name_from_identifier(r"C:\repo\src\index.js"),
      Some("src".into())
    );
    assert_eq!(
      short_name_from_identifier(r"C:\repo\node_modules\lib\dist\index.js"),
      Some("lib".into())
    );
    assert_eq!(
      short_name_from_identifier(r"C:\repo\node_modules\@scope\pkg\dist\index.js"),
      Some("@scope/pkg".into())
    );
  }
}
