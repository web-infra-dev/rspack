use rspack_core::{Compilation, CompilationOptimizeChunks, Logger, Plugin};
use rspack_error::{Result, ToStringResultToRspackResultExt};
use rspack_hook::{plugin, plugin_hook};
use rspack_util::fx_hash::{FxDashMap, FxHashSet as HashSet};
use tracing::info;

#[plugin]
#[derive(Debug, Default)]
pub struct EnsureChunkConditionsPlugin;

#[plugin_hook(CompilationOptimizeChunks for EnsureChunkConditionsPlugin, stage = Compilation::OPTIMIZE_CHUNKS_STAGE_BASIC)]
async fn optimize_chunks(&self, compilation: &mut Compilation) -> Result<Option<bool>> {
  let logger = compilation.get_logger(self.name());
  let start = logger.time("ensure chunk conditions");
  let source_module_chunks = FxDashMap::default();
  let compilation_ref = &*compilation;
  let source_module_chunk_results = rspack_parallel::scope::<_, Result<_>>(|token| {
    for (module_id, module) in compilation_ref.get_module_graph().modules() {
      let module_id = *module_id;
      let s = unsafe { token.used((module.as_ref(), compilation_ref, &source_module_chunks)) };
      s.spawn(
        move |(module, compilation, source_module_chunks)| async move {
          let external_module = module.as_external_module();
          let module_chunks = compilation
            .build_chunk_graph_artifact
            .chunk_graph
            .get_module_chunks(module.identifier());
          let mut source_chunks = HashSet::default();
          for chunk in module_chunks {
            let condition = if let Some(external_module) = external_module {
              external_module
                .chunk_condition_with_hooks(chunk, compilation)
                .await?
            } else {
              module.chunk_condition(chunk, compilation)
            };
            if matches!(condition, Some(false)) {
              source_chunks.insert(*chunk);
            }
          }
          if !source_chunks.is_empty() {
            source_module_chunks.insert(module_id, source_chunks);
          }
          Ok(())
        },
      );
    }
  })
  .await;
  for result in source_module_chunk_results {
    result.to_rspack_result()??;
  }

  let source_module_chunks = source_module_chunks.into_iter().collect::<Vec<_>>();

  let target_module_chunks = FxDashMap::default();

  // The following algorithm has high risk of performance problem, cause it's complexity is N(adjust_chunk_number) * N(adjust_module_number) * N(chunk_group_number) * N(chunk_in_chunk_group_number)
  // this is used to calculate the complexity of the adjust_chunk operation
  let target_module_chunk_results = rspack_parallel::scope::<_, Result<_>>(|token| {
    for (module_id, chunk_keys) in &source_module_chunks {
      let module_id = *module_id;
      let s = unsafe { token.used((chunk_keys, compilation_ref, &target_module_chunks)) };
      s.spawn(
        move |(chunk_keys, compilation, target_module_chunks)| async move {
          let module = compilation
            .get_module_graph()
            .module_by_identifier(&module_id);
          let external_module = module.and_then(|module| module.as_external_module());
          let mut target_chunks = HashSet::default();
          let mut visited_chunk_group_keys = HashSet::default();
          let mut adjust_chunk_size: u64 = 0;
          let mut adjust_chunk_group_size: u64 = 0;
          let mut adjust_chunk_in_chunk_group_size: u64 = 0;

          for chunk_key in chunk_keys {
            adjust_chunk_size += 1;
            if let Some(chunk) = compilation
              .build_chunk_graph_artifact
              .chunk_by_ukey
              .get(chunk_key)
            {
              let mut chunk_group_keys = chunk.groups().iter().copied().collect::<Vec<_>>();
              visited_chunk_group_keys.clear();
              'out: while let Some(chunk_group_key) = chunk_group_keys.pop() {
                if !visited_chunk_group_keys.insert(chunk_group_key) {
                  continue;
                }
                if let Some(chunk_group) = compilation
                  .build_chunk_graph_artifact
                  .chunk_group_by_ukey
                  .get(&chunk_group_key)
                {
                  adjust_chunk_group_size += 1;

                  for chunk in &chunk_group.chunks {
                    if chunk_keys.contains(chunk) {
                      continue;
                    }

                    adjust_chunk_in_chunk_group_size += 1;
                    if let Some(module) = module {
                      let condition = if let Some(external_module) = external_module {
                        external_module
                          .chunk_condition_with_hooks(chunk, compilation)
                          .await?
                      } else {
                        module.chunk_condition(chunk, compilation)
                      };
                      if matches!(condition, Some(true)) {
                        target_chunks.insert(*chunk);
                        continue 'out;
                      }
                    }
                  }
                  if chunk_group.is_initial() {
                    return Err(rspack_error::error!(
                      "Cannot fulfil chunk condition of {}",
                      module_id
                    ));
                  }
                  chunk_group_keys.extend(chunk_group.parents_iterable().copied());
                }
              }
            }
          }
          target_module_chunks.insert(module_id, target_chunks);
          Ok((
            adjust_chunk_size,
            adjust_chunk_group_size,
            adjust_chunk_in_chunk_group_size,
          ))
        },
      );
    }
  })
  .await;

  let mut adjust_chunk_size: u64 = 0;
  let adjust_module_size = target_module_chunk_results.len() as u64;
  let mut adjust_chunk_group_size: u64 = 0;
  let mut adjust_chunk_in_chunk_group_size: u64 = 0;
  for result in target_module_chunk_results {
    let (
      module_adjust_chunk_size,
      module_adjust_chunk_group_size,
      module_adjust_chunk_in_chunk_group_size,
    ) = result.to_rspack_result()??;
    adjust_chunk_size += module_adjust_chunk_size;
    adjust_chunk_group_size += module_adjust_chunk_group_size;
    adjust_chunk_in_chunk_group_size += module_adjust_chunk_in_chunk_group_size;
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
