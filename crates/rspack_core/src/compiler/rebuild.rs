use std::path::Path;

use rspack_collections::{DatabaseItem, IdentifierMap};
use rspack_error::Result;
use rspack_hash::RspackHashDigest;
use rspack_paths::ArcPath;
use rustc_hash::{FxHashMap, FxHashSet};

use crate::{
  chunk_graph_chunk::ChunkId, chunk_graph_module::ModuleId, fast_set,
  incremental::IncrementalPasses, ChunkGraph, ChunkKind, Compilation, Compiler, ModuleExecutor,
  RuntimeSpec, RuntimeSpecMap,
};

impl Compiler {
  #[tracing::instrument("Compiler:rebuild", skip_all, fields(
    compiler.changed_files = ?changed_files.iter().cloned().collect::<Vec<_>>(),
    compiler.deleted_files = ?deleted_files.iter().cloned().collect::<Vec<_>>()
  ))]
  pub async fn rebuild(
    &mut self,
    changed_files: std::collections::HashSet<String>,
    deleted_files: std::collections::HashSet<String>,
  ) -> Result<()> {
    let records = CompilationRecords::record(&self.compilation);

    // build without stats
    {
      let mut modified_files: FxHashSet<ArcPath> = FxHashSet::default();
      modified_files.extend(changed_files.iter().map(|files| Path::new(files).into()));
      let mut removed_files: FxHashSet<ArcPath> = FxHashSet::default();
      removed_files.extend(deleted_files.iter().map(|files| Path::new(files).into()));

      let mut all_files = modified_files.clone();
      all_files.extend(removed_files.clone());

      self.old_cache.end_idle();
      // self
      //   .old_cache
      //   .set_modified_files(all_files.into_iter().collect());

      self.plugin_driver.clear_cache();

      let mut new_compilation = Compilation::new(
        self.id,
        self.options.clone(),
        self.plugin_driver.clone(),
        self.buildtime_plugin_driver.clone(),
        self.resolver_factory.clone(),
        self.loader_resolver_factory.clone(),
        Some(records),
        self.cache.clone(),
        self.old_cache.clone(),
        Some(ModuleExecutor::default()),
        modified_files,
        removed_files,
        self.input_filesystem.clone(),
        self.intermediate_filesystem.clone(),
        self.output_filesystem.clone(),
        true,
      );

      new_compilation.hot_index = self.compilation.hot_index + 1;

      if new_compilation
        .incremental
        .can_read_mutations(IncrementalPasses::MAKE)
      {
        // copy field from old compilation
        // make stage used
        self
          .compilation
          .swap_make_artifact_with_compilation(&mut new_compilation);

        // seal stage used
        new_compilation.code_splitting_cache =
          std::mem::take(&mut self.compilation.code_splitting_cache);

        // reuse module executor
        new_compilation.module_executor = std::mem::take(&mut self.compilation.module_executor);
      }
      if new_compilation
        .incremental
        .can_read_mutations(IncrementalPasses::INFER_ASYNC_MODULES)
      {
        new_compilation.async_modules_artifact =
          std::mem::take(&mut self.compilation.async_modules_artifact);
      }
      if new_compilation
        .incremental
        .can_read_mutations(IncrementalPasses::DEPENDENCIES_DIAGNOSTICS)
      {
        new_compilation.dependencies_diagnostics_artifact =
          std::mem::take(&mut self.compilation.dependencies_diagnostics_artifact);
      }
      if new_compilation
        .incremental
        .can_read_mutations(IncrementalPasses::SIDE_EFFECTS)
      {
        new_compilation.side_effects_optimize_artifact =
          std::mem::take(&mut self.compilation.side_effects_optimize_artifact);
      }
      if new_compilation
        .incremental
        .can_read_mutations(IncrementalPasses::MODULE_IDS)
      {
        new_compilation.module_ids_artifact =
          std::mem::take(&mut self.compilation.module_ids_artifact);
      }
      if new_compilation
        .incremental
        .can_read_mutations(IncrementalPasses::CHUNK_IDS)
      {
        new_compilation.chunk_ids_artifact =
          std::mem::take(&mut self.compilation.chunk_ids_artifact);
      }
      if new_compilation
        .incremental
        .can_read_mutations(IncrementalPasses::MODULES_HASHES)
      {
        new_compilation.cgm_hash_artifact = std::mem::take(&mut self.compilation.cgm_hash_artifact);
      }
      if new_compilation
        .incremental
        .can_read_mutations(IncrementalPasses::MODULES_CODEGEN)
      {
        new_compilation.code_generation_results =
          std::mem::take(&mut self.compilation.code_generation_results);
      }
      if new_compilation
        .incremental
        .can_read_mutations(IncrementalPasses::MODULES_RUNTIME_REQUIREMENTS)
      {
        new_compilation.cgm_runtime_requirements_artifact =
          std::mem::take(&mut self.compilation.cgm_runtime_requirements_artifact);
      }
      if new_compilation
        .incremental
        .can_read_mutations(IncrementalPasses::CHUNKS_RUNTIME_REQUIREMENTS)
      {
        new_compilation.cgc_runtime_requirements_artifact =
          std::mem::take(&mut self.compilation.cgc_runtime_requirements_artifact);
      }
      if new_compilation
        .incremental
        .can_read_mutations(IncrementalPasses::CHUNKS_HASHES)
      {
        new_compilation.chunk_hashes_artifact =
          std::mem::take(&mut self.compilation.chunk_hashes_artifact);
      }
      if new_compilation
        .incremental
        .can_read_mutations(IncrementalPasses::CHUNKS_RENDER)
      {
        new_compilation.chunk_render_artifact =
          std::mem::take(&mut self.compilation.chunk_render_artifact);
      }

      // FOR BINDING SAFETY:
      // Update `compilation` for each rebuild.
      // Make sure `thisCompilation` hook was called before any other hooks that leverage `JsCompilation`.
      fast_set(&mut self.compilation, new_compilation);
      if let Err(err) = self.cache.before_compile(&mut self.compilation).await {
        self.compilation.push_diagnostic(err.into());
      }
      self.compile().await?;

      self.old_cache.begin_idle();
    }

    self.compile_done().await?;
    if let Err(err) = self.cache.after_compile(&self.compilation).await {
      self.compilation.push_diagnostic(err.into());
    }

    Ok(())
  }
}

#[derive(Debug)]
pub struct CompilationRecords {
  pub runtimes: RuntimeSpec,
  pub runtime_modules: IdentifierMap<RspackHashDigest>,
  pub chunks: FxHashMap<ChunkId, (RuntimeSpec, FxHashSet<ModuleId>)>,
  pub modules: FxHashMap<ModuleId, RuntimeSpecMap<RspackHashDigest>>,
  pub hash: Option<RspackHashDigest>,
}

impl CompilationRecords {
  pub fn record(compilation: &Compilation) -> Self {
    Self {
      runtimes: Self::record_runtimes(compilation),
      runtime_modules: Self::record_runtime_modules(compilation),
      chunks: Self::record_chunks(compilation),
      modules: Self::record_modules(compilation),
      hash: Self::record_hash(compilation),
    }
  }

  fn record_hash(compilation: &Compilation) -> Option<RspackHashDigest> {
    compilation.hash.clone()
  }

  fn record_modules(
    compilation: &Compilation,
  ) -> FxHashMap<ModuleId, RuntimeSpecMap<RspackHashDigest>> {
    compilation
      .chunk_graph
      .chunk_graph_module_by_module_identifier
      .keys()
      .filter_map(|identifier| {
        let module_id =
          ChunkGraph::get_module_id(&compilation.module_ids_artifact, *identifier)?.clone();
        let mut hashes = RuntimeSpecMap::new();
        for runtime in compilation
          .chunk_graph
          .get_module_runtimes_iter(*identifier, &compilation.chunk_by_ukey)
        {
          let hash = compilation
            .code_generation_results
            .get_hash(identifier, Some(runtime))
            .expect("should have hash");
          hashes.set(runtime.clone(), hash.clone());
        }
        Some((module_id, hashes))
      })
      .collect()
  }

  fn record_runtime_modules(compilation: &Compilation) -> IdentifierMap<RspackHashDigest> {
    compilation
      .runtime_modules
      .keys()
      .map(|identifier| {
        (
          *identifier,
          compilation
            .runtime_modules_hash
            .get(identifier)
            .expect("should have runtime module hash")
            .clone(),
        )
      })
      .collect()
  }

  fn record_runtimes(compilation: &Compilation) -> RuntimeSpec {
    compilation
      .get_chunk_graph_entries()
      .filter_map(|entry_ukey| compilation.chunk_by_ukey.get(&entry_ukey))
      .flat_map(|entry_chunk| entry_chunk.runtime().clone())
      .collect()
  }

  fn record_chunks(
    compilation: &Compilation,
  ) -> FxHashMap<ChunkId, (RuntimeSpec, FxHashSet<ModuleId>)> {
    compilation
      .chunk_by_ukey
      .values()
      .filter(|chunk| chunk.kind() != ChunkKind::HotUpdate)
      .map(|chunk| {
        let chunk_id = chunk.expect_id(&compilation.chunk_ids_artifact).clone();
        let chunk_runtime = chunk.runtime().clone();
        let chunk_modules: FxHashSet<ModuleId> = compilation
          .chunk_graph
          .get_chunk_modules_identifier(&chunk.ukey())
          .iter()
          .filter_map(|m| ChunkGraph::get_module_id(&compilation.module_ids_artifact, *m))
          .cloned()
          .collect();
        (chunk_id, (chunk_runtime, chunk_modules))
      })
      .collect()
  }
}
