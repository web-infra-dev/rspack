use std::path::PathBuf;

use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use rspack_error::Result;
use rspack_fs::AsyncWritableFileSystem;
use rspack_hash::RspackHashDigest;
use rspack_identifier::{Identifier, IdentifierMap};
use rspack_sources::Source;
use rustc_hash::FxHashSet as HashSet;

use super::MakeParam;
use crate::{fast_set, get_chunk_from_ukey, ChunkKind, Compilation, Compiler, RuntimeSpec};

impl<T> Compiler<T>
where
  T: AsyncWritableFileSystem + Send + Sync,
{
  pub async fn rebuild(
    &mut self,
    changed_files: std::collections::HashSet<String>,
    removed_files: std::collections::HashSet<String>,
  ) -> Result<()> {
    let old = self.compilation.get_stats();
    let old_hash = self.compilation.hash.clone();

    let (old_all_modules, old_runtime_modules) = collect_changed_modules(old.compilation)?;
    // TODO: should use `records`

    let mut all_old_runtime: RuntimeSpec = Default::default();
    for entry_ukey in old.compilation.get_chunk_graph_entries() {
      if let Some(runtime) = get_chunk_from_ukey(&entry_ukey, &old.compilation.chunk_by_ukey)
        .map(|entry_chunk| entry_chunk.runtime.clone())
      {
        all_old_runtime.extend(runtime);
      }
    }

    let mut old_chunks: Vec<(String, RuntimeSpec)> = vec![];
    for (_, chunk) in old.compilation.chunk_by_ukey.iter() {
      if chunk.kind != ChunkKind::HotUpdate {
        old_chunks.push((chunk.expect_id().to_string(), chunk.runtime.clone()));
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
      let mut modified_files = HashSet::default();
      modified_files.extend(changed_files.iter().map(PathBuf::from));
      let mut deleted_files = HashSet::default();
      deleted_files.extend(removed_files.iter().map(PathBuf::from));

      let mut all_files = modified_files.clone();
      all_files.extend(deleted_files.clone());

      self.cache.end_idle();
      self
        .cache
        .set_modified_files(all_files.into_iter().collect());
      self.plugin_driver.resolver_factory.clear_cache();

      let mut new_compilation = Compilation::new(
        self.options.clone(),
        self.plugin_driver.clone(),
        self.resolver_factory.clone(),
        self.loader_resolver_factory.clone(),
        Some(records),
        self.cache.clone(),
        // reuse module executor
        std::mem::take(&mut self.compilation.module_executor),
      );

      if let Some(state) = self.options.get_incremental_rebuild_make_state() {
        state.set_is_not_first();
      }

      new_compilation.hot_index = self.compilation.hot_index + 1;

      let is_incremental_rebuild_make = self.options.is_incremental_rebuild_make_enabled();
      if is_incremental_rebuild_make {
        // copy field from old compilation
        // make stage used
        self
          .compilation
          .swap_make_module_graph_with_compilation(&mut new_compilation);
        new_compilation.make_failed_dependencies =
          std::mem::take(&mut self.compilation.make_failed_dependencies);
        new_compilation.make_failed_module =
          std::mem::take(&mut self.compilation.make_failed_module);
        new_compilation.entries = std::mem::take(&mut self.compilation.entries);
        new_compilation.global_entry = std::mem::take(&mut self.compilation.global_entry);
        new_compilation.lazy_visit_modules =
          std::mem::take(&mut self.compilation.lazy_visit_modules);
        new_compilation.file_dependencies = std::mem::take(&mut self.compilation.file_dependencies);
        new_compilation.context_dependencies =
          std::mem::take(&mut self.compilation.context_dependencies);
        new_compilation.missing_dependencies =
          std::mem::take(&mut self.compilation.missing_dependencies);
        new_compilation.build_dependencies =
          std::mem::take(&mut self.compilation.build_dependencies);
        // tree shaking usage start
        new_compilation.optimize_analyze_result_map =
          std::mem::take(&mut self.compilation.optimize_analyze_result_map);
        new_compilation.entry_module_identifiers =
          std::mem::take(&mut self.compilation.entry_module_identifiers);
        new_compilation.bailout_module_identifiers =
          std::mem::take(&mut self.compilation.bailout_module_identifiers);
        // tree shaking usage end

        // seal stage used
        new_compilation.code_splitting_cache =
          std::mem::take(&mut self.compilation.code_splitting_cache);

        new_compilation.has_module_import_export_change = false;
      }

      let setup_make_params = if is_incremental_rebuild_make {
        vec![
          MakeParam::ModifiedFiles(modified_files),
          MakeParam::DeletedFiles(deleted_files),
        ]
      } else {
        vec![MakeParam::ForceBuildDeps(Default::default())]
      };

      new_compilation.lazy_visit_modules = changed_files.clone();

      // FOR BINDING SAFETY:
      // Update `compilation` for each rebuild.
      // Make sure `thisCompilation` hook was called before any other hooks that leverage `JsCompilation`.
      fast_set(&mut self.compilation, new_compilation);
      self.compile(setup_make_params).await?;

      self.cache.begin_idle();
    }

    self.compile_done().await?;

    Ok(())
  }
}

#[derive(Debug)]
pub struct CompilationRecords {
  pub old_chunks: Vec<(String, RuntimeSpec)>,
  pub all_old_runtime: RuntimeSpec,
  pub old_all_modules: IdentifierMap<(RspackHashDigest, String)>,
  pub old_runtime_modules: IdentifierMap<String>,
  pub old_hash: Option<RspackHashDigest>,
}

pub type ChangedModules = (
  IdentifierMap<(RspackHashDigest, String)>,
  IdentifierMap<String>,
);
pub fn collect_changed_modules(compilation: &Compilation) -> Result<ChangedModules> {
  let modules_map = compilation
    .chunk_graph
    .chunk_graph_module_by_module_identifier
    .par_iter()
    .filter_map(|(identifier, cgm)| {
      let cid = cgm.id.as_deref();
      // TODO: Determine how to calc module hash if module related to multiple runtime code
      // gen
      if let Some(code_generation_result) = compilation.code_generation_results.get_one(identifier)
        && let Some(module_hash) = &code_generation_result.hash
        && let Some(cid) = cid
      {
        Some((*identifier, (module_hash.clone(), cid.to_string())))
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
