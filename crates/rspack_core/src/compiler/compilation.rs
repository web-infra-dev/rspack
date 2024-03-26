use std::{
  collections::{hash_map, VecDeque},
  fmt::Debug,
  hash::{BuildHasherDefault, Hash},
  path::PathBuf,
  sync::Arc,
};

use dashmap::DashSet;
use indexmap::{IndexMap, IndexSet};
use itertools::Itertools;
use rayon::prelude::*;
use rspack_error::{error, Diagnostic, Result, Severity, TWithDiagnosticArray};
use rspack_futures::FuturesResults;
use rspack_hash::{RspackHash, RspackHashDigest};
use rspack_hook::{
  AsyncSeries2Hook, AsyncSeries3Hook, AsyncSeriesBailHook, AsyncSeriesHook, SyncSeries4Hook,
};
use rspack_identifier::{Identifiable, Identifier, IdentifierMap, IdentifierSet};
use rspack_sources::{BoxSource, CachedSource, SourceExt};
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet, FxHasher};
use swc_core::ecma::ast::ModuleItem;
use tracing::instrument;

use super::{
  hmr::CompilationRecords,
  make::{update_module_graph, MakeParam},
};
use crate::{
  build_chunk_graph::build_chunk_graph,
  cache::{use_code_splitting_cache, Cache, CodeSplittingCache},
  get_chunk_from_ukey, get_mut_chunk_from_ukey, is_source_equal, prepare_get_exports_type,
  tree_shaking::{optimizer, visitor::SymbolRef, BailoutFlag, OptimizeDependencyResult},
  AddQueueHandler, AdditionalChunkRuntimeRequirementsArgs, AdditionalModuleRequirementsArgs,
  BoxDependency, BoxModule, BuildQueueHandler, BuildTimeExecutionQueueHandler, CacheCount,
  CacheOptions, Chunk, ChunkByUkey, ChunkContentHash, ChunkGraph, ChunkGroupByUkey, ChunkGroupUkey,
  ChunkHashArgs, ChunkKind, ChunkUkey, CodeGenerationResults, CompilationLogger,
  CompilationLogging, CompilerOptions, ContentHashArgs, DependencyId, DependencyType, Entry,
  EntryData, EntryOptions, Entrypoint, ErrorSpan, FactorizeQueueHandler, Filename, LocalFilenameFn,
  Logger, Module, ModuleFactory, ModuleGraph, ModuleIdentifier, PathData,
  ProcessDependenciesQueueHandler, RenderManifestArgs, ResolverFactory, RuntimeGlobals,
  RuntimeModule, RuntimeRequirementsInTreeArgs, RuntimeSpec, SharedPluginDriver, SourceType, Stats,
};
use crate::{tree_shaking::visitor::OptimizeAnalyzeResult, ExecuteModuleId};

pub type BuildDependency = (
  DependencyId,
  Option<ModuleIdentifier>, /* parent module */
);

pub type CompilationBuildModuleHook = AsyncSeriesHook<BoxModule>;
pub type CompilationStillValidModuleHook = AsyncSeriesHook<BoxModule>;
pub type CompilationSucceedModuleHook = AsyncSeriesHook<BoxModule>;
pub type CompilationExecuteModuleHook =
  SyncSeries4Hook<ModuleIdentifier, IdentifierSet, CodeGenerationResults, ExecuteModuleId>;
pub type CompilationFinishModulesHook = AsyncSeriesHook<Compilation>;
pub type CompilationOptimizeModulesHook = AsyncSeriesBailHook<Compilation, bool>;
pub type CompilationAfterOptimizeModulesHook = AsyncSeriesHook<Compilation>;
pub type CompilationOptimizeTreeHook = AsyncSeriesHook<Compilation>;
pub type CompilationOptimizeChunkModulesHook = AsyncSeriesBailHook<Compilation, bool>;
pub type CompilationRuntimeModuleHook = AsyncSeries3Hook<Compilation, ModuleIdentifier, ChunkUkey>;
pub type CompilationChunkAssetHook = AsyncSeries2Hook<Chunk, String>;
pub type CompilationProcessAssetsHook = AsyncSeriesHook<Compilation>;
pub type CompilationAfterProcessAssetsHook = AsyncSeriesHook<Compilation>;

#[derive(Debug, Default)]
pub struct CompilationHooks {
  pub build_module: CompilationBuildModuleHook,
  pub still_valid_module: CompilationStillValidModuleHook,
  pub succeed_module: CompilationSucceedModuleHook,
  pub execute_module: CompilationExecuteModuleHook,
  pub finish_modules: CompilationFinishModulesHook,
  pub optimize_modules: CompilationOptimizeModulesHook,
  pub after_optimize_modules: CompilationAfterOptimizeModulesHook,
  pub optimize_tree: CompilationOptimizeTreeHook,
  pub optimize_chunk_modules: CompilationOptimizeChunkModulesHook,
  pub runtime_module: CompilationRuntimeModuleHook,
  pub chunk_asset: CompilationChunkAssetHook,
  pub process_assets: CompilationProcessAssetsHook,
  pub after_process_assets: CompilationAfterProcessAssetsHook,
}

#[derive(Debug)]
pub struct Compilation {
  // Mark compilation status, because the hash of `[hash].hot-update.js/json` is previous compilation hash.
  // Status A(hash: A) -> Status B(hash: B) will generate `A.hot-update.js`
  // Status A(hash: A) -> Status C(hash: C) will generate `A.hot-update.js`
  // The status is different, should generate different hash for `.hot-update.js`
  // So use compilation hash update `hot_index` to fix it.
  pub hot_index: u32,
  pub records: Option<CompilationRecords>,
  pub options: Arc<CompilerOptions>,
  pub entries: Entry,
  pub global_entry: EntryData,
  module_graph: ModuleGraph,
  dependency_factories: HashMap<DependencyType, Arc<dyn ModuleFactory>>,
  pub make_failed_dependencies: HashSet<BuildDependency>,
  pub make_failed_module: HashSet<ModuleIdentifier>,
  pub has_module_import_export_change: bool,
  pub runtime_modules: IdentifierMap<Box<dyn RuntimeModule>>,
  pub runtime_module_code_generation_results: IdentifierMap<(RspackHashDigest, BoxSource)>,
  pub chunk_graph: ChunkGraph,
  pub chunk_by_ukey: ChunkByUkey,
  pub chunk_group_by_ukey: ChunkGroupByUkey,
  pub entrypoints: IndexMap<String, ChunkGroupUkey>,
  pub async_entrypoints: Vec<ChunkGroupUkey>,
  assets: CompilationAssets,
  pub emitted_assets: DashSet<String, BuildHasherDefault<FxHasher>>,
  diagnostics: Vec<Diagnostic>,
  logging: CompilationLogging,
  pub plugin_driver: SharedPluginDriver,
  pub resolver_factory: Arc<ResolverFactory>,
  pub loader_resolver_factory: Arc<ResolverFactory>,
  pub named_chunks: HashMap<String, ChunkUkey>,
  pub(crate) named_chunk_groups: HashMap<String, ChunkGroupUkey>,
  pub entry_module_identifiers: IdentifierSet,
  /// Collecting all used export symbol
  pub used_symbol_ref: HashSet<SymbolRef>,
  /// Collecting all module that need to skip in tree-shaking ast modification phase
  pub bailout_module_identifiers: IdentifierMap<BailoutFlag>,
  pub optimize_analyze_result_map: IdentifierMap<OptimizeAnalyzeResult>,

  pub code_generation_results: CodeGenerationResults,
  pub code_generated_modules: IdentifierSet,
  pub cache: Arc<Cache>,
  pub code_splitting_cache: CodeSplittingCache,
  pub hash: Option<RspackHashDigest>,
  // lazy compilation visit module
  pub lazy_visit_modules: std::collections::HashSet<String>,
  pub used_chunk_ids: HashSet<String>,
  pub include_module_ids: IdentifierSet,

  pub file_dependencies: IndexSet<PathBuf, BuildHasherDefault<FxHasher>>,
  pub context_dependencies: IndexSet<PathBuf, BuildHasherDefault<FxHasher>>,
  pub missing_dependencies: IndexSet<PathBuf, BuildHasherDefault<FxHasher>>,
  pub build_dependencies: IndexSet<PathBuf, BuildHasherDefault<FxHasher>>,
  pub side_effects_free_modules: IdentifierSet,
  pub module_item_map: IdentifierMap<Vec<ModuleItem>>,

  pub factorize_queue: Option<FactorizeQueueHandler>,
  pub build_queue: Option<BuildQueueHandler>,
  pub add_queue: Option<AddQueueHandler>,
  pub process_dependencies_queue: Option<ProcessDependenciesQueueHandler>,
  pub build_time_execution_queue: Option<BuildTimeExecutionQueueHandler>,
}

impl Compilation {
  pub const PROCESS_ASSETS_STAGE_ADDITIONAL: i32 = -2000;
  pub const PROCESS_ASSETS_STAGE_PRE_PROCESS: i32 = -1000;
  pub const PROCESS_ASSETS_STAGE_DERIVED: i32 = -200;
  pub const PROCESS_ASSETS_STAGE_ADDITIONS: i32 = -100;
  pub const PROCESS_ASSETS_STAGE_OPTIMIZE: i32 = 100;
  pub const PROCESS_ASSETS_STAGE_OPTIMIZE_COUNT: i32 = 200;
  pub const PROCESS_ASSETS_STAGE_OPTIMIZE_COMPATIBILITY: i32 = 300;
  pub const PROCESS_ASSETS_STAGE_OPTIMIZE_SIZE: i32 = 400;
  pub const PROCESS_ASSETS_STAGE_DEV_TOOLING: i32 = 500;
  pub const PROCESS_ASSETS_STAGE_OPTIMIZE_INLINE: i32 = 700;
  pub const PROCESS_ASSETS_STAGE_SUMMARIZE: i32 = 1000;
  pub const PROCESS_ASSETS_STAGE_OPTIMIZE_HASH: i32 = 2500;
  pub const PROCESS_ASSETS_STAGE_OPTIMIZE_TRANSFER: i32 = 3000;
  pub const PROCESS_ASSETS_STAGE_ANALYSE: i32 = 4000;
  pub const PROCESS_ASSETS_STAGE_REPORT: i32 = 5000;

  #[allow(clippy::too_many_arguments)]
  pub fn new(
    options: Arc<CompilerOptions>,
    module_graph: ModuleGraph,
    plugin_driver: SharedPluginDriver,
    resolver_factory: Arc<ResolverFactory>,
    loader_resolver_factory: Arc<ResolverFactory>,
    records: Option<CompilationRecords>,
    cache: Arc<Cache>,
  ) -> Self {
    let module_graph = module_graph.with_treeshaking(options.is_new_tree_shaking());
    Self {
      hot_index: 0,
      records,
      options,
      module_graph,
      dependency_factories: Default::default(),
      make_failed_dependencies: HashSet::default(),
      make_failed_module: HashSet::default(),
      has_module_import_export_change: true,
      runtime_modules: Default::default(),
      runtime_module_code_generation_results: Default::default(),
      chunk_by_ukey: Default::default(),
      chunk_group_by_ukey: Default::default(),
      entries: Default::default(),
      global_entry: Default::default(),
      chunk_graph: Default::default(),
      entrypoints: Default::default(),
      async_entrypoints: Default::default(),
      assets: Default::default(),
      emitted_assets: Default::default(),
      diagnostics: Default::default(),
      logging: Default::default(),
      plugin_driver,
      resolver_factory,
      loader_resolver_factory,
      named_chunks: Default::default(),
      named_chunk_groups: Default::default(),
      entry_module_identifiers: IdentifierSet::default(),
      used_symbol_ref: HashSet::default(),
      optimize_analyze_result_map: IdentifierMap::default(),
      bailout_module_identifiers: IdentifierMap::default(),

      code_generation_results: Default::default(),
      code_generated_modules: Default::default(),
      cache,
      code_splitting_cache: Default::default(),
      hash: None,
      lazy_visit_modules: Default::default(),
      used_chunk_ids: Default::default(),

      file_dependencies: Default::default(),
      context_dependencies: Default::default(),
      missing_dependencies: Default::default(),
      build_dependencies: Default::default(),
      side_effects_free_modules: IdentifierSet::default(),
      module_item_map: IdentifierMap::default(),
      include_module_ids: IdentifierSet::default(),

      factorize_queue: None,
      build_queue: None,
      add_queue: None,
      process_dependencies_queue: None,
      build_time_execution_queue: None,
    }
  }

  pub fn get_module_graph(&self) -> &ModuleGraph {
    &self.module_graph
  }

  pub fn get_module_graph_mut(&mut self) -> &mut ModuleGraph {
    &mut self.module_graph
  }

  pub fn get_entry_runtime(&self, name: &String, options: Option<&EntryOptions>) -> RuntimeSpec {
    let (_, runtime) = if let Some(options) = options {
      ((), options.runtime.as_ref())
    } else {
      match self.entries.get(name) {
        Some(entry) => ((), entry.options.runtime.as_ref()),
        None => return RuntimeSpec::from_iter([Arc::from(name.as_str())]),
      }
    };
    // TODO: depend on https://github.com/webpack/webpack/blob/1f99ad6367f2b8a6ef17cce0e058f7a67fb7db18/lib/util/runtime.js#L33, we don't have that field now
    runtime
      .or(Some(name))
      .map(|runtime| RuntimeSpec::from_iter([Arc::from(runtime.as_ref())]))
      .unwrap_or_default()
  }

  pub fn add_entry(&mut self, entry: BoxDependency, options: EntryOptions) -> Result<()> {
    let entry_id = *entry.id();
    self.get_module_graph_mut().add_dependency(entry);
    if let Some(name) = options.name.clone() {
      if let Some(data) = self.entries.get_mut(&name) {
        data.dependencies.push(entry_id);
        data.options.merge(options)?;
      } else {
        let data = EntryData {
          dependencies: vec![entry_id],
          include_dependencies: vec![],
          options,
        };
        self.entries.insert(name, data);
      }
    } else {
      self.global_entry.dependencies.push(entry_id);
    }
    Ok(())
  }

  pub async fn add_include(&mut self, entry: BoxDependency, options: EntryOptions) -> Result<()> {
    let entry_id = *entry.id();
    self.get_module_graph_mut().add_dependency(entry);
    if let Some(name) = options.name.clone() {
      if let Some(data) = self.entries.get_mut(&name) {
        data.include_dependencies.push(entry_id);
      } else {
        let data = EntryData {
          dependencies: vec![],
          include_dependencies: vec![entry_id],
          options,
        };
        self.entries.insert(name, data);
      }
    } else {
      self.global_entry.include_dependencies.push(entry_id);
    }
    update_module_graph(
      self,
      vec![MakeParam::ForceBuildDeps(HashSet::from_iter([(
        entry_id, None,
      )]))],
    )
    .await
  }

  pub fn update_asset(
    &mut self,
    filename: &str,
    updater: impl FnOnce(BoxSource, AssetInfo) -> Result<(BoxSource, AssetInfo)>,
  ) -> Result<()> {
    // Safety: we don't move anything from compilation
    let assets = &mut self.assets;

    let (new_source, new_info) = match assets.remove(filename) {
      Some(CompilationAsset {
        source: Some(source),
        info,
      }) => updater(source, info)?,
      _ => {
        return Err(error!(
          "Called Compilation.updateAsset for not existing filename {filename}"
        ))
      }
    };
    self.emit_asset(
      filename.to_owned(),
      CompilationAsset {
        source: Some(new_source),
        info: new_info,
      },
    );
    Ok(())
  }

  pub fn emit_asset(&mut self, filename: String, asset: CompilationAsset) {
    tracing::trace!("Emit asset {}", filename);
    if let Some(mut original) = self.assets.remove(&filename)
      && let Some(original_source) = &original.source
      && let Some(asset_source) = asset.get_source()
    {
      let is_source_equal = is_source_equal(original_source, asset_source);
      if !is_source_equal {
        tracing::error!(
          "Emit Duplicate Filename({}), is_source_equal: {:?}",
          filename,
          is_source_equal
        );
        self.push_diagnostic(
          error!(
            "Conflict: Multiple assets emit different content to the same filename {}{}",
            filename,
            // TODO: source file name
            ""
          )
          .into(),
        );
        self.assets.insert(filename, asset);
        return;
      }
      original.info = asset.info;
      self.assets.insert(filename, original);
    } else {
      self.assets.insert(filename, asset);
    }
  }

  pub fn delete_asset(&mut self, filename: &str) {
    if let Some(asset) = self.assets.remove(filename) {
      if let Some(source_map) = asset.info.related.source_map {
        self.delete_asset(&source_map);
      }
      self.chunk_by_ukey.iter_mut().for_each(|(_, chunk)| {
        chunk.files.remove(filename);
        chunk.auxiliary_files.remove(filename);
      });
    }
  }

  pub fn rename_asset(&mut self, filename: &str, new_name: String) {
    if let Some(asset) = self.assets.remove(filename) {
      self.assets.insert(new_name.clone(), asset);
      self.chunk_by_ukey.iter_mut().for_each(|(_, chunk)| {
        if chunk.files.remove(filename) {
          chunk.files.insert(new_name.clone());
        }

        if chunk.auxiliary_files.remove(filename) {
          chunk.auxiliary_files.insert(new_name.clone());
        }
      });
    }
  }

  pub fn assets(&self) -> &CompilationAssets {
    &self.assets
  }

  pub fn assets_mut(&mut self) -> &mut CompilationAssets {
    &mut self.assets
  }

  pub fn entrypoints(&self) -> &IndexMap<String, ChunkGroupUkey> {
    &self.entrypoints
  }

  pub fn push_diagnostic(&mut self, diagnostic: Diagnostic) {
    self.diagnostics.push(diagnostic);
  }

  pub fn push_batch_diagnostic(&mut self, diagnostics: Vec<Diagnostic>) {
    self.diagnostics.extend(diagnostics);
  }

  pub fn get_errors(&self) -> impl Iterator<Item = &Diagnostic> {
    self
      .diagnostics
      .iter()
      .filter(|d| matches!(d.severity(), Severity::Error))
  }

  /// Get sorted errors based on the factors as follows in order:
  /// - module identifier
  /// - error offset
  /// Rspack assumes for each offset, there is only one error.
  /// However, when it comes to the case that there are multiple errors with the same offset,
  /// the order of these errors will not be guaranteed.
  pub fn get_errors_sorted(&self) -> impl Iterator<Item = &Diagnostic> {
    let get_offset = |d: &dyn rspack_error::miette::Diagnostic| {
      d.labels()
        .and_then(|mut l| l.next())
        .map(|l| l.offset())
        .unwrap_or_default()
    };
    self.get_errors().sorted_by(
      |a, b| match a.module_identifier().cmp(&b.module_identifier()) {
        std::cmp::Ordering::Equal => get_offset(a.as_ref()).cmp(&get_offset(b.as_ref())),
        other => other,
      },
    )
  }

  pub fn get_warnings(&self) -> impl Iterator<Item = &Diagnostic> {
    self
      .diagnostics
      .iter()
      .filter(|d| matches!(d.severity(), Severity::Warn))
  }

  /// Get sorted warnings based on the factors as follows in order:
  /// - module identifier
  /// - error offset
  /// Rspack assumes for each offset, there is only one error.
  /// However, when it comes to the case that there are multiple errors with the same offset,
  /// the order of these errors will not be guaranteed.
  pub fn get_warnings_sorted(&self) -> impl Iterator<Item = &Diagnostic> {
    let get_offset = |d: &dyn rspack_error::miette::Diagnostic| {
      d.labels()
        .and_then(|mut l| l.next())
        .map(|l| l.offset())
        .unwrap_or_default()
    };
    self.get_warnings().sorted_by(
      |a, b| match a.module_identifier().cmp(&b.module_identifier()) {
        std::cmp::Ordering::Equal => get_offset(a.as_ref()).cmp(&get_offset(b.as_ref())),
        other => other,
      },
    )
  }

  pub fn get_logging(&self) -> &CompilationLogging {
    &self.logging
  }

  pub fn get_stats(&self) -> Stats {
    Stats::new(self)
  }

  pub fn add_named_chunk(
    name: String,
    chunk_by_ukey: &mut ChunkByUkey,
    named_chunks: &mut HashMap<String, ChunkUkey>,
  ) -> ChunkUkey {
    let existed_chunk_ukey = named_chunks.get(&name);
    if let Some(chunk_ukey) = existed_chunk_ukey {
      assert!(chunk_by_ukey.contains(chunk_ukey));
      *chunk_ukey
    } else {
      let chunk = Chunk::new(Some(name.clone()), ChunkKind::Normal);
      let ukey = chunk.ukey;
      named_chunks.insert(name, chunk.ukey);
      chunk_by_ukey.entry(ukey).or_insert_with(|| chunk);
      ukey
    }
  }

  pub fn add_chunk(chunk_by_ukey: &mut ChunkByUkey) -> ChunkUkey {
    let chunk = Chunk::new(None, ChunkKind::Normal);
    let ukey = chunk.ukey;
    chunk_by_ukey.add(chunk);
    ukey
  }

  #[instrument(name = "compilation:make", skip_all)]
  pub async fn make(&mut self, mut params: Vec<MakeParam>) -> Result<()> {
    let make_failed_module =
      MakeParam::ForceBuildModules(std::mem::take(&mut self.make_failed_module));
    let make_failed_dependencies =
      MakeParam::ForceBuildDeps(std::mem::take(&mut self.make_failed_dependencies));

    params.push(make_failed_module);
    params.push(make_failed_dependencies);
    update_module_graph(self, params).await
  }

  pub async fn rebuild_module(
    &mut self,
    module_identifiers: HashSet<ModuleIdentifier>,
  ) -> Result<Vec<&BoxModule>> {
    for id in &module_identifiers {
      self.cache.build_module_occasion.remove_cache(id);
    }

    update_module_graph(
      self,
      vec![MakeParam::ForceBuildModules(module_identifiers.clone())],
    )
    .await?;

    if self.options.is_new_tree_shaking() {
      let logger = self.get_logger("rspack.Compilation");
      let start = logger.time("finish module");
      self.finish(self.plugin_driver.clone()).await?;
      logger.time_end(start);
    }

    let module_graph = self.get_module_graph();
    Ok(
      module_identifiers
        .into_iter()
        .filter_map(|id| module_graph.module_by_identifier(&id))
        .collect::<Vec<_>>(),
    )
  }

  #[instrument(name = "compilation:code_generation", skip(self))]
  fn code_generation(&mut self) -> Result<()> {
    let logger = self.get_logger("rspack.Compilation");
    let mut codegen_cache_counter = match self.options.cache {
      CacheOptions::Disabled => None,
      _ => Some(logger.cache("module code generation cache")),
    };

    fn run_iteration(
      compilation: &mut Compilation,
      codegen_cache_counter: &mut Option<CacheCount>,
      filter_op: impl Fn(&(&ModuleIdentifier, &Box<dyn Module>)) -> bool + Sync + Send,
    ) -> Result<()> {
      // If the runtime optimization is not opt out, a module codegen should be executed for each runtime.
      // Else, share same codegen result for all runtimes.
      let used_exports_optimization = compilation.options.is_new_tree_shaking()
        && compilation.options.optimization.used_exports.is_true();
      let results = compilation.code_generation_modules(
        codegen_cache_counter,
        used_exports_optimization,
        compilation
          .get_module_graph()
          .modules()
          .iter()
          .filter(filter_op)
          .map(|(id, _)| *id)
          .collect::<Vec<_>>()
          .into_par_iter(),
      )?;

      results.iter().for_each(|module_identifier| {
        compilation
          .code_generated_modules
          .insert(*module_identifier);
      });

      Ok(())
    }

    // FIXME:
    // Webpack may modify the moduleGraph in module.getExportsType()
    // and it is widely called after compilation.finish()
    // so add this method to trigger moduleGraph modification and
    // then make sure that moduleGraph is immutable
    prepare_get_exports_type(self.get_module_graph_mut());

    run_iteration(self, &mut codegen_cache_counter, |(_, module)| {
      module.get_code_generation_dependencies().is_none()
    })?;

    run_iteration(self, &mut codegen_cache_counter, |(_, module)| {
      module.get_code_generation_dependencies().is_some()
    })?;

    if let Some(counter) = codegen_cache_counter {
      logger.cache_end(counter);
    }

    Ok(())
  }

  pub(crate) fn code_generation_modules(
    &mut self,
    codegen_cache_counter: &mut Option<CacheCount>,
    used_exports_optimization: bool,
    modules: impl ParallelIterator<Item = ModuleIdentifier>,
  ) -> Result<Vec<ModuleIdentifier>> {
    let chunk_graph = &self.chunk_graph;
    #[allow(clippy::type_complexity)]
    let results = modules
      .filter_map(|module_identifier| {
        let runtimes = chunk_graph.get_module_runtimes(module_identifier, &self.chunk_by_ukey);
        if runtimes.is_empty() {
          return None;
        }

        let module = self
          .get_module_graph()
          .module_by_identifier(&module_identifier)
          .expect("module should exist");
        let res = self
          .cache
          .code_generate_occasion
          .use_cache(module, runtimes, self, |module, runtimes| {
            let take_length = if used_exports_optimization {
              runtimes.len()
            } else {
              // Only codegen once
              1
            };
            let mut codegen_list = vec![];
            for runtime in runtimes.into_values().take(take_length) {
              codegen_list.push((module.code_generation(self, Some(&runtime), None)?, runtime));
            }
            Ok(codegen_list)
          })
          .map(|(result, from_cache)| (module_identifier, result, from_cache));
        Some(res)
      })
      .collect::<Result<Vec<_>>>()?;
    let results = results
      .into_iter()
      .map(|(module_identifier, item, from_cache)| {
        item.into_iter().for_each(|(result, runtime)| {
          if let Some(counter) = codegen_cache_counter {
            if from_cache {
              counter.hit();
            } else {
              counter.miss();
            }
          }

          let runtimes = chunk_graph.get_module_runtimes(module_identifier, &self.chunk_by_ukey);
          let result_id = result.id;
          self
            .code_generation_results
            .module_generation_result_map
            .insert(result.id, result);
          if used_exports_optimization {
            self
              .code_generation_results
              .add(module_identifier, runtime, result_id);
          } else {
            for runtime in runtimes.into_values() {
              self
                .code_generation_results
                .add(module_identifier, runtime, result_id);
            }
          }
        });
        module_identifier
      });

    Ok(results.collect())
  }

  #[instrument(name = "compilation::create_module_assets", skip_all)]
  async fn create_module_assets(&mut self, _plugin_driver: SharedPluginDriver) {
    let mut temp = vec![];
    for (module_identifier, module) in self.get_module_graph().modules() {
      if let Some(build_info) = module.build_info() {
        for asset in build_info.asset_filenames.iter() {
          for chunk in self
            .chunk_graph
            .get_module_chunks(*module_identifier)
            .iter()
          {
            temp.push((*chunk, asset.clone()))
          }
          // already emitted asset by loader, so no need to re emit here
        }
      }
    }

    for (chunk, asset) in temp {
      let chunk = self.chunk_by_ukey.expect_get_mut(&chunk);
      chunk.auxiliary_files.insert(asset);
    }
  }

  #[instrument(skip_all)]
  async fn create_chunk_assets(&mut self, plugin_driver: SharedPluginDriver) {
    let results = self
      .chunk_by_ukey
      .values()
      .map(|chunk| async {
        let manifest_result = plugin_driver
          .render_manifest(RenderManifestArgs {
            chunk_ukey: chunk.ukey,
            compilation: self,
          })
          .await;

        if let Ok(manifest) = &manifest_result {
          tracing::debug!(
            "For Chunk({:?}), collected assets: {:?}",
            chunk.id,
            manifest
              .inner
              .iter()
              .map(|m| m.filename())
              .collect::<Vec<_>>()
          );
        };

        (chunk.ukey, manifest_result)
      })
      .collect::<FuturesResults<_>>();

    let chunk_ukey_and_manifest = results.into_inner();

    for (chunk_ukey, manifest_result) in chunk_ukey_and_manifest.into_iter() {
      let (manifests, diagnostics) = manifest_result
        .expect("We should return this error rathen expect")
        .split_into_parts();

      self.push_batch_diagnostic(diagnostics);

      for file_manifest in manifests {
        let filename = file_manifest.filename().to_string();

        let current_chunk = self.chunk_by_ukey.expect_get_mut(&chunk_ukey);

        if file_manifest.auxiliary {
          current_chunk.auxiliary_files.insert(filename.clone());
        } else {
          current_chunk.files.insert(filename.clone());
        }

        self.emit_asset(
          filename.clone(),
          CompilationAsset::new(
            Some(CachedSource::new(file_manifest.source).boxed()),
            file_manifest.info,
          ),
        );

        _ = self
          .chunk_asset(chunk_ukey, filename, plugin_driver.clone())
          .await;
      }
      //
      // .into_iter()
      // .for_each(|file_manifest| {
      // });
    }
    // .for_each(|(chunk_ukey, manifest)| {
    // })
  }

  #[instrument(name = "compilation:after_process_asssets", skip_all)]
  async fn after_process_assets(&mut self, plugin_driver: SharedPluginDriver) -> Result<()> {
    plugin_driver
      .compilation_hooks
      .after_process_assets
      .call(self)
      .await
  }

  #[instrument(
    name = "compilation:chunk_asset",
    skip(self, plugin_driver, chunk_ukey)
  )]
  async fn chunk_asset(
    &mut self,
    chunk_ukey: ChunkUkey,
    mut filename: String,
    plugin_driver: SharedPluginDriver,
  ) -> Result<()> {
    let current_chunk = self.chunk_by_ukey.expect_get_mut(&chunk_ukey);
    plugin_driver
      .compilation_hooks
      .chunk_asset
      .call(current_chunk, &mut filename)
      .await?;
    Ok(())
  }

  pub async fn optimize_dependency(
    &mut self,
  ) -> Result<TWithDiagnosticArray<OptimizeDependencyResult>> {
    let logger = self.get_logger("rspack.Compilation");
    let start = logger.time("optimize dependencies");
    let result = optimizer::CodeSizeOptimizer::new(self).run().await;
    logger.time_end(start);
    result
  }

  pub async fn done(&mut self, plugin_driver: SharedPluginDriver) -> Result<()> {
    let stats = &mut Stats::new(self);
    plugin_driver.done(stats).await?;
    Ok(())
  }

  pub fn entry_modules(&self) -> impl Iterator<Item = ModuleIdentifier> {
    self.entry_module_identifiers.clone().into_iter()
  }

  pub fn entrypoint_by_name(&self, name: &str) -> &Entrypoint {
    let ukey = self.entrypoints.get(name).expect("entrypoint not found");
    self.chunk_group_by_ukey.expect_get(ukey)
  }

  #[instrument(name = "compilation:finish", skip_all)]
  pub async fn finish(&mut self, plugin_driver: SharedPluginDriver) -> Result<()> {
    let logger = self.get_logger("rspack.Compilation");
    let start = logger.time("finish modules");
    plugin_driver
      .compilation_hooks
      .finish_modules
      .call(self)
      .await?;
    logger.time_end(start);

    Ok(())
  }

  #[instrument(name = "compilation:seal", skip_all)]
  pub async fn seal(&mut self, plugin_driver: SharedPluginDriver) -> Result<()> {
    let logger = self.get_logger("rspack.Compilation");

    // https://github.com/webpack/webpack/blob/main/lib/Compilation.js#L2809
    plugin_driver.seal(self)?;

    let start = logger.time("optimize dependencies");
    // https://github.com/webpack/webpack/blob/d15c73469fd71cf98734685225250148b68ddc79/lib/Compilation.js#L2812-L2814
    while plugin_driver.optimize_dependencies(self).await?.is_some() {}
    logger.time_end(start);

    // if self.options.is_new_tree_shaking() {
    //   // let filter = |item: &str| ["config-provider"].iter().any(|pat| item.contains(pat));
    //   // debug_all_exports_info!(&self.module_graph, filter);
    // }
    let start = logger.time("create chunks");
    use_code_splitting_cache(self, |compilation| async {
      build_chunk_graph(compilation)?;
      while matches!(
        plugin_driver
          .compilation_hooks
          .optimize_modules
          .call(compilation)
          .await?,
        Some(true)
      ) {}
      plugin_driver
        .compilation_hooks
        .after_optimize_modules
        .call(compilation)
        .await?;
      plugin_driver.optimize_chunks(compilation).await?;
      Ok(compilation)
    })
    .await?;
    logger.time_end(start);

    let start = logger.time("optimize");
    plugin_driver
      .compilation_hooks
      .optimize_tree
      .call(self)
      .await?;

    plugin_driver
      .compilation_hooks
      .optimize_chunk_modules
      .call(self)
      .await?;

    logger.time_end(start);

    let start = logger.time("module ids");
    plugin_driver.module_ids(self)?;
    logger.time_end(start);

    let start = logger.time("chunk ids");
    plugin_driver.chunk_ids(self)?;
    logger.time_end(start);

    self.assign_runtime_ids();

    let start = logger.time("optimize code generation");
    plugin_driver.optimize_code_generation(self).await?;
    logger.time_end(start);

    let start = logger.time("code generation");
    self.code_generation()?;
    logger.time_end(start);

    let start = logger.time("runtime requirements");
    self
      .process_runtime_requirements(
        self
          .get_module_graph()
          .modules()
          .keys()
          .copied()
          .collect::<Vec<_>>(),
        self
          .chunk_by_ukey
          .keys()
          .copied()
          .collect::<Vec<_>>()
          .into_iter(),
        self.get_chunk_graph_entries().into_iter(),
        plugin_driver.clone(),
      )
      .await?;
    logger.time_end(start);

    let start = logger.time("hashing");
    self.create_hash(plugin_driver.clone()).await?;
    logger.time_end(start);

    let start = logger.time("create module assets");
    self.create_module_assets(plugin_driver.clone()).await;
    logger.time_end(start);

    let start = logger.time("create chunk assets");
    self.create_chunk_assets(plugin_driver.clone()).await;
    logger.time_end(start);

    let start = logger.time("process assets");
    plugin_driver
      .compilation_hooks
      .process_assets
      .call(self)
      .await?;
    logger.time_end(start);

    let start = logger.time("after process assets");
    self.after_process_assets(plugin_driver).await?;
    logger.time_end(start);

    Ok(())
  }

  pub fn assign_runtime_ids(&mut self) {
    fn process_entrypoint(
      entrypoint_ukey: &ChunkGroupUkey,
      chunk_group_by_ukey: &ChunkGroupByUkey,
      chunk_by_ukey: &ChunkByUkey,
      chunk_graph: &mut ChunkGraph,
    ) {
      let entrypoint = chunk_group_by_ukey.expect_get(entrypoint_ukey);
      let runtime = entrypoint
        .kind
        .get_entry_options()
        .and_then(|o| o.name.clone())
        .or(entrypoint.name().map(|n| n.to_string()));
      if let (Some(runtime), Some(chunk)) = (
        runtime,
        get_chunk_from_ukey(
          &entrypoint.get_runtime_chunk(chunk_group_by_ukey),
          chunk_by_ukey,
        ),
      ) {
        chunk_graph.set_runtime_id(runtime, chunk.id.clone());
      }
    }
    for i in self.entrypoints.iter() {
      process_entrypoint(
        i.1,
        &self.chunk_group_by_ukey,
        &self.chunk_by_ukey,
        &mut self.chunk_graph,
      )
    }
    for i in self.async_entrypoints.iter() {
      process_entrypoint(
        i,
        &self.chunk_group_by_ukey,
        &self.chunk_by_ukey,
        &mut self.chunk_graph,
      )
    }
  }

  pub fn get_chunk_graph_entries(&self) -> HashSet<ChunkUkey> {
    let entries = self.entrypoints.values().map(|entrypoint_ukey| {
      let entrypoint = self.chunk_group_by_ukey.expect_get(entrypoint_ukey);
      entrypoint.get_runtime_chunk(&self.chunk_group_by_ukey)
    });
    let async_entries = self.async_entrypoints.iter().map(|entrypoint_ukey| {
      let entrypoint = self.chunk_group_by_ukey.expect_get(entrypoint_ukey);
      entrypoint.get_runtime_chunk(&self.chunk_group_by_ukey)
    });
    HashSet::from_iter(entries.chain(async_entries))
  }

  #[allow(clippy::unwrap_in_result)]
  pub(crate) async fn process_runtime_requirements(
    &mut self,
    modules: impl IntoParallelIterator<Item = ModuleIdentifier>,
    chunks: impl Iterator<Item = ChunkUkey>,
    chunk_graph_entries: impl Iterator<Item = ChunkUkey>,
    plugin_driver: SharedPluginDriver,
  ) -> Result<()> {
    let logger = self.get_logger("rspack.Compilation");
    let start = logger.time("runtime requirements.modules");
    let mut module_runtime_requirements = modules
      .into_par_iter()
      .filter_map(|module_identifier| {
        if self
          .chunk_graph
          .get_number_of_module_chunks(module_identifier)
          > 0
        {
          let mut module_runtime_requirements: Vec<(RuntimeSpec, RuntimeGlobals)> = vec![];
          for runtime in self
            .chunk_graph
            .get_module_runtimes(module_identifier, &self.chunk_by_ukey)
            .values()
          {
            let runtime_requirements = self
              .code_generation_results
              .get_runtime_requirements(&module_identifier, Some(runtime));
            module_runtime_requirements.push((runtime.clone(), runtime_requirements));
          }
          return Some((module_identifier, module_runtime_requirements));
        }
        None
      })
      .collect::<Vec<_>>();

    for (module_identifier, runtime_requirements) in module_runtime_requirements.iter_mut() {
      for (runtime, requirements) in runtime_requirements.iter_mut() {
        let mut runtime_requirements_mut = *requirements;
        let mut runtime_requirements;
        while !runtime_requirements_mut.is_empty() {
          requirements.insert(runtime_requirements_mut);
          runtime_requirements = runtime_requirements_mut;
          runtime_requirements_mut = RuntimeGlobals::default();
          plugin_driver.runtime_requirement_in_module(&mut AdditionalModuleRequirementsArgs {
            compilation: self,
            module_identifier,
            runtime_requirements: &runtime_requirements,
            runtime_requirements_mut: &mut runtime_requirements_mut,
          })?;
          runtime_requirements_mut = runtime_requirements_mut
            .difference(requirements.intersection(runtime_requirements_mut));
        }
        self.chunk_graph.add_module_runtime_requirements(
          *module_identifier,
          runtime,
          std::mem::take(requirements),
        )
      }
    }
    logger.time_end(start);

    let start = logger.time("runtime requirements.chunks");
    let mut chunk_requirements = HashMap::default();
    for chunk_ukey in chunks {
      let mut set = RuntimeGlobals::default();
      for module in self
        .chunk_graph
        .get_chunk_modules(&chunk_ukey, self.get_module_graph())
      {
        let chunk = self.chunk_by_ukey.expect_get(&chunk_ukey);
        if let Some(runtime_requirements) = self
          .chunk_graph
          .get_module_runtime_requirements(module.identifier(), &chunk.runtime)
        {
          set.insert(*runtime_requirements);
        }
      }
      chunk_requirements.insert(chunk_ukey, set);
    }
    for (chunk_ukey, set) in chunk_requirements.iter_mut() {
      plugin_driver
        .additional_chunk_runtime_requirements(&mut AdditionalChunkRuntimeRequirementsArgs {
          compilation: self,
          chunk: chunk_ukey,
          runtime_requirements: set,
        })
        .await?;

      self
        .chunk_graph
        .add_chunk_runtime_requirements(chunk_ukey, std::mem::take(set));
    }
    logger.time_end(start);

    let start = logger.time("runtime requirements.entries");
    for entry_ukey in chunk_graph_entries {
      let entry = self.chunk_by_ukey.expect_get(&entry_ukey);
      let mut set = RuntimeGlobals::default();
      for chunk_ukey in entry
        .get_all_referenced_chunks(&self.chunk_group_by_ukey)
        .iter()
      {
        let runtime_requirements = self.chunk_graph.get_chunk_runtime_requirements(chunk_ukey);
        set.insert(*runtime_requirements);
      }

      plugin_driver
        .additional_tree_runtime_requirements(&mut AdditionalChunkRuntimeRequirementsArgs {
          compilation: self,
          chunk: &entry_ukey,
          runtime_requirements: &mut set,
        })
        .await?;

      let requirements = &mut set;
      let mut runtime_requirements_mut = *requirements;
      let mut runtime_requirements;
      while !runtime_requirements_mut.is_empty() {
        requirements.insert(runtime_requirements_mut);
        runtime_requirements = runtime_requirements_mut;
        runtime_requirements_mut = RuntimeGlobals::default();
        plugin_driver
          .runtime_requirements_in_tree(&mut RuntimeRequirementsInTreeArgs {
            compilation: self,
            chunk: &entry_ukey,
            runtime_requirements: &runtime_requirements,
            runtime_requirements_mut: &mut runtime_requirements_mut,
          })
          .await?;
        runtime_requirements_mut =
          runtime_requirements_mut.difference(requirements.intersection(runtime_requirements_mut));
      }

      self
        .chunk_graph
        .add_tree_runtime_requirements(&entry_ukey, set);
    }

    // NOTE: webpack runs hooks.runtime_module in compilation.add_runtime_module
    // and overwrite the runtime_module.generate() to get new source in create_chunk_assets
    // this needs full runtime requirements, so run hooks.runtime_module after runtime_requirements_in_tree
    for mut entry_ukey in self.get_chunk_graph_entries() {
      let runtime_module_ids: Vec<_> = self
        .chunk_graph
        .get_chunk_runtime_modules_iterable(&entry_ukey)
        .copied()
        .collect();
      for mut runtime_module_id in runtime_module_ids {
        self
          .plugin_driver
          .clone()
          .compilation_hooks
          .runtime_module
          .call(self, &mut runtime_module_id, &mut entry_ukey)
          .await?;
      }
    }

    logger.time_end(start);
    Ok(())
  }

  #[instrument(name = "compilation:create_hash", skip_all)]
  pub async fn create_hash(&mut self, plugin_driver: SharedPluginDriver) -> Result<()> {
    let logger = self.get_logger("rspack.Compilation");
    let mut compilation_hasher = RspackHash::from(&self.options.output);
    // TODO: runtimeChunk referencedBy for correct hashing AsyncEntrypoint
    let runtime_chunk_ukeys = self.get_chunk_graph_entries();

    fn try_process_chunk_hash_results(
      compilation: &mut Compilation,
      chunk_hash_results: Vec<Result<(ChunkUkey, (RspackHashDigest, ChunkContentHash))>>,
    ) -> Result<()> {
      for hash_result in chunk_hash_results {
        let (chunk_ukey, (chunk_hash, content_hash)) = hash_result?;
        if let Some(chunk) = get_mut_chunk_from_ukey(&chunk_ukey, &mut compilation.chunk_by_ukey) {
          chunk.rendered_hash = Some(
            chunk_hash
              .rendered(compilation.options.output.hash_digest_length)
              .into(),
          );
          chunk.hash = Some(chunk_hash);
          chunk.content_hash = content_hash;
        }
      }
      Ok(())
    }

    let start = logger.time("hashing: hash chunks");
    let other_chunk_hash_results: Vec<Result<(ChunkUkey, (RspackHashDigest, ChunkContentHash))>> =
      self
        .chunk_by_ukey
        .keys()
        .filter(|key| !runtime_chunk_ukeys.contains(key))
        .map(|chunk| async {
          let hash_result = self.process_chunk_hash(*chunk, &plugin_driver).await?;
          Ok((*chunk, hash_result))
        })
        .collect::<FuturesResults<_>>()
        .into_inner();

    try_process_chunk_hash_results(self, other_chunk_hash_results)?;
    logger.time_end(start);

    // runtime chunks should be hashed after all other chunks
    let start = logger.time("hashing: hash runtime chunks");
    self.create_runtime_module_hash()?;

    let runtime_chunk_hash_results: Vec<Result<(ChunkUkey, (RspackHashDigest, ChunkContentHash))>> =
      runtime_chunk_ukeys
        .iter()
        .map(|chunk| async {
          let hash_result = self.process_chunk_hash(*chunk, &plugin_driver).await?;
          Ok((*chunk, hash_result))
        })
        .collect::<FuturesResults<_>>()
        .into_inner();
    try_process_chunk_hash_results(self, runtime_chunk_hash_results)?;
    logger.time_end(start);

    self
      .chunk_by_ukey
      .values()
      .sorted_unstable_by_key(|chunk| chunk.ukey)
      .filter_map(|chunk| chunk.hash.as_ref())
      .for_each(|hash| {
        hash.hash(&mut compilation_hasher);
      });
    self.hot_index.hash(&mut compilation_hasher);
    self.hash = Some(compilation_hasher.digest(&self.options.output.hash_digest));

    // here omit re-create full hash runtime module hash, only update compilation hash to runtime chunk content hash
    let start = logger.time("hashing: process full hash chunks");
    self.chunk_by_ukey.values_mut().for_each(|chunk| {
      if runtime_chunk_ukeys.contains(&chunk.ukey) {
        if let Some(chunk_hash) = &mut chunk.hash {
          let mut hasher = RspackHash::from(&self.options.output);
          chunk_hash.hash(&mut hasher);
          self.hash.hash(&mut hasher);
          *chunk_hash = hasher.digest(&self.options.output.hash_digest);
          chunk.rendered_hash = Some(
            chunk_hash
              .rendered(self.options.output.hash_digest_length)
              .into(),
          );
        }
        if let Some(content_hash) = chunk.content_hash.get_mut(&SourceType::JavaScript) {
          let mut hasher = RspackHash::from(&self.options.output);
          content_hash.hash(&mut hasher);
          self.hash.hash(&mut hasher);
          *content_hash = hasher.digest(&self.options.output.hash_digest);
        }
      }
    });
    logger.time_end(start);
    Ok(())
  }

  async fn process_chunk_hash(
    &self,
    chunk_ukey: ChunkUkey,
    plugin_driver: &SharedPluginDriver,
  ) -> Result<(RspackHashDigest, ChunkContentHash)> {
    let mut hasher = RspackHash::from(&self.options.output);
    if let Some(chunk) = get_chunk_from_ukey(&chunk_ukey, &self.chunk_by_ukey) {
      chunk.update_hash(&mut hasher, self);
    }
    plugin_driver
      .chunk_hash(&mut ChunkHashArgs {
        chunk_ukey,
        compilation: self,
        hasher: &mut hasher,
      })
      .await?;
    let chunk_hash = hasher.digest(&self.options.output.hash_digest);

    let content_hash = plugin_driver
      .content_hash(&ContentHashArgs {
        chunk_ukey,
        compilation: self,
      })
      .await?;

    Ok((chunk_hash, content_hash))
  }

  // #[instrument(name = "compilation:create_module_hash", skip_all)]
  // pub fn create_module_hash(&mut self) {
  //   let module_hash_map: HashMap<ModuleIdentifier, u64> = self
  //     .get_module_graph()
  //     .module_identifier_to_module
  //     .par_iter()
  //     .map(|(identifier, module)| {
  //       let mut hasher = RspackHash::new();
  //       module.hash(&mut hasher);
  //       (*identifier, hasher.finish())
  //     })
  //     .collect();

  //   for (identifier, hash) in module_hash_map {
  //     for runtime in self
  //       .chunk_graph
  //       .get_module_runtimes(identifier, &self.chunk_by_ukey)
  //       .values()
  //     {
  //       self
  //         .chunk_graph
  //         .set_module_hashes(identifier, runtime, hash);
  //     }
  //   }
  // }

  #[instrument(name = "compilation:create_runtime_module_hash", skip_all)]
  pub fn create_runtime_module_hash(&mut self) -> Result<()> {
    self.runtime_module_code_generation_results = self
      .runtime_modules
      .par_iter()
      .map(
        |(identifier, module)| -> Result<(Identifier, (RspackHashDigest, BoxSource))> {
          let source = module.generate_with_custom(self)?;
          let mut hasher = RspackHash::from(&self.options.output);
          module.identifier().hash(&mut hasher);
          source.source().hash(&mut hasher);
          Ok((
            *identifier,
            (hasher.digest(&self.options.output.hash_digest), source),
          ))
        },
      )
      .collect::<Result<IdentifierMap<(RspackHashDigest, BoxSource)>>>()?;
    Ok(())
  }

  pub async fn add_runtime_module(
    &mut self,
    chunk_ukey: &ChunkUkey,
    mut module: Box<dyn RuntimeModule>,
  ) -> Result<()> {
    // add chunk runtime to prefix module identifier to avoid multiple entry runtime modules conflict
    let chunk = self.chunk_by_ukey.expect_get(chunk_ukey);
    let runtime_module_identifier =
      ModuleIdentifier::from(format!("{:?}/{}", chunk.runtime, module.identifier()));
    module.attach(*chunk_ukey);
    self.chunk_graph.add_module(runtime_module_identifier);
    self
      .chunk_graph
      .connect_chunk_and_module(*chunk_ukey, runtime_module_identifier);
    self
      .chunk_graph
      .connect_chunk_and_runtime_module(*chunk_ukey, runtime_module_identifier);

    self
      .runtime_modules
      .insert(runtime_module_identifier, module);

    Ok(())
  }

  pub fn get_hash(&self) -> Option<&str> {
    self
      .hash
      .as_ref()
      .map(|hash| hash.rendered(self.options.output.hash_digest_length))
  }

  pub fn get_path<'b, 'a: 'b, F: LocalFilenameFn>(
    &'a self,
    filename: &Filename<F>,
    mut data: PathData<'b>,
  ) -> Result<String, F::Error> {
    if data.hash.is_none() {
      data.hash = self.get_hash();
    }
    filename.render(data, None)
  }

  pub fn get_path_with_info<'b, 'a: 'b, F: LocalFilenameFn>(
    &'a self,
    filename: &Filename<F>,
    mut data: PathData<'b>,
  ) -> Result<(String, AssetInfo), F::Error> {
    let mut info = AssetInfo::default();
    if data.hash.is_none() {
      data.hash = self.get_hash();
    }
    let path = filename.render(data, Some(&mut info))?;
    Ok((path, info))
  }

  pub fn get_asset_path<F: LocalFilenameFn>(
    &self,
    filename: &Filename<F>,
    data: PathData,
  ) -> Result<String, F::Error> {
    filename.render(data, None)
  }

  pub fn get_asset_path_with_info<F: LocalFilenameFn>(
    &self,
    filename: &Filename<F>,
    data: PathData,
  ) -> Result<(String, AssetInfo), F::Error> {
    let mut info = AssetInfo::default();
    let path = filename.render(data, Some(&mut info))?;
    Ok((path, info))
  }

  pub fn get_logger(&self, name: impl Into<String>) -> CompilationLogger {
    CompilationLogger::new(name.into(), self.logging.clone())
  }

  pub fn set_dependency_factory(
    &mut self,
    dependency_type: DependencyType,
    module_factory: Arc<dyn ModuleFactory>,
  ) {
    self
      .dependency_factories
      .insert(dependency_type, module_factory);
  }

  pub fn get_dependency_factory(&self, dependency: &BoxDependency) -> Arc<dyn ModuleFactory> {
    let dependency_type = dependency.dependency_type();
    self
      .dependency_factories
      .get(&match dependency_type {
        DependencyType::EsmImport(_) => DependencyType::EsmImport(ErrorSpan::default()),
        DependencyType::EsmExport(_) => DependencyType::EsmExport(ErrorSpan::default()),
        _ => dependency_type.clone(),
      })
      .unwrap_or_else(|| {
        panic!(
          "No module factory available for dependency type: {}, resourceIdentifier: {:?}",
          dependency_type,
          dependency.resource_identifier()
        )
      })
      .clone()
  }
}

pub type CompilationAssets = HashMap<String, CompilationAsset>;

#[derive(Debug, Clone)]
pub struct CompilationAsset {
  pub source: Option<BoxSource>,
  pub info: AssetInfo,
}

impl From<BoxSource> for CompilationAsset {
  fn from(value: BoxSource) -> Self {
    Self::new(Some(value), Default::default())
  }
}

impl CompilationAsset {
  pub fn new(source: Option<BoxSource>, info: AssetInfo) -> Self {
    Self { source, info }
  }

  pub fn get_source(&self) -> Option<&BoxSource> {
    self.source.as_ref()
  }

  pub fn get_source_mut(&mut self) -> Option<&mut BoxSource> {
    self.source.as_mut()
  }

  pub fn set_source(&mut self, source: Option<BoxSource>) {
    self.source = source;
  }

  pub fn get_info(&self) -> &AssetInfo {
    &self.info
  }

  pub fn get_info_mut(&mut self) -> &mut AssetInfo {
    &mut self.info
  }

  pub fn set_info(&mut self, info: AssetInfo) {
    self.info = info;
  }
}

#[derive(Debug, Default, Clone)]
pub struct AssetInfo {
  /// if the asset can be long term cached forever (contains a hash)
  pub immutable: bool,
  /// whether the asset is minimized
  pub minimized: bool,
  /// the value(s) of the full hash used for this asset
  // pub full_hash:
  /// the value(s) of the chunk hash used for this asset
  pub chunk_hash: HashSet<String>,
  /// the value(s) of the module hash used for this asset
  // pub module_hash:
  /// the value(s) of the content hash used for this asset
  pub content_hash: HashSet<String>,
  /// when asset was created from a source file (potentially transformed), the original filename relative to compilation context
  // pub source_filename:
  /// size in bytes, only set after asset has been emitted
  // pub size: f64,
  /// when asset is only used for development and doesn't count towards user-facing assets
  pub development: bool,
  /// when asset ships data for updating an existing application (HMR)
  pub hot_module_replacement: bool,
  /// when asset is javascript and an ESM
  pub javascript_module: Option<bool>,
  /// related object to other assets, keyed by type of relation (only points from parent to child)
  pub related: AssetInfoRelated,
  /// the asset version, emit can be skipped when both filename and version are the same
  /// An empty string means no version, it will always emit
  pub version: String,
  pub source_filename: Option<String>,
}

impl AssetInfo {
  pub fn with_minimized(mut self, v: bool) -> Self {
    self.minimized = v;
    self
  }

  pub fn with_development(mut self, v: bool) -> Self {
    self.development = v;
    self
  }

  pub fn with_hot_module_replacement(mut self, v: bool) -> Self {
    self.hot_module_replacement = v;
    self
  }

  pub fn with_related(mut self, v: AssetInfoRelated) -> Self {
    self.related = v;
    self
  }

  pub fn with_content_hashes(mut self, v: HashSet<String>) -> Self {
    self.content_hash = v;
    self
  }

  pub fn with_version(mut self, v: String) -> Self {
    self.version = v;
    self
  }

  pub fn set_content_hash(&mut self, v: String) {
    self.content_hash.insert(v);
  }

  pub fn set_chunk_hash(&mut self, v: String) {
    self.chunk_hash.insert(v);
  }

  pub fn set_immutable(&mut self, v: bool) {
    self.immutable = v;
  }

  pub fn set_source_filename(&mut self, v: String) {
    self.source_filename = Some(v);
  }

  pub fn set_javascript_module(&mut self, v: bool) {
    self.javascript_module = Some(v);
  }
}

#[derive(Debug, Default, Clone)]
pub struct AssetInfoRelated {
  pub source_map: Option<String>,
}

/// level order, the impl is different from webpack, since we can't iterate a set and mutate it at
/// the same time.
pub fn assign_depths(
  assign_map: &mut HashMap<ModuleIdentifier, usize>,
  mg: &ModuleGraph,
  modules: Vec<&ModuleIdentifier>,
) {
  // https://github.com/webpack/webpack/blob/1f99ad6367f2b8a6ef17cce0e058f7a67fb7db18/lib/Compilation.js#L3720
  let mut q = VecDeque::new();
  for item in modules.iter() {
    q.push_back((**item, 0));
  }
  while let Some((id, depth)) = q.pop_front() {
    match assign_map.entry(id) {
      hash_map::Entry::Occupied(_) => {
        continue;
      }
      hash_map::Entry::Vacant(vac) => {
        vac.insert(depth);
      }
    };
    for con in mg.get_outgoing_connections(&id) {
      q.push_back((*con.module_identifier(), depth + 1));
    }
  }
}

pub fn assign_depth(
  assign_map: &mut HashMap<ModuleIdentifier, usize>,
  mg: &ModuleGraph,
  module_id: ModuleIdentifier,
) {
  // https://github.com/webpack/webpack/blob/1f99ad6367f2b8a6ef17cce0e058f7a67fb7db18/lib/Compilation.js#L3720
  let mut q = VecDeque::new();
  q.push_back(module_id);
  let mut depth;
  assign_map.insert(module_id, 0);
  let process_module =
    |m: ModuleIdentifier,
     depth: usize,
     q: &mut VecDeque<ModuleIdentifier>,
     assign_map: &mut HashMap<rspack_identifier::Identifier, usize>| {
      if !set_depth_if_lower(m, depth, assign_map) {
        return;
      }
      q.push_back(m);
    };
  while let Some(item) = q.pop_front() {
    depth = assign_map.get(&item).expect("should have depth") + 1;

    for con in mg.get_outgoing_connections(&item) {
      process_module(*con.module_identifier(), depth, &mut q, assign_map);
    }
  }
}

pub fn set_depth_if_lower(
  module_id: ModuleIdentifier,
  depth: usize,
  assign_map: &mut HashMap<ModuleIdentifier, usize>,
) -> bool {
  let Some(&cur_depth) = assign_map.get(&module_id) else {
    assign_map.insert(module_id, depth);
    return true;
  };
  if cur_depth > depth {
    assign_map.insert(module_id, depth);
    return true;
  }
  false
}
