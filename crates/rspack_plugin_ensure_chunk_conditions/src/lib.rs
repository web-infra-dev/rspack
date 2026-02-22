use rspack_core::{Compilation, CompilationOptimizeChunks, Logger, Plugin};
use rspack_error::Result;
use rspack_hook::{plugin, plugin_hook};
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};
use tracing::info;

#[plugin]
#[derive(Debug, Default)]
pub struct EnsureChunkConditionsPlugin;

#[plugin_hook(CompilationOptimizeChunks for EnsureChunkConditionsPlugin, stage = Compilation::OPTIMIZE_CHUNKS_STAGE_BASIC)]
async fn optimize_chunks(&self, compilation: &mut Compilation) -> Result<Option<bool>> {
  let logger = compilation.get_logger(self.name());
  let start = logger.time("ensure chunk conditions");
  compilation
    .build_chunk_graph_artifact
    .chunk_graph
    .generate_dot(compilation, "before-ensure-chunk-conditions")
    .await;
  let mut source_module_chunks = HashMap::default();
  compilation
    .get_module_graph()
    .modules()
    .for_each(|(module_id, module)| {
      let source_chunks = compilation
        .build_chunk_graph_artifact
        .chunk_graph
        .get_module_chunks(module.identifier())
        .iter()
        .filter_map(|chunk| {
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

  let mut target_module_chunks = HashMap::default();
  let mut visited_chunk_group_keys = HashSet::default();

  // The following algorithm has high risk of performance problem, cause it's complexity is N(adjust_chunk_number) * N(adjust_module_number) * N(chunk_group_number) * N(chunk_in_chunk_group_number)
  // this is used to calculate the complexity of the adjust_chunk operation
  let mut adjust_chunk_size: u64 = 0;
  let mut adjust_module_size: u64 = 0;
  let mut adjust_chunk_group_size: u64 = 0;
  let mut adjust_chunk_in_chunk_group_size: u64 = 0;
  for (module_id, chunk_keys) in &source_module_chunks {
    adjust_module_size += 1;
    let mut target_chunks = HashSet::default();
    for chunk_key in chunk_keys {
      adjust_chunk_size += 1;
      if let Some(chunk) = compilation
        .build_chunk_graph_artifact
        .chunk_by_ukey
        .get(chunk_key)
      {
        let mut chunk_group_keys = chunk.groups().iter().collect::<Vec<_>>();
        visited_chunk_group_keys.clear();
        'out: while let Some(chunk_group_key) = chunk_group_keys.pop() {
          if visited_chunk_group_keys.contains(chunk_group_key) {
            continue;
          }
          visited_chunk_group_keys.insert(chunk_group_key);
          if let Some(chunk_group) = compilation
            .build_chunk_graph_artifact
            .chunk_group_by_ukey
            .get(chunk_group_key)
          {
            adjust_chunk_group_size += 1;
            for chunk in &chunk_group.chunks {
              adjust_chunk_in_chunk_group_size += 1;
              if let Some(module) = compilation
                .get_module_graph()
                .module_by_identifier(module_id)
                && matches!(module.chunk_condition(chunk, compilation), Some(true))
              {
                target_chunks.insert(*chunk);
                continue 'out;
              }
            }
            if chunk_group.is_initial() {
              return Err(rspack_error::error!(
                "Cannot fulfil chunk condition of {}",
                module_id
              ));
            }
            let parent_chunks = chunk_group.parents_iterable();

            chunk_group_keys.extend(parent_chunks);
          }
        }
      }
    }
    target_module_chunks.insert(*module_id, target_chunks);
  }
  info!(
    name:"ensure_chunk_conditions.complexity",
    adjust_chunk_size = adjust_chunk_size,
    adjust_module_size = adjust_module_size,
    adjust_chunk_group_size = adjust_chunk_group_size,
    adjust_chunk_in_chunk_group_size = adjust_chunk_in_chunk_group_size,

  );
  let mut chunk_graph = std::mem::take(&mut compilation.build_chunk_graph_artifact.chunk_graph);
  for (module_id, chunks) in source_module_chunks {
    for chunk in chunks {
      chunk_graph.disconnect_chunk_and_module(&chunk, module_id);
    }
  }

  for (module_id, chunks) in target_module_chunks {
    for chunk in chunks {
      chunk_graph.connect_chunk_and_module(chunk, module_id);
    }
  }
  compilation.build_chunk_graph_artifact.chunk_graph = chunk_graph;

  logger.time_end(start);

  Ok(None)
}

impl Plugin for EnsureChunkConditionsPlugin {
  fn name(&self) -> &'static str {
    "rspack.EnsureChunkConditionsPlugin"
  }

  fn apply(&self, ctx: &mut rspack_core::ApplyContext<'_>) -> Result<()> {
    ctx
      .compilation_hooks
      .optimize_chunks
      .tap(optimize_chunks::new(self));
    Ok(())
  }
}
