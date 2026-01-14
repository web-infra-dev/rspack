use rspack_collections::{IdentifierSet, UkeySet};
use rspack_error::Result;
use rustc_hash::FxHashSet as HashSet;
use tracing::instrument;

use crate::{
  Compilation, Logger, RuntimeMode, SharedPluginDriver,
  compilation::{
    assign_runtime_ids,
    build_chunk_graph::{build_chunk_graph, use_code_splitting_cache},
    build_module_graph::{build_module_graph, finish_build_module_graph},
    chunk_ids, code_generation, create_chunk_assets, create_hash, create_module_hashes, module_ids,
    optimize, optimize_code_generation, process_assets, runtime_requirements,
  },
  get_runtime_key, incremental,
  incremental::{IncrementalPasses, Mutation},
  reset_artifact_if_passes_disabled,
};

/// Runs all compilation passes: make (build module graph) and seal phases
#[instrument("Compilation:run_passes", skip_all)]
pub async fn run_passes(
  compilation: &mut Compilation,
  plugin_driver: SharedPluginDriver,
) -> Result<()> {
  // ===== MAKE PHASE =====
  make_phase(compilation).await?;

  // ===== FINISH MAKE PHASE =====
  finish_make_phase(compilation).await?;

  // ===== SEAL PHASE =====
  seal_phase(compilation, plugin_driver).await?;

  Ok(())
}

/// Make phase: builds the module graph
#[instrument("Compilation:make_phase", skip_all)]
pub async fn make_phase(compilation: &mut Compilation) -> Result<()> {
  // run module_executor
  if let Some(module_executor) = &mut compilation.module_executor {
    let mut module_executor = std::mem::take(module_executor);
    module_executor.hook_before_make(compilation).await?;
    compilation.module_executor = Some(module_executor);
  }

  let artifact = compilation.build_module_graph_artifact.take();
  compilation
    .build_module_graph_artifact
    .replace(build_module_graph(compilation, artifact).await?);

  compilation
    .in_finish_make
    .store(true, std::sync::atomic::Ordering::Release);

  Ok(())
}

/// Finish make phase: completes module graph construction
#[instrument("Compilation:finish_make_phase", skip_all)]
pub async fn finish_make_phase(compilation: &mut Compilation) -> Result<()> {
  compilation
    .in_finish_make
    .store(false, std::sync::atomic::Ordering::Release);

  // clean up the entry deps
  let make_artifact = compilation.build_module_graph_artifact.take();
  compilation
    .build_module_graph_artifact
    .replace(finish_build_module_graph(compilation, make_artifact).await?);

  // sync assets to module graph from module_executor
  if let Some(module_executor) = &mut compilation.module_executor {
    let mut module_executor = std::mem::take(module_executor);
    module_executor
      .hook_after_finish_modules(compilation)
      .await?;
    compilation.module_executor = Some(module_executor);
  }

  Ok(())
}

/// Seal phase: processes the module graph and generates assets
#[instrument("Compilation:seal_phase", skip_all)]
pub async fn seal_phase(
  compilation: &mut Compilation,
  plugin_driver: SharedPluginDriver,
) -> Result<()> {
  // add a checkpoint here since we may modify module graph later in incremental compilation
  // and we can recover to this checkpoint in the future
  if compilation
    .incremental
    .passes_enabled(IncrementalPasses::MAKE)
  {
    compilation
      .build_module_graph_artifact
      .module_graph
      .checkpoint();
  }

  if !compilation.options.mode.is_development() {
    compilation.module_static_cache_artifact.freeze();
  }

  let logger = compilation.get_logger("rspack.Compilation");

  // https://github.com/webpack/webpack/blob/main/lib/Compilation.js#L2809
  plugin_driver
    .compilation_hooks
    .seal
    .call(compilation)
    .await
    .map_err(|e| e.wrap_err("caused by plugins in Compilation.hooks.seal"))?;

  let start = logger.time("optimize dependencies");
  logger.time_end(start);

  // ModuleGraph is frozen for now on, we have a module graph that won't change
  // so now we can start to create a chunk graph based on the module graph

  let start = logger.time("create chunks");
  compilation.module_graph_cache_artifact.freeze();
  // Check if BUILD_CHUNK_GRAPH pass is disabled, and reset artifact state if needed
  reset_artifact_if_passes_disabled(
    &compilation.incremental,
    &mut compilation.build_chunk_graph_artifact,
  );

  use_code_splitting_cache(compilation, |compilation| async {
    let start = logger.time("rebuild chunk graph");
    build_chunk_graph(compilation)?;
    compilation
      .chunk_graph
      .generate_dot(compilation, "after-code-splitting")
      .await;
    logger.time_end(start);
    Ok(compilation)
  })
  .await?;

  optimize::optimize_chunks_phase(compilation, plugin_driver.clone()).await?;
  logger.time_end(start);

  optimize::optimize_tree_phase(compilation, plugin_driver.clone()).await?;

  // ChunkGraph is frozen for now on, we have a chunk graph that won't change
  // so now we can start to generate assets based on the chunk graph

  module_ids::module_ids(compilation, plugin_driver.clone()).await?;
  chunk_ids::chunk_ids(compilation, plugin_driver.clone()).await?;
  assign_runtime_ids::assign_runtime_ids(compilation);
  optimize_code_generation::optimize_code_generation(compilation, plugin_driver.clone()).await?;

  // ===== Module Hashes =====
  create_module_hashes_phase(compilation).await?;

  // ===== Code Generation =====
  code_generation_phase(compilation, plugin_driver.clone()).await?;

  // ===== Runtime Requirements =====
  runtime_requirements_phase(compilation, plugin_driver.clone()).await?;

  // ===== Hashing =====
  let start = logger.time("hashing");
  create_hash::create_hash(compilation, plugin_driver.clone()).await?;
  create_hash::runtime_modules_code_generation(compilation).await?;
  logger.time_end(start);

  // ===== Create Assets =====
  let start = logger.time("create module assets");
  create_chunk_assets::create_module_assets(compilation, plugin_driver.clone()).await;
  logger.time_end(start);

  let start = logger.time("create chunk assets");
  create_chunk_assets::create_chunk_assets(compilation, plugin_driver.clone()).await?;
  logger.time_end(start);

  // ===== Process Assets =====
  process_assets::process_assets(compilation, plugin_driver.clone()).await?;
  process_assets::after_process_assets(compilation, plugin_driver.clone()).await?;
  process_assets::after_seal(compilation, plugin_driver).await?;

  if !compilation.options.mode.is_development() {
    compilation.module_static_cache_artifact.unfreeze();
  }
  Ok(())
}

/// Creates module hashes with incremental support
async fn create_module_hashes_phase(compilation: &mut Compilation) -> Result<()> {
  // Check if MODULES_HASHES pass is disabled, and reset artifact state if needed
  reset_artifact_if_passes_disabled(&compilation.incremental, &mut compilation.cgm_hash_artifact);

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
        .chunk_graph
        .get_module_runtimes(*mi, &compilation.chunk_by_ukey);
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
  create_module_hashes::create_module_hashes(compilation, create_module_hashes_modules).await
}

/// Code generation phase with incremental support
async fn code_generation_phase(
  compilation: &mut Compilation,
  plugin_driver: SharedPluginDriver,
) -> Result<()> {
  let logger = compilation.get_logger("rspack.Compilation");
  let start = logger.time("code generation");

  // Check if MODULES_CODEGEN pass is disabled, and reset artifact state if needed
  reset_artifact_if_passes_disabled(
    &compilation.incremental,
    &mut compilation.code_generation_results,
  );

  let code_generation_modules = if let Some(mutations) = compilation
    .incremental
    .mutations_read(IncrementalPasses::MODULES_CODEGEN)
    && !compilation.code_generation_results.is_empty()
  {
    let revoked_modules = mutations.iter().filter_map(|mutation| match mutation {
      Mutation::ModuleRemove { module } => Some(*module),
      _ => None,
    });
    for revoked_module in revoked_modules {
      compilation.code_generation_results.remove(&revoked_module);
    }
    let modules: IdentifierSet = mutations
      .iter()
      .filter_map(|mutation| match mutation {
        Mutation::ModuleSetHashes { module } => Some(*module),
        _ => None,
      })
      .collect();
    // also cleanup for updated modules
    for module in &modules {
      compilation.code_generation_results.remove(module);
    }
    tracing::debug!(target: incremental::TRACING_TARGET, passes = %IncrementalPasses::MODULES_CODEGEN, %mutations);
    let logger = compilation.get_logger("rspack.incremental.modulesCodegen");
    logger.log(format!(
      "{} modules are affected, {} in total",
      modules.len(),
      compilation.get_module_graph().modules().len()
    ));
    modules
  } else {
    compilation.code_generation_results = Default::default();
    compilation
      .get_module_graph()
      .modules()
      .keys()
      .copied()
      .collect()
  };
  code_generation::code_generation(compilation, code_generation_modules).await?;

  let mut diagnostics = vec![];
  plugin_driver
    .compilation_hooks
    .after_code_generation
    .call(compilation, &mut diagnostics)
    .await
    .map_err(|e| e.wrap_err("caused by plugins in Compilation.hooks.afterCodeGeneration"))?;
  compilation.extend_diagnostics(diagnostics);

  logger.time_end(start);
  Ok(())
}

/// Runtime requirements phase with incremental support
async fn runtime_requirements_phase(
  compilation: &mut Compilation,
  plugin_driver: SharedPluginDriver,
) -> Result<()> {
  let logger = compilation.get_logger("rspack.Compilation");
  let start = logger.time("runtime requirements");

  // Check if MODULES_RUNTIME_REQUIREMENTS pass is disabled, and reset artifact state if needed
  reset_artifact_if_passes_disabled(
    &compilation.incremental,
    &mut compilation.cgm_runtime_requirements_artifact,
  );

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
      compilation.get_module_graph().modules().len()
    ));
    modules
  } else {
    compilation.cgm_runtime_requirements_artifact = Default::default();
    compilation
      .get_module_graph()
      .modules()
      .keys()
      .copied()
      .collect()
  };
  runtime_requirements::process_modules_runtime_requirements(
    compilation,
    process_runtime_requirements_modules,
    plugin_driver.clone(),
  )
  .await?;
  let runtime_chunks: UkeySet<_> = compilation.get_chunk_graph_entries().collect();

  // Check if CHUNKS_RUNTIME_REQUIREMENTS pass is disabled, and reset artifact state if needed
  reset_artifact_if_passes_disabled(
    &compilation.incremental,
    &mut compilation.cgc_runtime_requirements_artifact,
  );

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
      .retain(|chunk, _| compilation.chunk_by_ukey.contains(chunk));
    let logger = compilation.get_logger("rspack.incremental.chunksRuntimeRequirements");
    logger.log(format!(
      "{} chunks are affected, {} in total",
      affected_chunks.len(),
      compilation.chunk_by_ukey.len()
    ));
    affected_chunks
  } else {
    compilation.chunk_by_ukey.keys().copied().collect()
  };
  runtime_requirements::process_chunks_runtime_requirements(
    compilation,
    process_runtime_requirements_chunks,
    runtime_chunks,
    plugin_driver,
  )
  .await?;
  logger.time_end(start);

  Ok(())
}
