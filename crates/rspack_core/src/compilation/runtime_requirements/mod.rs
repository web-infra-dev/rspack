use async_trait::async_trait;

use super::*;
use crate::{cache::Cache, compilation::pass::PassExt, logger::Logger};

pub struct RuntimeRequirementsPass;

#[async_trait]
impl PassExt for RuntimeRequirementsPass {
  fn name(&self) -> &'static str {
    "runtime requirements"
  }

  async fn before_pass(&self, compilation: &mut Compilation, cache: &mut dyn Cache) {
    cache.before_modules_runtime_requirements(compilation).await;
    cache.before_chunks_runtime_requirements(compilation).await;
  }

  async fn run_pass(&self, compilation: &mut Compilation) -> Result<()> {
    runtime_requirements_pass_impl(compilation).await
  }

  async fn after_pass(&self, compilation: &mut Compilation, cache: &mut dyn Cache) {
    cache.after_modules_runtime_requirements(compilation).await;
    cache.after_chunks_runtime_requirements(compilation).await;
  }
}

async fn runtime_requirements_pass_impl(compilation: &mut Compilation) -> Result<()> {
  let plugin_driver = compilation.plugin_driver.clone();
  let process_runtime_requirements_modules = if let Some(mutations) = compilation
    .incremental
    .mutations_read(IncrementalPasses::MODULES_RUNTIME_REQUIREMENTS)
    && !compilation.cgm_runtime_requirements_artifact.is_empty()
  {
    let revoked_modules = mutations.iter().filter_map(|mutation| match mutation {
      Mutation::ModuleRemove { module } => Some(*module),
      _ => None,
    });
    for revoked_module in revoked_modules {
      compilation
        .cgm_runtime_requirements_artifact
        .remove(&revoked_module);
    }
    let modules: IdentifierSet = mutations
      .iter()
      .filter_map(|mutation| match mutation {
        Mutation::ModuleSetHashes { module } => Some(*module),
        _ => None,
      })
      .collect();
    let logger = compilation.get_logger("rspack.incremental.modulesRuntimeRequirements");
    logger.log(format!(
      "{} modules are affected, {} in total",
      modules.len(),
      compilation.get_module_graph().modules_len()
    ));
    modules
  } else {
    compilation.cgm_runtime_requirements_artifact =
      CgmRuntimeRequirementsArtifact::default().into();
    compilation
      .get_module_graph()
      .modules_keys()
      .copied()
      .collect()
  };
  compilation
    .process_modules_runtime_requirements(
      process_runtime_requirements_modules,
      plugin_driver.clone(),
    )
    .await?;
  let runtime_chunks = compilation.get_chunk_graph_entries().collect();

  // Check if CHUNKS_RUNTIME_REQUIREMENTS pass is disabled, and clear artifact if needed
  if !compilation
    .incremental
    .passes_enabled(IncrementalPasses::CHUNKS_RUNTIME_REQUIREMENTS)
  {
    compilation.cgc_runtime_requirements_artifact.clear();
  }

  let process_runtime_requirements_chunks = if let Some(mutations) = compilation
    .incremental
    .mutations_read(IncrementalPasses::CHUNKS_RUNTIME_REQUIREMENTS)
    && !compilation.cgc_runtime_requirements_artifact.is_empty()
  {
    let removed_chunks = mutations.iter().filter_map(|mutation| match mutation {
      Mutation::ChunkRemove { chunk } => Some(chunk),
      _ => None,
    });
    for removed_chunk in removed_chunks {
      compilation
        .cgc_runtime_requirements_artifact
        .remove(removed_chunk);
    }
    let affected_chunks = mutations.get_affected_chunks_with_chunk_graph(compilation);
    for affected_chunk in &affected_chunks {
      compilation
        .cgc_runtime_requirements_artifact
        .remove(affected_chunk);
    }
    for runtime_chunk in &runtime_chunks {
      compilation
        .cgc_runtime_requirements_artifact
        .remove(runtime_chunk);
    }
    compilation
      .cgc_runtime_requirements_artifact
      .retain(|chunk, _| {
        compilation
          .build_chunk_graph_artifact
          .chunk_by_ukey
          .contains(chunk)
      });
    let logger = compilation.get_logger("rspack.incremental.chunksRuntimeRequirements");
    logger.log(format!(
      "{} chunks are affected, {} in total",
      affected_chunks.len(),
      compilation.build_chunk_graph_artifact.chunk_by_ukey.len()
    ));
    affected_chunks
  } else {
    compilation
      .build_chunk_graph_artifact
      .chunk_by_ukey
      .keys()
      .copied()
      .collect()
  };
  compilation
    .process_chunks_runtime_requirements(
      process_runtime_requirements_chunks,
      runtime_chunks,
      plugin_driver.clone(),
    )
    .await?;
  Ok(())
}

macro_rules! process_runtime_requirement_hook_macro {
  ($name: ident, $s: ty, $c: ty) => {
    async fn $name<C: ?Sized>(
      self: $s,
      requirements: &mut RuntimeGlobals,
      context: &mut C,
      mut call_hook: impl for<'a> FnMut(
        $c,
        &'a RuntimeGlobals,
        &'a RuntimeGlobals,
        &'a mut RuntimeGlobals,
        &'a mut C,
      ) -> BoxFuture<'a, Result<()>>,
    ) -> Result<()> {
      let mut runtime_requirements_mut = *requirements;
      let mut runtime_requirements;

      loop {
        runtime_requirements = runtime_requirements_mut;
        runtime_requirements_mut = RuntimeGlobals::default();
        // runtime_requirements: rt_requirements of last time
        // runtime_requirements_mut: changed rt_requirements
        // requirements: all rt_requirements
        call_hook(
          self,
          requirements,
          &runtime_requirements,
          &mut runtime_requirements_mut,
          context,
        )
        .await?;

        // check if we have changes to runtime_requirements
        runtime_requirements_mut =
          runtime_requirements_mut.difference(requirements.intersection(runtime_requirements_mut));
        if runtime_requirements_mut.is_empty() {
          break;
        } else {
          requirements.insert(runtime_requirements_mut);
        }
      }
      Ok(())
    }
  };
}

impl Compilation {
  #[instrument("Compilation:process_modules_runtime_requirements", skip_all)]
  pub async fn process_modules_runtime_requirements(
    &mut self,
    modules: IdentifierSet,
    plugin_driver: SharedPluginDriver,
  ) -> Result<()> {
    let logger = self.get_logger("rspack.Compilation");
    let start = logger.time("runtime requirements.modules");

    let module_results = rspack_futures::scope::<_, Result<_>>(|token| {
      modules
        .into_iter()
        .filter(|module| self.build_chunk_graph_artifact.chunk_graph.get_number_of_module_chunks(*module) > 0)
        .for_each(|module| {
          let s = unsafe { token.used((&self, &plugin_driver)) };
          s.spawn(move |(compilation, plugin_driver)| async move {
            let mut map = RuntimeSpecMap::new();
            let runtimes = compilation.build_chunk_graph_artifact.chunk_graph
              .get_module_runtimes_iter(module, &compilation.build_chunk_graph_artifact.chunk_by_ukey);
            for runtime in runtimes {
              let runtime_requirements = compilation
                .process_runtime_requirements_cache_artifact
                .use_cache(module, runtime, compilation, || async {
                  let mut runtime_requirements = compilation
                    .code_generation_results
                    .get_runtime_requirements(&module, Some(runtime));

                  plugin_driver
                    .compilation_hooks
                    .additional_module_runtime_requirements
                    .call(compilation, &module, &mut runtime_requirements)
                    .await
                    .map_err(|e| e.wrap_err("caused by plugins in Compilation.hooks.additionalModuleRuntimeRequirements"))?;

                  compilation
                    .process_runtime_requirement_hook(&mut runtime_requirements, &mut (), {
                      let plugin_driver = plugin_driver.clone();
                      move |compilation,
                                  all_runtime_requirements,
                                  runtime_requirements,
                                  runtime_requirements_mut,
                                  _| {
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
                            .map_err(|e| e.wrap_err("caused by plugins in Compilation.hooks.runtimeRequirementInModule"))?;
                          Ok(())
                        }})
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
      ChunkGraph::set_module_runtime_requirements(self, module, map);
    }
    logger.time_end(start);
    Ok(())
  }

  #[instrument(name = "Compilation:process_chunks_runtime_requirements", target=TRACING_BENCH_TARGET skip_all)]
  pub async fn process_chunks_runtime_requirements(
    &mut self,
    chunks: UkeySet<ChunkUkey>,
    entries: UkeySet<ChunkUkey>,
    plugin_driver: SharedPluginDriver,
  ) -> Result<()> {
    let logger = self.get_logger("rspack.Compilation");
    let start = logger.time("runtime requirements.chunks");
    let chunk_requirements = chunks
      .iter()
      .chain(entries.iter())
      .par_bridge()
      .map(|chunk_ukey| {
        let mut set = RuntimeGlobals::default();
        for mid in self
          .build_chunk_graph_artifact
          .chunk_graph
          .get_chunk_modules_identifier(chunk_ukey)
        {
          let chunk = self
            .build_chunk_graph_artifact
            .chunk_by_ukey
            .expect_get(chunk_ukey);
          if let Some(runtime_requirements) =
            ChunkGraph::get_module_runtime_requirements(self, *mid, chunk.runtime())
          {
            set.insert(*runtime_requirements);
          }
        }

        (*chunk_ukey, set)
      })
      .collect::<UkeyMap<_, _>>();

    for (chunk_ukey, mut set) in chunk_requirements {
      let mut additional_runtime_modules = Vec::new();
      plugin_driver
        .compilation_hooks
        .additional_chunk_runtime_requirements
        .call(self, &chunk_ukey, &mut set, &mut additional_runtime_modules)
        .await
        .map_err(|e| {
          e.wrap_err("caused by plugins in Compilation.hooks.additionalChunkRuntimeRequirements")
        })?;

      for module in additional_runtime_modules {
        let additional_runtime_requirements = module.additional_runtime_requirements(self);
        set.extend(additional_runtime_requirements);
        self.add_runtime_module(&chunk_ukey, module)?;
      }

      let mut runtime_modules_to_add = Vec::new();
      self
        .process_runtime_requirement_hook_mut(&mut set, &mut runtime_modules_to_add, {
          let plugin_driver = plugin_driver.clone();
          move |compilation,
                all_runtime_requirements,
                runtime_requirements,
                runtime_requirements_mut,
                runtime_modules_to_add| {
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
                    runtime_modules_to_add,
                  )
                  .await
                  .map_err(|e| {
                    e.wrap_err("caused by plugins in Compilation.hooks.runtimeRequirementInChunk")
                  })?;
                for runtime_module in runtime_modules_to_add.iter() {
                  let additional_runtime_requirements =
                    runtime_module.additional_runtime_requirements(compilation);
                  runtime_requirements_mut.extend(additional_runtime_requirements);
                }
                Ok(())
              }
            })
          }
        })
        .await?;

      for module in runtime_modules_to_add {
        self.add_runtime_module(&chunk_ukey, module)?;
      }

      ChunkGraph::set_chunk_runtime_requirements(self, chunk_ukey, set);
    }
    logger.time_end(start);

    let start = logger.time("runtime requirements.entries");
    for &entry_ukey in &entries {
      let mut all_runtime_requirements = RuntimeGlobals::default();
      let mut runtime_modules_to_add: Vec<(ChunkUkey, Box<dyn RuntimeModule>)> = Vec::new();

      let entry = self
        .build_chunk_graph_artifact
        .chunk_by_ukey
        .expect_get(&entry_ukey);
      for chunk_ukey in entry
        .get_all_referenced_chunks(&self.build_chunk_graph_artifact.chunk_group_by_ukey)
        .iter()
      {
        let runtime_requirements = ChunkGraph::get_chunk_runtime_requirements(self, chunk_ukey);
        all_runtime_requirements.insert(*runtime_requirements);
      }

      let mut additional_runtime_modules = Vec::new();
      plugin_driver
        .compilation_hooks
        .additional_tree_runtime_requirements
        .call(
          self,
          &entry_ukey,
          &mut all_runtime_requirements,
          &mut additional_runtime_modules,
        )
        .await
        .map_err(|e| {
          e.wrap_err("caused by plugins in Compilation.hooks.additionalTreeRuntimeRequirements")
        })?;
      for module in additional_runtime_modules {
        let additional_runtime_requirements = module.additional_runtime_requirements(self);
        all_runtime_requirements.extend(additional_runtime_requirements);
        self.add_runtime_module(&entry_ukey, module)?;
      }

      // Inline process_runtime_requirement_hook logic for runtime_requirement_in_tree
      {
        let mut runtime_requirements_to_add = all_runtime_requirements;
        let mut runtime_requirements_added;
        loop {
          runtime_requirements_added = runtime_requirements_to_add;
          runtime_requirements_to_add = RuntimeGlobals::default();
          plugin_driver
            .compilation_hooks
            .runtime_requirement_in_tree
            .call(
              self,
              &entry_ukey,
              &all_runtime_requirements,
              &runtime_requirements_added,
              &mut runtime_requirements_to_add,
              &mut runtime_modules_to_add,
            )
            .await
            .map_err(|e| {
              e.wrap_err("caused by plugins in Compilation.hooks.runtimeRequirementInTree")
            })?;

          for runtime_module in runtime_modules_to_add.iter() {
            let additional_runtime_requirements =
              runtime_module.1.additional_runtime_requirements(self);
            runtime_requirements_to_add.extend(additional_runtime_requirements);
          }
          runtime_requirements_to_add = runtime_requirements_to_add
            .difference(all_runtime_requirements.intersection(runtime_requirements_to_add));
          if runtime_requirements_to_add.is_empty() {
            break;
          } else {
            all_runtime_requirements.insert(runtime_requirements_to_add);
          }
        }
      }

      ChunkGraph::set_tree_runtime_requirements(self, entry_ukey, all_runtime_requirements);
      for (chunk_ukey, module) in runtime_modules_to_add {
        self.add_runtime_module(&chunk_ukey, module)?;
      }
    }

    // NOTE: webpack runs hooks.runtime_module in compilation.add_runtime_module
    // and overwrite the runtime_module.generate() to get new source in create_chunk_assets
    // this needs full runtime requirements, so run hooks.runtime_module after runtime_requirements_in_tree
    let mut runtime_modules = mem::take(&mut self.runtime_modules);
    for entry_ukey in &entries {
      let runtime_module_ids: Vec<_> = self
        .build_chunk_graph_artifact
        .chunk_graph
        .get_chunk_runtime_modules_iterable(entry_ukey)
        .copied()
        .collect();
      for runtime_module_id in runtime_module_ids {
        plugin_driver
          .compilation_hooks
          .runtime_module
          .call(self, &runtime_module_id, entry_ukey, &mut runtime_modules)
          .await
          .map_err(|e| e.wrap_err("caused by plugins in Compilation.hooks.runtimeModule"))?;
      }
    }
    self.runtime_modules = runtime_modules;

    logger.time_end(start);
    Ok(())
  }

  process_runtime_requirement_hook_macro!(
    process_runtime_requirement_hook,
    &Compilation,
    &'a Compilation
  );
  process_runtime_requirement_hook_macro!(
    process_runtime_requirement_hook_mut,
    &mut Compilation,
    &'a mut Compilation
  );
}
