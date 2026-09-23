use rspack_core::{
  ChunkGroupKind, Compilation, CompilationOptimizeChunks, DependencyType, incremental::Mutation,
};
use rspack_error::Result;
use rspack_hook::plugin_hook;
use rustc_hash::FxHashSet;

use crate::{JsPlugin, JsPluginInner, dependency::is_resolve_entry};

// Run before chunk optimizers see the temporary asset entries. The asset's
// generated URL export must be synchronously available to each referencing module.
#[plugin_hook(CompilationOptimizeChunks for JsPlugin, stage = Compilation::OPTIMIZE_CHUNKS_STAGE_BASIC - 1)]
pub(super) async fn optimize_chunks(&self, compilation: &mut Compilation) -> Result<Option<bool>> {
  let module_graph = compilation.get_module_graph();
  let graph = &compilation.build_chunk_graph_artifact;
  let mut moves = Vec::new();
  for entry in &graph.async_entrypoints {
    let group = graph.chunk_group_by_ukey.expect_get(entry);
    // Unused dependencies can leave an entry with no entry module. Its origins
    // still identify the referencing modules, so we can remove that entry too.
    for origin in group.origins() {
      let Some(origin_module) = origin.module else {
        continue;
      };
      for connection in module_graph.get_outgoing_connections(&origin_module) {
        if module_graph
          .dependency_by_id(&connection.dependency_id)
          .dependency_type()
          != &DependencyType::ImportMetaResolve
        {
          continue;
        }
        let target = *connection.module_identifier();
        let Some(module) = module_graph.module_by_identifier(&target) else {
          continue;
        };
        if is_resolve_entry(module.as_ref()) {
          continue;
        }
        // A target can be referenced by several independent async entries.
        if module_graph
          .get_parent_block(&connection.dependency_id)
          .and_then(|block| {
            graph
              .chunk_graph
              .get_block_chunk_group(block, &graph.chunk_group_by_ukey)
          })
          .is_none_or(|dependency_group| dependency_group.ukey != *entry)
        {
          continue;
        }
        let Some(issuer) = connection.original_module_identifier else {
          continue;
        };
        let chunks = graph
          .chunk_graph
          .get_module_chunks(issuer)
          .iter()
          .copied()
          .filter(|chunk| {
            connection.is_target_active(
              module_graph,
              Some(graph.chunk_by_ukey.expect_get(chunk).runtime()),
              &compilation.module_graph_cache_artifact,
              &compilation
                .build_module_graph_artifact
                .side_effects_state_artifact,
              &compilation.exports_info_artifact,
            )
          })
          .collect::<Vec<_>>();
        moves.push((target, group.ukey, group.chunks.clone(), chunks));
      }
    }
  }

  let graph = &mut compilation.build_chunk_graph_artifact;
  let mut removed_entries = FxHashSet::default();
  for (module, group, old_chunks, chunks) in moves {
    for chunk in chunks {
      graph.chunk_graph.connect_chunk_and_module(chunk, module);
      if let Some(mut mutations) = compilation.incremental.mutations_write() {
        mutations.add(Mutation::ChunksIntegrate { to: chunk });
      }
    }
    for chunk in old_chunks {
      graph
        .chunk_graph
        .disconnect_chunk_and_entry_module(&chunk, module);
      if let Some(mut chunk_data) = graph.chunk_by_ukey.remove(&chunk) {
        graph
          .chunk_graph
          .disconnect_chunk(&mut chunk_data, &mut graph.chunk_group_by_ukey);
        if let Some(mut mutations) = compilation.incremental.mutations_write() {
          mutations.add(Mutation::ChunkRemove { chunk });
        }
      }
    }
    // Keep the block-to-group mapping, as for an optimized-away normal async
    // chunk, but remove the independent runtime and all async entry references.
    graph.chunk_group_by_ukey.expect_get_mut(&group).kind = ChunkGroupKind::Normal {
      options: Default::default(),
    };
    removed_entries.insert(group);
  }
  if !removed_entries.is_empty() {
    graph
      .async_entrypoints
      .retain(|entry| !removed_entries.contains(entry));
    let parents = graph
      .chunk_group_by_ukey
      .keys()
      .copied()
      .collect::<Vec<_>>();
    for parent in parents {
      let parent = graph.chunk_group_by_ukey.expect_get_mut(&parent);
      let removed = parent
        .async_entrypoints_iterable()
        .filter(|entry| removed_entries.contains(entry))
        .copied()
        .collect::<Vec<_>>();
      for entry in removed {
        parent.remove_async_entrypoint(&entry);
      }
    }
  }
  Ok(None)
}
