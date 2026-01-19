use super::*;
use crate::logger::Logger;

pub async fn create_module_hashes_pass(compilation: &mut Compilation) -> Result<()> {
  // Check if MODULES_HASHES pass is disabled, and clear artifact if needed
  if !compilation
    .incremental
    .passes_enabled(IncrementalPasses::MODULES_HASHES)
  {
    compilation.cgm_hash_artifact.clear();
  }

  let create_module_hashes_modules = if let Some(mutations) = compilation
    .incremental
    .mutations_read(IncrementalPasses::MODULES_HASHES)
    && !compilation.cgm_hash_artifact.is_empty()
  {
    let revoked_modules = mutations.iter().filter_map(|mutation| match mutation {
      Mutation::ModuleRemove { module } => Some(*module),
      _ => None,
    });
    for revoked_module in revoked_modules {
      compilation.cgm_hash_artifact.remove(&revoked_module);
    }
    let mut modules = mutations.get_affected_modules_with_chunk_graph(compilation);

    // check if module runtime changes
    let mg = compilation.get_module_graph();
    for mi in mg.modules().keys() {
      let module_runtimes = compilation
        .build_chunk_graph_artifact.chunk_graph
        .get_module_runtimes(*mi, &compilation.build_chunk_graph_artifact.chunk_by_ukey);
      let module_runtime_keys = module_runtimes
        .values()
        .map(get_runtime_key)
        .collect::<HashSet<_>>();

      if let Some(runtime_map) = compilation.cgm_hash_artifact.get_runtime_map(mi) {
        if module_runtimes.is_empty() {
          // module has no runtime, skip
          continue;
        }
        if module_runtimes.len() == 1 {
          // single runtime
          if !matches!(runtime_map.mode, RuntimeMode::SingleEntry)
            || runtime_map
              .single_runtime
              .as_ref()
              .expect("should have single runtime for single entry")
              != module_runtimes
                .values()
                .next()
                .expect("should have at least one runtime")
          {
            modules.insert(*mi);
          }
        } else {
          // multiple runtimes
          if matches!(runtime_map.mode, RuntimeMode::SingleEntry) {
            modules.insert(*mi);
            continue;
          }

          if runtime_map.map.len() != module_runtimes.len() {
            modules.insert(*mi);
            continue;
          }

          for runtime_key in runtime_map.map.keys() {
            if !module_runtime_keys.contains(runtime_key) {
              modules.insert(*mi);
              break;
            }
          }
        }
      }
    }

    tracing::debug!(target: incremental::TRACING_TARGET, passes = %IncrementalPasses::MODULES_HASHES, %mutations, ?modules);
    let logger = compilation.get_logger("rspack.incremental.modulesHashes");
    logger.log(format!(
      "{} modules are affected, {} in total",
      modules.len(),
      mg.modules().len()
    ));

    modules
  } else {
    compilation
      .get_module_graph()
      .modules()
      .keys()
      .copied()
      .collect()
  };
  compilation
    .create_module_hashes(create_module_hashes_modules)
    .await
}

impl Compilation {
  #[instrument("Compilation:create_module_hashes", skip_all)]
  pub async fn create_module_hashes(&mut self, modules: IdentifierSet) -> Result<()> {
    let mg = self.get_module_graph();
    let chunk_graph = &self.build_chunk_graph_artifact.chunk_graph;
    let chunk_by_ukey = &self.build_chunk_graph_artifact.chunk_by_ukey;

    let results = rspack_futures::scope::<_, Result<_>>(|token| {
      for module_identifier in modules {
        let s = unsafe { token.used((&*self, &mg, chunk_graph, chunk_by_ukey)) };
        s.spawn(
          move |(compilation, mg, chunk_graph, chunk_by_ukey)| async move {
            let mut hashes = RuntimeSpecMap::new();
            let module = mg
              .module_by_identifier(&module_identifier)
              .expect("should have module");
            for runtime in chunk_graph.get_module_runtimes_iter(module_identifier, chunk_by_ukey) {
              let hash = module.get_runtime_hash(compilation, Some(runtime)).await?;
              hashes.set(runtime.clone(), hash);
            }
            Ok((module_identifier, hashes))
          },
        );
      }
    })
    .await
    .into_iter()
    .map(|r| r.to_rspack_result())
    .collect::<Result<Vec<_>>>()?;

    for result in results {
      let (module, hashes) = result?;
      if ChunkGraph::set_module_hashes(self, module, hashes)
        && let Some(mut mutations) = self.incremental.mutations_write()
      {
        mutations.add(Mutation::ModuleSetHashes { module });
      }
    }
    Ok(())
  }
}
