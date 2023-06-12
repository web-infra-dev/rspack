use std::collections::HashSet;

use rspack_error::Error::InternalError;
use rspack_identifier::Identifiable;

use crate::{OptimizeChunksArgs, Plugin, PluginContext, PluginOptimizeChunksOutput};

#[derive(Debug)]
struct EnsureChunkConditionsPlugin;

impl Plugin for EnsureChunkConditionsPlugin {
  async fn optimize_chunks(
    &self,
    _ctx: PluginContext,
    args: OptimizeChunksArgs<'_>,
  ) -> PluginOptimizeChunksOutput {
    let mut compilation = args.compilation;

    let chunk_graph = &mut compilation.chunk_graph;

    let mut source_chunks = HashSet::new();
    let mut chunk_groups = HashSet::new();

    for (_, module) in compilation.module_graph.modules() {
      if !module.has_chunk_condition() {
        continue;
      }

      for chunk_key in chunk_graph.get_module_chunks(module.identifier()) {
        if module.chunk_condition(chunk, &*compilation) {}
        let chunk = compilation.chunk_by_ukey.get(chunk_key);
        if let Some(chunk) = chunk {
          source_chunks.insert(chunk_key);
          for chunk_group in &chunk.groups {
            chunk_groups.insert(chunk_group)
          }
        }
      }

      if source_chunks.is_empty() {
        continue;
      }

      let mut target_chunks = HashSet::new();
      'chunkGroupLoop: for chunk_group in chunk_groups {
        if let Some(chunk_group) = compilation.chunk_group_by_ukey.get(chunk_group) {
          for chunk in &chunk_group.chunks {
            target_chunks.insert(chunk.clone());
            continue 'chunkGroupLoop;
          }
          if chunk_group.is_initial() {
            // return Err()
            return Err(InternalError(rspack_error::InternalError {
              error_message: format!("Cannot fullfil chunk condition of {}", module.identifier()),
              severity: Default::default(),
            }));
          }

          for chunk_group_ukey in chunk_group.parents_iterable() {
            chunk_groups.insert(chunk_group_ukey)
          }
        }
      }
      for source_chunk in source_chunks {
        chunk_graph.disconnect_chunk_and_module(source_chunk, module.identifier());
      }

      for target_chunk in target_chunks {
        chunk_graph.connect_chunk_and_module(target_chunk, module.identifier());
      }

      source_chunks.clear();
      chunk_groups.clear();
    }

    Ok(())
  }
}
