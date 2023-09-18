use std::collections::{HashMap, HashSet};

use rspack_core::{Logger, OptimizeChunksArgs, Plugin, PluginContext, PluginOptimizeChunksOutput};
use rspack_error::Error;

#[derive(Debug)]
pub struct EnsureChunkConditionsPlugin;

#[async_trait::async_trait]
impl Plugin for EnsureChunkConditionsPlugin {
  fn name(&self) -> &'static str {
    "rspack.EnsureChunkConditionsPlugin"
  }

  async fn optimize_chunks(
    &self,
    _ctx: PluginContext,
    args: OptimizeChunksArgs<'_>,
  ) -> PluginOptimizeChunksOutput {
    let compilation = args.compilation;
    let logger = compilation.get_logger(self.name());
    let start = logger.time("ensure chunk conditions");

    let mut source_module_chunks = HashMap::new();
    compilation
      .module_graph
      .modules()
      .iter()
      .for_each(|(module_id, module)| {
        if module.has_chunk_condition() {
          let source_chunks = compilation
            .chunk_graph
            .get_module_chunks(module.identifier())
            .iter()
            .flat_map(|chunk| {
              if !module.chunk_condition(chunk, compilation) {
                return Some(chunk.to_owned());
              }
              None
            })
            .collect::<Vec<_>>();
          source_module_chunks.insert(module_id, source_chunks);
        }
      });

    let mut target_module_chunks = HashMap::new();

    for (module_id, chunk_keys) in &source_module_chunks {
      let mut target_chunks = HashSet::new();
      for chunk_key in chunk_keys {
        if let Some(chunk) = compilation.chunk_by_ukey.get(chunk_key) {
          let mut chunk_group_keys = chunk.groups.iter().collect::<Vec<_>>();
          let mut visited_chunk_group_keys = HashSet::new();
          'out: while let Some(chunk_group_key) = chunk_group_keys.pop() {
            if visited_chunk_group_keys.contains(chunk_group_key) {
              continue;
            }
            visited_chunk_group_keys.insert(chunk_group_key);
            if let Some(chunk_group) = compilation.chunk_group_by_ukey.get(chunk_group_key) {
              for chunk in &chunk_group.chunks {
                if let Some(module) = compilation.module_graph.module_by_identifier(module_id) {
                  if module.chunk_condition(chunk, compilation) {
                    target_chunks.insert(chunk);
                    continue 'out;
                  }
                }
              }
              if chunk_group.is_initial() {
                return Err(Error::InternalError(rspack_error::InternalError {
                  error_message: format!("Cannot fulfil chunk condition of {}", module_id),
                  severity: Default::default(),
                }));
              }
              let parent_chunks = chunk_group.parents_iterable();

              chunk_group_keys.extend(parent_chunks);
            }
          }
        }
      }
      target_module_chunks.insert(*module_id, target_chunks);
    }
    for (module_id, chunks) in source_module_chunks {
      for chunk in chunks {
        compilation
          .chunk_graph
          .disconnect_chunk_and_module(&chunk, module_id.to_owned());
      }
    }

    for (module_id, chunks) in target_module_chunks {
      for chunk in chunks {
        compilation
          .chunk_graph
          .connect_chunk_and_module(chunk.to_owned(), module_id.to_owned());
      }
    }
    logger.time_end(start);

    Ok(())
  }
}
