use async_trait::async_trait;

use super::*;
use crate::{cache::Cache, compilation::pass::PassExt, logger::Logger};

pub struct CreateModuleHashesPass;

#[async_trait]
impl PassExt for CreateModuleHashesPass {
  fn name(&self) -> &'static str {
    "create module hashes"
  }

  async fn before_pass(&self, compilation: &mut Compilation, cache: &mut dyn Cache) {
    cache.before_modules_hashes(compilation).await;
  }

  async fn run_pass(&self, compilation: &mut Compilation) -> Result<()> {
    create_module_hashes_pass_impl(compilation).await
  }

  async fn after_pass(&self, compilation: &mut Compilation, cache: &mut dyn Cache) {
    cache.after_modules_hashes(compilation).await;
  }
}

async fn create_module_hashes_pass_impl(compilation: &mut Compilation) -> Result<()> {
  // Check if MODULES_HASHES pass is disabled, and clear artifact if needed
  if !compilation
    .incremental
    .passes_enabled(IncrementalPasses::MODULES_HASHES)
  {
    compilation.cgm_hash_artifact.clear();
  }

  let all_modules_for_code_generation_hashes = if compilation
    .plugin_driver
    .compilation_hooks
    .concatenation_scope
    .is_empty()
  {
    None
  } else {
    Some(
      compilation
        .get_module_graph()
        .modules_keys()
        .copied()
        .collect::<IdentifierSet>(),
    )
  };
  let has_code_generation_hashes = all_modules_for_code_generation_hashes.is_some();
  let code_generation_hashes = if let Some(modules) = &all_modules_for_code_generation_hashes {
    collect_code_generation_hashes(compilation, modules).await?
  } else {
    Default::default()
  };

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

    add_modules_with_changed_runtimes(compilation, &mut modules);
    if has_code_generation_hashes {
      add_modules_with_changed_code_generation_hashes(
        compilation,
        &code_generation_hashes,
        &mut modules,
      );
    }

    tracing::debug!(target: incremental::TRACING_TARGET, passes = %IncrementalPasses::MODULES_HASHES, %mutations, ?modules);
    let logger = compilation.get_logger("rspack.incremental.modulesHashes");
    logger.log(format!(
      "{} modules are affected, {} in total",
      modules.len(),
      compilation.get_module_graph().modules_len()
    ));

    modules
  } else {
    all_modules_for_code_generation_hashes.unwrap_or_else(|| {
      compilation
        .get_module_graph()
        .modules_keys()
        .copied()
        .collect()
    })
  };
  create_module_hashes_with_code_generation_hashes(
    compilation,
    create_module_hashes_modules,
    &code_generation_hashes,
  )
  .await
}

fn add_modules_with_changed_runtimes(compilation: &Compilation, modules: &mut IdentifierSet) {
  let mg = compilation.get_module_graph();
  for mi in mg.modules_keys() {
    let module_runtimes = compilation
      .build_chunk_graph_artifact
      .chunk_graph
      .get_module_runtimes(*mi, &compilation.build_chunk_graph_artifact.chunk_by_ukey);

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

        let module_runtime_keys = module_runtimes
          .values()
          .map(get_runtime_key)
          .collect::<HashSet<_>>();
        for runtime_key in runtime_map.map.keys() {
          if !module_runtime_keys.contains(runtime_key) {
            modules.insert(*mi);
            break;
          }
        }
      }
    }
  }
}

fn add_modules_with_changed_code_generation_hashes(
  compilation: &Compilation,
  code_generation_hashes: &IdentifierMap<RspackHashDigest>,
  modules: &mut IdentifierSet,
) {
  for module in compilation.get_module_graph().modules_keys() {
    if compilation
      .cgm_hash_artifact
      .get_code_generation_hash(module)
      != code_generation_hashes.get(module)
    {
      modules.insert(*module);
    }
  }
}

async fn collect_code_generation_hashes(
  compilation: &Compilation,
  modules: &IdentifierSet,
) -> Result<IdentifierMap<RspackHashDigest>> {
  let hook = &compilation
    .plugin_driver
    .compilation_hooks
    .concatenation_scope;
  if hook.is_empty() {
    return Ok(Default::default());
  }

  let mut hashes = IdentifierMap::default();
  for module in modules {
    let scope = hook.call(compilation, *module).await?;
    if let Some(hash) = scope.and_then(|scope| scope.code_generation_hash) {
      hashes.insert(*module, hash);
    }
  }
  Ok(hashes)
}

fn merge_code_generation_hash(
  compilation: &Compilation,
  module_hash: RspackHashDigest,
  code_generation_hash: Option<&RspackHashDigest>,
) -> RspackHashDigest {
  let Some(code_generation_hash) = code_generation_hash else {
    return module_hash;
  };

  let mut hasher = RspackHasher::from(&compilation.options.output);
  rspack_hash::RspackHash::hash("module-code-generation-v1", &mut hasher);
  rspack_hash::RspackHash::hash(&module_hash, &mut hasher);
  rspack_hash::RspackHash::hash(code_generation_hash, &mut hasher);
  hasher.digest(&compilation.options.output.hash_digest)
}

#[instrument("Compilation:create_module_hashes", skip_all)]
pub async fn create_module_hashes(
  compilation: &mut Compilation,
  modules: IdentifierSet,
) -> Result<()> {
  let code_generation_hashes = collect_code_generation_hashes(compilation, &modules).await?;
  create_module_hashes_with_code_generation_hashes(compilation, modules, &code_generation_hashes)
    .await
}

async fn create_module_hashes_with_code_generation_hashes(
  compilation: &mut Compilation,
  modules: IdentifierSet,
  code_generation_hashes: &IdentifierMap<RspackHashDigest>,
) -> Result<()> {
  let mg = compilation.get_module_graph();
  let chunk_graph = &compilation.build_chunk_graph_artifact.chunk_graph;
  let chunk_by_ukey = &compilation.build_chunk_graph_artifact.chunk_by_ukey;
  let compilation_ref = &*compilation;

  let results = rspack_parallel::scope::<_, Result<_>>(|token| {
    for module_identifier in modules {
      let s = unsafe {
        token.used((
          compilation_ref,
          &mg,
          chunk_graph,
          chunk_by_ukey,
          code_generation_hashes,
        ))
      };
      s.spawn(
        move |(compilation, mg, chunk_graph, chunk_by_ukey, code_generation_hashes)| async move {
          let mut hashes = RuntimeSpecMap::new();
          let code_generation_hash = code_generation_hashes.get(&module_identifier).cloned();
          let module = mg
            .module_by_identifier(&module_identifier)
            .expect("should have module");
          for runtime in chunk_graph.get_module_runtimes_iter(module_identifier, chunk_by_ukey) {
            let hash = merge_code_generation_hash(
              compilation,
              module.get_runtime_hash(compilation, Some(runtime)).await?,
              code_generation_hash.as_ref(),
            );
            hashes.set(runtime.clone(), hash);
          }
          Ok((module_identifier, hashes, code_generation_hash))
        },
      );
    }
  })
  .await
  .into_iter()
  .map(|r| r.to_rspack_result())
  .collect::<Result<Vec<_>>>()?;

  for result in results {
    let (module, hashes, code_generation_hash) = result?;
    if ChunkGraph::set_module_hashes(compilation, module, hashes, code_generation_hash)
      && let Some(mut mutations) = compilation.incremental.mutations_write()
    {
      mutations.add(Mutation::ModuleSetHashes { module });
    }
  }
  Ok(())
}
