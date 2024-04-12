use std::collections::{HashMap, HashSet};

use rspack_core::{
  get_chunk_from_ukey, get_chunk_group_from_ukey, Compilation, CompilationOptimizeChunks, Logger,
  Plugin, PluginContext,
};
use rspack_error::Result;
use rspack_hook::{plugin, plugin_hook};

#[plugin]
#[derive(Debug, Default)]
pub struct EnsureChunkConditionsPlugin;

#[plugin_hook(CompilationOptimizeChunks for EnsureChunkConditionsPlugin, stage = Compilation::OPTIMIZE_CHUNKS_STAGE_BASIC)]
fn optimize_chunks(&self, compilation: &mut Compilation) -> Result<Option<bool>> {
  let logger = compilation.get_logger(self.name());
  let start = logger.time("ensure chunk conditions");

  let mut source_module_chunks = HashMap::new();
  compilation
    .get_module_graph()
    .modules()
    .iter()
    .for_each(|(module_id, module)| {
      let source_chunks = compilation
        .chunk_graph
        .get_module_chunks(module.identifier())
        .iter()
        .flat_map(|chunk| {
          if matches!(module.chunk_condition(chunk, compilation), Some(false)) {
            return Some(chunk.to_owned());
          }
          None
        })
        .collect::<Vec<_>>();
      if !source_chunks.is_empty() {
        source_module_chunks.insert(*module_id, source_chunks);
      }
    });

  let mut target_module_chunks = HashMap::new();

  for (module_id, chunk_keys) in &source_module_chunks {
    let mut target_chunks = HashSet::new();
    for chunk_key in chunk_keys {
      if let Some(chunk) = get_chunk_from_ukey(chunk_key, &compilation.chunk_by_ukey) {
        let mut chunk_group_keys = chunk.groups.iter().collect::<Vec<_>>();
        let mut visited_chunk_group_keys = HashSet::new();
        'out: while let Some(chunk_group_key) = chunk_group_keys.pop() {
          if visited_chunk_group_keys.contains(chunk_group_key) {
            continue;
          }
          visited_chunk_group_keys.insert(chunk_group_key);
          if let Some(chunk_group) =
            get_chunk_group_from_ukey(chunk_group_key, &compilation.chunk_group_by_ukey)
          {
            for chunk in &chunk_group.chunks {
              if let Some(module) = compilation
                .get_module_graph()
                .module_by_identifier(module_id)
              {
                if matches!(module.chunk_condition(chunk, compilation), Some(true)) {
                  target_chunks.insert(*chunk);
                  continue 'out;
                }
              }
            }
            if chunk_group.is_initial() {
              return Err(
                rspack_error::InternalError::new(
                  format!("Cannot fulfil chunk condition of {}", module_id),
                  Default::default(),
                )
                .into(),
              );
            }
            let parent_chunks = chunk_group.parents_iterable();

            chunk_group_keys.extend(parent_chunks);
          }
        }
      }
    }
    target_module_chunks.insert(*module_id, target_chunks);
  }

  let mut chunk_graph = std::mem::take(&mut compilation.chunk_graph);
  for (module_id, chunks) in source_module_chunks {
    for chunk in chunks {
      chunk_graph.disconnect_chunk_and_module(&chunk, module_id.to_owned());
    }
  }

  for (module_id, chunks) in target_module_chunks {
    for chunk in chunks {
      chunk_graph.connect_chunk_and_module(chunk, module_id);
    }
  }
  compilation.chunk_graph = chunk_graph;

  logger.time_end(start);

  Ok(None)
}

impl Plugin for EnsureChunkConditionsPlugin {
  fn name(&self) -> &'static str {
    "rspack.EnsureChunkConditionsPlugin"
  }

  fn apply(
    &self,
    ctx: PluginContext<&mut rspack_core::ApplyContext>,
    _options: &mut rspack_core::CompilerOptions,
  ) -> Result<()> {
    ctx
      .context
      .compilation_hooks
      .optimize_chunks
      .tap(optimize_chunks::new(self));
    Ok(())
  }
}
