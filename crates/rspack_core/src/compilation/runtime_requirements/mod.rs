use std::mem;

use rayon::prelude::*;
use rspack_collections::{IdentifierSet, UkeyMap, UkeySet};
use rspack_error::{Result, ToStringResultToRspackResultExt};
use rspack_util::tracing_preset::TRACING_BENCH_TARGET;
use tracing::instrument;

use crate::{
  ChunkGraph, ChunkUkey, Compilation, Logger, RuntimeGlobals, RuntimeSpecMap, SharedPluginDriver,
};

#[instrument("Compilation:process_modules_runtime_requirements", skip_all)]
pub async fn process_modules_runtime_requirements(
  compilation: &mut Compilation,
  modules: IdentifierSet,
  plugin_driver: SharedPluginDriver,
) -> Result<()> {
  let logger = compilation.get_logger("rspack.Compilation");
  let start = logger.time("runtime requirements.modules");

  let module_results = rspack_futures::scope::<_, Result<_>>(|token| {
    modules
      .into_iter()
      .filter(|module| compilation.chunk_graph.get_number_of_module_chunks(*module) > 0)
      .for_each(|module| {
        let s = unsafe { token.used((&compilation, &plugin_driver)) };
        s.spawn(move |(compilation, plugin_driver)| async move {
          let mut map = RuntimeSpecMap::new();
          let runtimes = compilation
            .chunk_graph
            .get_module_runtimes_iter(module, &compilation.chunk_by_ukey);
          for runtime in runtimes {
            let runtime_requirements = compilation
              .old_cache
              .process_runtime_requirements_occasion
              .use_cache(module, runtime, compilation, || async {
                let mut runtime_requirements = compilation
                  .code_generation_results
                  .get_runtime_requirements(&module, Some(runtime));

                plugin_driver
                  .compilation_hooks
                  .additional_module_runtime_requirements
                  .call(compilation, &module, &mut runtime_requirements)
                  .await
                  .map_err(|e| {
                    e.wrap_err(
                      "caused by plugins in Compilation.hooks.additionalModuleRuntimeRequirements",
                    )
                  })?;

                compilation
                  .process_runtime_requirement_hook(&mut runtime_requirements, {
                    let plugin_driver = plugin_driver.clone();
                    move |compilation,
                          all_runtime_requirements,
                          runtime_requirements,
                          runtime_requirements_mut| {
                      Box::pin({
                        let plugin_driver = plugin_driver.clone();
                        async move {
                          plugin_driver
                            .compilation_hooks
                            .runtime_requirement_in_module
                            .call(
                              compilation,
                              &module,
                              all_runtime_requirements,
                              runtime_requirements,
                              runtime_requirements_mut,
                            )
                            .await
                            .map_err(|e| {
                              e.wrap_err(
                                "caused by plugins in Compilation.hooks.runtimeRequirementInModule",
                              )
                            })?;
                          Ok(())
                        }
                      })
                    }
                  })
                  .await?;
                Ok(runtime_requirements)
              })
              .await?;
            map.set(runtime.clone(), runtime_requirements);
          }
          Ok((module, map))
        });
      });
  })
  .await
  .into_iter()
  .map(|r| r.to_rspack_result())
  .collect::<Result<Vec<_>>>()?;

  for entry in module_results {
    let (module, map) = entry?;
    ChunkGraph::set_module_runtime_requirements(compilation, module, map);
  }
  logger.time_end(start);
  Ok(())
}

#[instrument(
  name = "Compilation:process_chunks_runtime_requirements",
  target = TRACING_BENCH_TARGET,
  skip_all
)]
pub async fn process_chunks_runtime_requirements(
  compilation: &mut Compilation,
  chunks: UkeySet<ChunkUkey>,
  entries: UkeySet<ChunkUkey>,
  plugin_driver: SharedPluginDriver,
) -> Result<()> {
  let logger = compilation.get_logger("rspack.Compilation");
  let start = logger.time("runtime requirements.chunks");
  let chunk_requirements = chunks
    .iter()
    .chain(entries.iter())
    .par_bridge()
    .map(|chunk_ukey| {
      let mut set = RuntimeGlobals::default();
      for mid in compilation
        .chunk_graph
        .get_chunk_modules_identifier(chunk_ukey)
      {
        let chunk = compilation.chunk_by_ukey.expect_get(chunk_ukey);
        if let Some(runtime_requirements) =
          ChunkGraph::get_module_runtime_requirements(compilation, *mid, chunk.runtime())
        {
          set.insert(*runtime_requirements);
        }
      }

      (*chunk_ukey, set)
    })
    .collect::<UkeyMap<_, _>>();

  for (chunk_ukey, mut set) in chunk_requirements {
    plugin_driver
      .compilation_hooks
      .additional_chunk_runtime_requirements
      .call(compilation, &chunk_ukey, &mut set)
      .await
      .map_err(|e| {
        e.wrap_err("caused by plugins in Compilation.hooks.additionalChunkRuntimeRequirements")
      })?;

    compilation
      .process_runtime_requirement_hook_mut(&mut set, {
        let plugin_driver = plugin_driver.clone();
        move |compilation,
              all_runtime_requirements,
              runtime_requirements,
              runtime_requirements_mut| {
          Box::pin({
            let plugin_driver = plugin_driver.clone();
            async move {
              plugin_driver
                .compilation_hooks
                .runtime_requirement_in_chunk
                .call(
                  compilation,
                  &chunk_ukey,
                  all_runtime_requirements,
                  runtime_requirements,
                  runtime_requirements_mut,
                )
                .await
                .map_err(|e| {
                  e.wrap_err("caused by plugins in Compilation.hooks.runtimeRequirementInChunk")
                })?;
              Ok(())
            }
          })
        }
      })
      .await?;

    ChunkGraph::set_chunk_runtime_requirements(compilation, chunk_ukey, set);
  }
  logger.time_end(start);

  let start = logger.time("runtime requirements.entries");
  for &entry_ukey in &entries {
    let entry = compilation.chunk_by_ukey.expect_get(&entry_ukey);
    let mut set = RuntimeGlobals::default();
    for chunk_ukey in entry
      .get_all_referenced_chunks(&compilation.chunk_group_by_ukey)
      .iter()
    {
      let runtime_requirements =
        ChunkGraph::get_chunk_runtime_requirements(compilation, chunk_ukey);
      set.insert(*runtime_requirements);
    }

    plugin_driver
      .compilation_hooks
      .additional_tree_runtime_requirements
      .call(compilation, &entry_ukey, &mut set)
      .await
      .map_err(|e| {
        e.wrap_err("caused by plugins in Compilation.hooks.additionalTreeRuntimeRequirements")
      })?;

    compilation
      .process_runtime_requirement_hook_mut(&mut set, {
        let plugin_driver = plugin_driver.clone();
        move |compilation,
              all_runtime_requirements,
              runtime_requirements,
              runtime_requirements_mut| {
          Box::pin({
            let plugin_driver = plugin_driver.clone();
            async move {
              plugin_driver
                .compilation_hooks
                .runtime_requirement_in_tree
                .call(
                  compilation,
                  &entry_ukey,
                  all_runtime_requirements,
                  runtime_requirements,
                  runtime_requirements_mut,
                )
                .await
                .map_err(|e| {
                  e.wrap_err("caused by plugins in Compilation.hooks.runtimeRequirementInTree")
                })?;
              Ok(())
            }
          })
        }
      })
      .await?;

    ChunkGraph::set_tree_runtime_requirements(compilation, entry_ukey, set);
  }

  // NOTE: webpack runs hooks.runtime_module in compilation.add_runtime_module
  // and overwrite the runtime_module.generate() to get new source in create_chunk_assets
  // this needs full runtime requirements, so run hooks.runtime_module after runtime_requirements_in_tree
  let mut runtime_modules = mem::take(&mut compilation.runtime_modules);
  for entry_ukey in &entries {
    let runtime_module_ids: Vec<_> = compilation
      .chunk_graph
      .get_chunk_runtime_modules_iterable(entry_ukey)
      .copied()
      .collect();
    for runtime_module_id in runtime_module_ids {
      plugin_driver
        .compilation_hooks
        .runtime_module
        .call(
          compilation,
          &runtime_module_id,
          entry_ukey,
          &mut runtime_modules,
        )
        .await
        .map_err(|e| e.wrap_err("caused by plugins in Compilation.hooks.runtimeModule"))?;
    }
  }
  compilation.runtime_modules = runtime_modules;

  logger.time_end(start);
  Ok(())
}
