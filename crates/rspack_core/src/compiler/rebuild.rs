use std::path::Path;

use rspack_collections::IdentifierMap;
use rspack_error::Result;
use rspack_hash::RspackHashDigest;
use rspack_paths::InternedPathSet;
use rspack_tasks::within_compiler_context;
use rustc_hash::{FxHashMap as HashMap, FxHashSet};

use crate::{
  AsyncDependenciesBlockIdentifier, ChunkGraph, ChunkKind, Compilation, Compiler, GroupOptions,
  RuntimeModuleRecord, RuntimeSpec,
  chunk_graph_chunk::{ChunkId, ChunkIdMap},
  chunk_graph_module::{ModuleIdMap, ModuleIdSet},
  compilation::build_module_graph::ModuleExecutor,
  incremental::{Incremental, IncrementalPasses},
};

impl Compiler {
  pub async fn rebuild(
    &mut self,
    changed_files: FxHashSet<String>,
    deleted_files: FxHashSet<String>,
  ) -> Result<()> {
    let result = match within_compiler_context(
      self.compiler_context.clone(),
      self.rebuild_inner(changed_files, deleted_files),
    )
    .await
    {
      Ok(_) => {
        self
          .plugin_driver
          .compiler_hooks
          .done
          .call(&self.compilation)
          .await
      }
      Err(e) => {
        let failed = self
          .plugin_driver
          .compiler_hooks
          .failed
          .call(&self.compilation)
          .await;
        failed.and(Err(e))
      }
    };
    self.store_cache_metadata(result.is_ok());
    result
  }

  #[tracing::instrument("Compiler:rebuild", skip_all, fields(
    compiler.changed_files = ?changed_files.iter().cloned().collect::<Vec<_>>(),
    compiler.deleted_files = ?deleted_files.iter().cloned().collect::<Vec<_>>()
  ))]
  async fn rebuild_inner(
    &mut self,
    changed_files: FxHashSet<String>,
    deleted_files: FxHashSet<String>,
  ) -> Result<()> {
    let records = self.last_records.clone();

    // build without stats
    {
      let mut modified_files: InternedPathSet = InternedPathSet::default();
      modified_files.extend(changed_files.iter().map(|files| Path::new(files).into()));
      let mut removed_files: InternedPathSet = InternedPathSet::default();
      removed_files.extend(deleted_files.iter().map(|files| Path::new(files).into()));

      let mut all_files = modified_files.clone();
      all_files.extend(removed_files.iter().cloned());

      self.resolver_factory.clear_cache();
      self.loader_resolver_factory.clear_cache();
      self.plugin_driver.clear_cache(self.compilation.id());

      let compilation_logging = self.compilation.get_logging().clone();
      compilation_logging.clear();

      let mut next_compilation = Compilation::new(
        self.id,
        self.options.clone(),
        self.platform.clone(),
        self.plugin_driver.clone(),
        self.buildtime_plugin_driver.clone(),
        self.resolver_factory.clone(),
        self.loader_resolver_factory.clone(),
        records,
        Incremental::new_hot(self.options.incremental),
        Some(ModuleExecutor::default()),
        compilation_logging,
        self.new_cache.clone(),
        modified_files,
        removed_files,
        self.input_filesystem.clone(),
        self.intermediate_filesystem.clone(),
        self.output_filesystem.clone(),
        true,
        self.compiler_context.clone(),
      );
      next_compilation.hot_index = self.compilation.hot_index + 1;

      if next_compilation
        .incremental
        .mutations_readable(IncrementalPasses::BUILD_MODULE_GRAPH)
      {
        // reuse module executor
        next_compilation.module_executor = std::mem::take(&mut self.compilation.module_executor);
      }

      self.cache.store_hot_cache(&mut self.compilation);

      // Artifact recovery belongs to incremental compilation and is independent
      // from the configured build cache.
      let old_compilation = std::mem::replace(&mut self.compilation, next_compilation);
      self
        .incremental_artifacts
        .store_previous_compilation(Box::new(old_compilation));

      // FOR BINDING SAFETY:
      // Update `compilation` for each rebuild.
      // Make sure `thisCompilation` hook was called before any other hooks that leverage `JsCompilation`.
      self.cache.before_compile(&mut self.compilation).await;
      self.compile().await?;
    }

    self
      .plugin_driver
      .compiler_hooks
      .after_compile
      .call(&mut self.compilation)
      .await?;
    self.compile_done().await?;
    self.cache.after_compile(&self.compilation).await;

    #[cfg(allocative)]
    crate::utils::snapshot_allocative("rebuild", self)?;

    Ok(())
  }
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum EntryKey {
  Named(String),
  Async(AsyncDependenciesBlockIdentifier),
}

pub type EntryChunks = HashMap<EntryKey, (ChunkId, Vec<ChunkId>)>;

#[derive(Debug)]
pub struct CompilationRecords {
  pub runtimes: RuntimeSpec,
  pub runtime_modules: IdentifierMap<RuntimeModuleRecord>,
  pub chunks: ChunkIdMap<(RuntimeSpec, ModuleIdSet)>,
  pub entries: EntryChunks,
  pub modules: ModuleIdMap<ChunkIdMap<RspackHashDigest>>,
  pub hash: Option<RspackHashDigest>,
}

impl CompilationRecords {
  pub fn record(compilation: &Compilation) -> Self {
    Self {
      runtimes: Self::record_runtimes(compilation),
      runtime_modules: Self::record_runtime_modules(compilation),
      chunks: Self::record_chunks(compilation),
      entries: Self::record_entries(compilation),
      modules: Self::record_modules(compilation),
      hash: Self::record_hash(compilation),
    }
  }

  fn record_hash(compilation: &Compilation) -> Option<RspackHashDigest> {
    compilation.hash.clone()
  }

  fn record_modules(compilation: &Compilation) -> ModuleIdMap<ChunkIdMap<RspackHashDigest>> {
    compilation
      .build_chunk_graph_artifact
      .chunk_graph
      .chunk_graph_module_by_module_identifier
      .keys()
      .filter_map(|identifier| {
        let module_id =
          ChunkGraph::get_module_id(&compilation.module_ids_artifact, *identifier)?.clone();
        let mut hashes = ChunkIdMap::default();
        for chunk in compilation
          .build_chunk_graph_artifact
          .chunk_graph
          .get_module_chunks(*identifier)
        {
          let chunk = compilation
            .build_chunk_graph_artifact
            .chunk_by_ukey
            .expect_get(chunk);
          let chunk_id = chunk.id().expect("should have chunk_id").clone();
          let hash = compilation
            .code_generation_results
            .get_hash(identifier, Some(chunk.runtime()))
            .expect("should have hash");
          hashes.insert(chunk_id, hash.clone());
        }
        Some((module_id, hashes))
      })
      .collect()
  }

  fn record_runtime_modules(compilation: &Compilation) -> IdentifierMap<RuntimeModuleRecord> {
    compilation
      .runtime_modules
      .iter()
      .map(|(identifier, module)| {
        (
          *identifier,
          RuntimeModuleRecord {
            hash: compilation
              .runtime_modules_hash
              .get(identifier)
              .expect("should have runtime module hash")
              .clone(),
            update_data: module.get_update_data(compilation),
            // Retain loading metadata, but custom sources still need a full update.
            has_custom_source: module.get_custom_source().is_some(),
          },
        )
      })
      .collect()
  }

  fn record_runtimes(compilation: &Compilation) -> RuntimeSpec {
    compilation
      .get_chunk_graph_entries()
      .filter_map(|entry_ukey| {
        compilation
          .build_chunk_graph_artifact
          .chunk_by_ukey
          .get(&entry_ukey)
      })
      .flat_map(|entry_chunk| entry_chunk.runtime().iter().copied())
      .collect()
  }

  fn record_chunks(compilation: &Compilation) -> ChunkIdMap<(RuntimeSpec, ModuleIdSet)> {
    compilation
      .build_chunk_graph_artifact
      .chunk_by_ukey
      .values()
      .filter(|chunk| chunk.kind() != ChunkKind::HotUpdate)
      .map(|chunk| {
        let chunk_id = chunk.expect_id().clone();
        let chunk_runtime = chunk.runtime().clone();
        let chunk_modules: ModuleIdSet = compilation
          .build_chunk_graph_artifact
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

  /// Record startup dependencies for ordinary and async entries, including dependOn parents.
  pub fn record_entries(compilation: &Compilation) -> EntryChunks {
    if !compilation.hot {
      return Default::default();
    }
    let graph = &compilation.build_chunk_graph_artifact;
    let module_graph = compilation.get_module_graph();
    let entries = compilation
      .entrypoints()
      .iter()
      .map(|(name, group)| (EntryKey::Named(name.clone()), *group));
    // Keep async entries tied to their dependency blocks across compilations,
    // even when splitting changes their physical entry chunk IDs.
    let async_entries = module_graph
      .blocks()
      .values()
      .filter(|block| matches!(block.get_group_options(), Some(GroupOptions::Entrypoint(_))))
      .filter_map(|block| {
        graph
          .chunk_graph
          .get_block_chunk_group(&block.identifier(), &graph.chunk_group_by_ukey)
          .map(|group| (EntryKey::Async(block.identifier()), group.ukey()))
      });
    entries
      .chain(async_entries)
      .map(|(key, entry_group_ukey)| {
        let entry = graph.chunk_group_by_ukey.expect_get(&entry_group_ukey);
        let entry_chunk = graph
          .chunk_by_ukey
          .expect_get(&entry.get_entrypoint_chunk());
        let mut groups = vec![entry_group_ukey];
        let mut visited_groups = FxHashSet::default();
        let mut chunks = Vec::new();
        while let Some(group_ukey) = groups.pop() {
          let group = graph.chunk_group_by_ukey.expect_get(&group_ukey);
          if (group_ukey != entry_group_ukey && !group.is_initial())
            || !visited_groups.insert(group_ukey)
          {
            continue;
          }
          if group.is_initial() {
            groups.extend(group.parents.iter().copied());
          }
          chunks.extend(group.chunks.iter().map(|chunk_ukey| {
            graph
              .chunk_by_ukey
              .expect_get(chunk_ukey)
              .expect_id()
              .clone()
          }));
        }
        // This records membership only. Keep manifest serialization deterministic.
        chunks.sort_unstable();
        chunks.dedup();
        (key, (entry_chunk.expect_id().clone(), chunks))
      })
      .collect()
  }
}
