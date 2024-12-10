use std::path::Path;

use rayon::iter::{ParallelBridge, ParallelIterator};
use rspack_collections::{Identifier, IdentifierMap};
use rspack_error::Result;
use rspack_hash::RspackHashDigest;
use rspack_paths::ArcPath;
use rspack_sources::Source;
use rustc_hash::FxHashSet as HashSet;

use crate::{
  chunk_graph_module::ModuleId, fast_set, incremental::IncrementalPasses, ChunkGraph, ChunkKind,
  Compilation, Compiler, ModuleExecutor, RuntimeSpec,
};

impl Compiler {
  pub async fn rebuild(
    &mut self,
    changed_files: std::collections::HashSet<String>,
    deleted_files: std::collections::HashSet<String>,
  ) -> Result<()> {
    let old = self.compilation.get_stats();
    let old_hash = self.compilation.hash.clone();

    let (old_all_modules, old_runtime_modules) = collect_changed_modules(old.compilation)?;
    // TODO: should use `records`

    let all_old_runtime = old
      .compilation
      .get_chunk_graph_entries()
      .filter_map(|entry_ukey| old.compilation.chunk_by_ukey.get(&entry_ukey))
      .flat_map(|entry_chunk| entry_chunk.runtime().clone())
      .collect();

    let mut old_chunks: Vec<(String, RuntimeSpec)> = vec![];
    for (_, chunk) in old.compilation.chunk_by_ukey.iter() {
      if chunk.kind() != ChunkKind::HotUpdate {
        old_chunks.push((chunk.expect_id().to_string(), chunk.runtime().clone()));
      }
    }

    let records = CompilationRecords {
      old_chunks,
      all_old_runtime,
      old_all_modules,
      old_runtime_modules,
      old_hash,
    };

    // build without stats
    {
      let mut modified_files: HashSet<ArcPath> = HashSet::default();
      modified_files.extend(changed_files.iter().map(|files| Path::new(files).into()));
      let mut removed_files: HashSet<ArcPath> = HashSet::default();
      removed_files.extend(deleted_files.iter().map(|files| Path::new(files).into()));

      let mut all_files = modified_files.clone();
      all_files.extend(removed_files.clone());

      self.old_cache.end_idle();
      // self
      //   .old_cache
      //   .set_modified_files(all_files.into_iter().collect());

      self.plugin_driver.clear_cache();

      let mut new_compilation = Compilation::new(
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
      );

      new_compilation.hot_index = self.compilation.hot_index + 1;

      // TODO: remove this
      if let Some(mutations) = new_compilation.incremental.mutations_write()
        && let Some(old_mutations) = self.compilation.incremental.mutations_write()
      {
        mutations.swap_modules_with_chunk_graph_cache(old_mutations);
      }
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
        new_compilation.async_modules = std::mem::take(&mut self.compilation.async_modules);
      }
      if new_compilation
        .incremental
        .can_read_mutations(IncrementalPasses::DEPENDENCIES_DIAGNOSTICS)
      {
        new_compilation.dependencies_diagnostics =
          std::mem::take(&mut self.compilation.dependencies_diagnostics);
      }
      if new_compilation
        .incremental
        .can_read_mutations(IncrementalPasses::MODULE_IDS)
      {
        new_compilation.module_ids = std::mem::take(&mut self.compilation.module_ids);
      }
      if new_compilation
        .incremental
        .can_read_mutations(IncrementalPasses::MODULES_HASHES)
      {
        new_compilation.cgm_hash_results = std::mem::take(&mut self.compilation.cgm_hash_results);
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
        new_compilation.cgm_runtime_requirements_results =
          std::mem::take(&mut self.compilation.cgm_runtime_requirements_results);
      }
      if new_compilation
        .incremental
        .can_read_mutations(IncrementalPasses::CHUNKS_RUNTIME_REQUIREMENTS)
      {
        new_compilation.cgc_runtime_requirements_results =
          std::mem::take(&mut self.compilation.cgc_runtime_requirements_results);
      }
      if new_compilation
        .incremental
        .can_read_mutations(IncrementalPasses::CHUNKS_HASHES)
      {
        new_compilation.chunk_hashes_results =
          std::mem::take(&mut self.compilation.chunk_hashes_results);
      }
      if new_compilation
        .incremental
        .can_read_mutations(IncrementalPasses::CHUNKS_RENDER)
      {
        new_compilation.chunk_render_results =
          std::mem::take(&mut self.compilation.chunk_render_results);
      }

      // FOR BINDING SAFETY:
      // Update `compilation` for each rebuild.
      // Make sure `thisCompilation` hook was called before any other hooks that leverage `JsCompilation`.
      fast_set(&mut self.compilation, new_compilation);
      self.cache.before_compile(&mut self.compilation).await;
      self.compile().await?;

      self.old_cache.begin_idle();
    }

    self.compile_done().await?;
    self.cache.after_compile(&self.compilation);

    Ok(())
  }
}

#[derive(Debug)]
pub struct CompilationRecords {
  pub old_chunks: Vec<(String, RuntimeSpec)>,
  pub all_old_runtime: RuntimeSpec,
  pub old_all_modules: IdentifierMap<(RspackHashDigest, ModuleId)>,
  pub old_runtime_modules: IdentifierMap<String>,
  pub old_hash: Option<RspackHashDigest>,
}

pub type ChangedModules = (
  IdentifierMap<(RspackHashDigest, ModuleId)>,
  IdentifierMap<String>,
);
pub fn collect_changed_modules(compilation: &Compilation) -> Result<ChangedModules> {
  let modules_map = compilation
    .chunk_graph
    .chunk_graph_module_by_module_identifier
    .keys()
    .par_bridge()
    .filter_map(|identifier| {
      let cid = ChunkGraph::get_module_id(&compilation.module_ids, *identifier);
      // TODO: Determine how to calc module hash if module related to multiple runtime code
      // gen
      if let Some(code_generation_result) = compilation.code_generation_results.get_one(identifier)
        && let Some(module_hash) = &code_generation_result.hash
        && let Some(cid) = cid
      {
        Some((*identifier, (module_hash.clone(), cid.clone())))
      } else {
        None
      }
    })
    .collect::<IdentifierMap<_>>();

  let old_runtime_modules = compilation
    .runtime_modules
    .iter()
    .map(|(identifier, module)| -> Result<(Identifier, String)> {
      Ok((
        *identifier,
        module
          .generate_with_custom(compilation)?
          .source()
          .to_string(),
      ))
    })
    .collect::<Result<IdentifierMap<String>>>()?;

  Ok((modules_map, old_runtime_modules))
}
