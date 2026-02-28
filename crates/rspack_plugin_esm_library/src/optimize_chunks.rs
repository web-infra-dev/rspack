use rayon::prelude::*;
use rspack_collections::{IdentifierMap, IdentifierSet, UkeyDashSet, UkeyMap, UkeySet};
use rspack_core::{
  ChunkGroupUkey, ChunkUkey, Compilation, DependenciesBlock, DependencyType, Logger,
  ModuleIdentifier, incremental::Mutation,
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
  let mut entrypoint_chunks = UkeyMap::<ChunkUkey, ChunkGroupUkey>::default();
  let mut entry_module_belongs: IdentifierMap<UkeySet<ChunkUkey>> = IdentifierMap::default();

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

  let dirty_chunks = UkeyDashSet::default();

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

/// Create an empty facade chunk that will only contain re-exports.
/// The module stays in the original chunk — the facade re-exports from it.
fn create_facade_chunk(compilation: &mut Compilation, source_chunk_ukey: ChunkUkey) -> ChunkUkey {
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

  let [Some(source_chunk), Some(new_chunk)] = compilation
    .build_chunk_graph_artifact
    .chunk_by_ukey
    .get_many_mut([&source_chunk_ukey, &new_chunk_ukey])
  else {
    unreachable!()
  };

  source_chunk.split(
    new_chunk,
    &mut compilation.build_chunk_graph_artifact.chunk_group_by_ukey,
  );

  // No module movement — the facade chunk is empty, only re-exports.
  new_chunk_ukey
}

pub(crate) fn ensure_dyn_import_namespace_facades(
  compilation: &mut Compilation,
  concatenated_modules: &IdentifierSet,
) -> (UkeySet<ChunkUkey>, IdentifierSet, IdentifierMap<ChunkUkey>) {
  let module_graph = compilation.get_module_graph();
  // all_dyn_targets: all scope-hoisted modules that are dynamically imported
  // namespace_targets: subset that are imported as namespace (need facade split for multi-module chunks)
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
        exports_info_artifact,
      ) {
        continue;
      }
      let target = conn.module_identifier();
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

  // Classify targets: which are already in single-module/entry chunks vs which need splitting.
  // We must do this while module_graph is still borrowed, then drop it before mutations.
  let mut already_strict = UkeySet::default();
  let mut needs_split = Vec::new();

  let entrypoint_chunks: UkeySet<ChunkUkey> = compilation
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

  // Precompute: which modules are referenced from a different chunk?
  // For each module's outgoing active connections, if the target is in another chunk,
  // mark the target as cross-chunk referenced. O(total_connections) once.
  let mut cross_chunk_referenced = IdentifierSet::default();
  for chunk_ukey in compilation.build_chunk_graph_artifact.chunk_by_ukey.keys() {
    let modules = compilation
      .build_chunk_graph_artifact
      .chunk_graph
      .get_chunk_modules_identifier(chunk_ukey);
    for m in modules {
      let outgoings = module_graph.get_active_outcoming_connections_by_module(
        m,
        None,
        module_graph,
        &compilation.module_graph_cache_artifact,
        &compilation.exports_info_artifact,
      );
      for target in outgoings.keys() {
        let target_chunks = compilation
          .build_chunk_graph_artifact
          .chunk_graph
          .get_module_chunks(*target);
        if target_chunks.iter().any(|c| c != chunk_ukey) {
          cross_chunk_referenced.insert(*target);
        }
      }
    }
  }

  // Track how many dyn targets are in each multi-module chunk.
  // When multiple dyn targets share a chunk, we need facades to avoid export conflicts.
  let mut dyn_targets_per_chunk = UkeyMap::<ChunkUkey, Vec<ModuleIdentifier>>::default();

  for module_id in &all_dyn_targets {
    let Some(module) = module_graph.module_by_identifier(module_id) else {
      continue;
    };
    if module.as_external_module().is_some() {
      continue;
    }

    let chunk_ukey = EsmLibraryPlugin::get_module_chunk(*module_id, compilation);
    let chunk_modules = compilation
      .build_chunk_graph_artifact
      .chunk_graph
      .get_chunk_modules_identifier(&chunk_ukey);

    if chunk_modules.len() <= 1 || entrypoint_chunks.contains(&chunk_ukey) {
      // Single-module or entry chunk: exports are already correct.
      // Mark as strict so link_entry_module_exports registers all exports,
      // allowing dyn_import.rs to skip .then() remapping.
      already_strict.insert(chunk_ukey);
    } else if namespace_targets.contains(module_id) {
      // Multi-module chunk with namespace import.
      // However, if none of the other modules in this chunk are referenced from
      // other chunks, the chunk's exports will only be the dyn-imported module's
      // exports — no name conflicts possible, so no facade is needed.
      let has_cross_chunk_sibling = chunk_modules
        .iter()
        .any(|other_m| other_m != module_id && cross_chunk_referenced.contains(other_m));
      if has_cross_chunk_sibling {
        needs_split.push((*module_id, chunk_ukey));
      } else {
        already_strict.insert(chunk_ukey);
      }
    } else if concatenated_modules.contains(module_id) {
      // Multi-module chunk with only named imports: track for potential conflict detection.
      dyn_targets_per_chunk
        .entry(chunk_ukey)
        .or_default()
        .push(*module_id);
    }
  }

  // When multiple dyn targets share the same multi-module chunk, their exports
  // would be merged into the chunk's exports. If they have overlapping export names,
  // this causes conflicts. Create facade chunks for all of them to ensure each
  // dynamic import gets the correct module's exports.
  for (chunk_ukey, targets) in &dyn_targets_per_chunk {
    if targets.len() > 1 {
      for module_id in targets {
        needs_split.push((*module_id, *chunk_ukey));
      }
    }
  }

  needs_split.sort_by_key(|(module_id, _)| *module_id);

  let mut strict_chunks = already_strict;
  let mut facade_mapping = IdentifierMap::default();
  let logger = compilation.get_logger("rspack.EsmLibraryPlugin");
  logger.debug(format!("create facade chunks: {needs_split:?}"));

  for (module_id, chunk_ukey) in needs_split {
    let facade_chunk_ukey = create_facade_chunk(compilation, chunk_ukey);
    strict_chunks.insert(facade_chunk_ukey);
    facade_mapping.insert(module_id, facade_chunk_ukey);
  }

  (strict_chunks, all_dyn_targets, facade_mapping)
}
