use std::{
  collections::{hash_map, VecDeque},
  fmt::Debug,
  hash::{BuildHasherDefault, Hash},
  path::PathBuf,
  sync::{atomic::AtomicU32, Arc},
};

use dashmap::DashSet;
use indexmap::{IndexMap, IndexSet};
use itertools::Itertools;
use rayon::prelude::*;
use rspack_collections::{Identifiable, IdentifierDashMap, IdentifierMap, IdentifierSet, UkeySet};
use rspack_error::{error, miette::diagnostic, Diagnostic, DiagnosticExt, Result, Severity};
use rspack_fs::ReadableFileSystem;
use rspack_futures::FuturesResults;
use rspack_hash::{RspackHash, RspackHashDigest};
use rspack_hook::define_hook;
use rspack_sources::{BoxSource, CachedSource, SourceExt};
use rspack_util::itoa;
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet, FxHasher};
use tracing::instrument;

use super::{
  hmr::CompilationRecords,
  make::{make_module_graph, update_module_graph, MakeArtifact, MakeParam},
  module_executor::ModuleExecutor,
};
use crate::{
  build_chunk_graph::build_chunk_graph,
  cgm_hash_results::CgmHashResults,
  cgm_runtime_requirement_results::CgmRuntimeRequirementsResults,
  get_chunk_from_ukey, get_mut_chunk_from_ukey,
  incremental::{Incremental, IncrementalPasses, Mutation},
  is_source_equal,
  old_cache::{use_code_splitting_cache, Cache as OldCache, CodeSplittingCache},
  to_identifier, BoxDependency, BoxModule, CacheCount, CacheOptions, Chunk, ChunkByUkey,
  ChunkContentHash, ChunkGraph, ChunkGroupByUkey, ChunkGroupUkey, ChunkKind, ChunkUkey,
  CodeGenerationJob, CodeGenerationResult, CodeGenerationResults, CompilationLogger,
  CompilationLogging, CompilerOptions, DependencyId, DependencyType, Entry, EntryData,
  EntryOptions, EntryRuntime, Entrypoint, ExecuteModuleId, Filename, ImportVarMap, LocalFilenameFn,
  Logger, ModuleFactory, ModuleGraph, ModuleGraphPartial, ModuleIdentifier, PathData,
  ResolverFactory, RuntimeGlobals, RuntimeModule, RuntimeSpecMap, SharedPluginDriver, SourceType,
  Stats,
};

pub type BuildDependency = (
  DependencyId,
  Option<ModuleIdentifier>, /* parent module */
);

define_hook!(CompilationAddEntry: AsyncSeries(compilation: &mut Compilation, entry_name: Option<&str>));
define_hook!(CompilationBuildModule: AsyncSeries(module: &mut BoxModule));
define_hook!(CompilationStillValidModule: AsyncSeries(module: &mut BoxModule));
define_hook!(CompilationSucceedModule: AsyncSeries(module: &mut BoxModule));
define_hook!(CompilationExecuteModule:
  SyncSeries(module: &ModuleIdentifier, runtime_modules: &IdentifierSet, codegen_results: &CodeGenerationResults, execute_module_id: &ExecuteModuleId));
define_hook!(CompilationFinishModules: AsyncSeries(compilation: &mut Compilation));
define_hook!(CompilationSeal: AsyncSeries(compilation: &mut Compilation));
define_hook!(CompilationOptimizeDependencies: SyncSeriesBail(compilation: &mut Compilation) -> bool);
define_hook!(CompilationOptimizeModules: AsyncSeriesBail(compilation: &mut Compilation) -> bool);
define_hook!(CompilationAfterOptimizeModules: AsyncSeries(compilation: &mut Compilation));
define_hook!(CompilationOptimizeChunks: SyncSeriesBail(compilation: &mut Compilation) -> bool);
define_hook!(CompilationOptimizeTree: AsyncSeries(compilation: &mut Compilation));
define_hook!(CompilationOptimizeChunkModules: AsyncSeriesBail(compilation: &mut Compilation) -> bool);
define_hook!(CompilationModuleIds: SyncSeries(compilation: &mut Compilation));
define_hook!(CompilationChunkIds: SyncSeries(compilation: &mut Compilation));
define_hook!(CompilationRuntimeModule: AsyncSeries(compilation: &mut Compilation, module: &ModuleIdentifier, chunk: &ChunkUkey));
define_hook!(CompilationRuntimeRequirementInModule: SyncSeriesBail(compilation: &Compilation, module_identifier: &ModuleIdentifier, all_runtime_requirements: &RuntimeGlobals, runtime_requirements: &RuntimeGlobals, runtime_requirements_mut: &mut RuntimeGlobals));
define_hook!(CompilationAdditionalChunkRuntimeRequirements: SyncSeries(compilation: &mut Compilation, chunk_ukey: &ChunkUkey, runtime_requirements: &mut RuntimeGlobals));
define_hook!(CompilationAdditionalTreeRuntimeRequirements: AsyncSeries(compilation: &mut Compilation, chunk_ukey: &ChunkUkey, runtime_requirements: &mut RuntimeGlobals));
define_hook!(CompilationRuntimeRequirementInTree: SyncSeriesBail(compilation: &mut Compilation, chunk_ukey: &ChunkUkey, all_runtime_requirements: &RuntimeGlobals, runtime_requirements: &RuntimeGlobals, runtime_requirements_mut: &mut RuntimeGlobals));
define_hook!(CompilationOptimizeCodeGeneration: SyncSeries(compilation: &mut Compilation));
define_hook!(CompilationChunkHash: AsyncSeries(compilation: &Compilation, chunk_ukey: &ChunkUkey, hasher: &mut RspackHash));
define_hook!(CompilationContentHash: AsyncSeries(compilation: &Compilation, chunk_ukey: &ChunkUkey, hashes: &mut HashMap<SourceType, RspackHash>));
define_hook!(CompilationRenderManifest: AsyncSeries(compilation: &Compilation, chunk_ukey: &ChunkUkey, manifest: &mut Vec<RenderManifestEntry>, diagnostics: &mut Vec<Diagnostic>));
define_hook!(CompilationChunkAsset: AsyncSeries(chunk: &mut Chunk, filename: &str));
define_hook!(CompilationProcessAssets: AsyncSeries(compilation: &mut Compilation));
define_hook!(CompilationAfterProcessAssets: AsyncSeries(compilation: &mut Compilation));
define_hook!(CompilationAfterSeal: AsyncSeries(compilation: &mut Compilation));

#[derive(Debug, Default)]
pub struct CompilationHooks {
  pub add_entry: CompilationAddEntryHook,
  pub build_module: CompilationBuildModuleHook,
  pub still_valid_module: CompilationStillValidModuleHook,
  pub succeed_module: CompilationSucceedModuleHook,
  pub execute_module: CompilationExecuteModuleHook,
  pub finish_modules: CompilationFinishModulesHook,
  pub seal: CompilationSealHook,
  pub optimize_dependencies: CompilationOptimizeDependenciesHook,
  pub optimize_modules: CompilationOptimizeModulesHook,
  pub after_optimize_modules: CompilationAfterOptimizeModulesHook,
  pub optimize_chunks: CompilationOptimizeChunksHook,
  pub optimize_tree: CompilationOptimizeTreeHook,
  pub optimize_chunk_modules: CompilationOptimizeChunkModulesHook,
  pub module_ids: CompilationModuleIdsHook,
  pub chunk_ids: CompilationChunkIdsHook,
  pub runtime_module: CompilationRuntimeModuleHook,
  pub runtime_requirement_in_module: CompilationRuntimeRequirementInModuleHook,
  pub additional_chunk_runtime_requirements: CompilationAdditionalChunkRuntimeRequirementsHook,
  pub additional_tree_runtime_requirements: CompilationAdditionalTreeRuntimeRequirementsHook,
  pub runtime_requirement_in_tree: CompilationRuntimeRequirementInTreeHook,
  pub optimize_code_generation: CompilationOptimizeCodeGenerationHook,
  pub chunk_hash: CompilationChunkHashHook,
  pub content_hash: CompilationContentHashHook,
  pub render_manifest: CompilationRenderManifestHook,
  pub chunk_asset: CompilationChunkAssetHook,
  pub process_assets: CompilationProcessAssetsHook,
  pub after_process_assets: CompilationAfterProcessAssetsHook,
  pub after_seal: CompilationAfterSealHook,
}

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct CompilationId(u32);

impl CompilationId {
  pub fn new() -> Self {
    Self(COMPILATION_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed))
  }
}

impl Default for CompilationId {
  fn default() -> Self {
    Self::new()
  }
}

type ValueCacheVersions = HashMap<String, String>;

static COMPILATION_ID: AtomicU32 = AtomicU32::new(0);
#[derive(Debug)]
pub struct Compilation {
  /// get_compilation_hooks(compilation.id)
  id: CompilationId,
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
  other_module_graph: Option<ModuleGraphPartial>,
  pub dependency_factories: HashMap<DependencyType, Arc<dyn ModuleFactory>>,
  pub runtime_modules: IdentifierMap<Box<dyn RuntimeModule>>,
  pub runtime_modules_hash: IdentifierMap<RspackHashDigest>,
  pub runtime_modules_code_generation_source: IdentifierMap<BoxSource>,
  pub chunk_graph: ChunkGraph,
  pub chunk_by_ukey: ChunkByUkey,
  pub chunk_group_by_ukey: ChunkGroupByUkey,
  pub entrypoints: IndexMap<String, ChunkGroupUkey>,
  pub async_entrypoints: Vec<ChunkGroupUkey>,
  assets: CompilationAssets,
  pub module_assets: IdentifierMap<HashSet<String>>,
  pub emitted_assets: DashSet<String, BuildHasherDefault<FxHasher>>,
  diagnostics: Vec<Diagnostic>,
  logging: CompilationLogging,
  pub plugin_driver: SharedPluginDriver,
  pub buildtime_plugin_driver: SharedPluginDriver,
  pub resolver_factory: Arc<ResolverFactory>,
  pub loader_resolver_factory: Arc<ResolverFactory>,
  pub named_chunks: HashMap<String, ChunkUkey>,
  pub named_chunk_groups: HashMap<String, ChunkGroupUkey>,

  pub async_modules: IdentifierSet,
  pub dependencies_diagnostics: IdentifierMap<Vec<Diagnostic>>,
  pub code_generation_results: CodeGenerationResults,
  pub cgm_hash_results: CgmHashResults,
  pub cgm_runtime_requirements_results: CgmRuntimeRequirementsResults,
  pub built_modules: IdentifierSet,
  pub code_generated_modules: IdentifierSet,
  pub build_time_executed_modules: IdentifierSet,
  pub old_cache: Arc<OldCache>,
  pub code_splitting_cache: CodeSplittingCache,
  pub incremental: Incremental,

  pub hash: Option<RspackHashDigest>,
  pub used_chunk_ids: HashSet<String>,

  pub file_dependencies: IndexSet<PathBuf, BuildHasherDefault<FxHasher>>,
  pub context_dependencies: IndexSet<PathBuf, BuildHasherDefault<FxHasher>>,
  pub missing_dependencies: IndexSet<PathBuf, BuildHasherDefault<FxHasher>>,
  pub build_dependencies: IndexSet<PathBuf, BuildHasherDefault<FxHasher>>,

  pub value_cache_versions: ValueCacheVersions,

  import_var_map: IdentifierDashMap<ImportVarMap>,

  pub module_executor: Option<ModuleExecutor>,

  pub modified_files: HashSet<PathBuf>,
  pub removed_files: HashSet<PathBuf>,
  make_artifact: MakeArtifact,
  pub input_filesystem: Arc<dyn ReadableFileSystem>,
}

impl Compilation {
  pub const OPTIMIZE_CHUNKS_STAGE_BASIC: i32 = -10;
  pub const OPTIMIZE_CHUNKS_STAGE_DEFAULT: i32 = 0;
  pub const OPTIMIZE_CHUNKS_STAGE_ADVANCED: i32 = 10;

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
    plugin_driver: SharedPluginDriver,
    buildtime_plugin_driver: SharedPluginDriver,
    resolver_factory: Arc<ResolverFactory>,
    loader_resolver_factory: Arc<ResolverFactory>,
    records: Option<CompilationRecords>,
    old_cache: Arc<OldCache>,
    module_executor: Option<ModuleExecutor>,
    modified_files: HashSet<PathBuf>,
    removed_files: HashSet<PathBuf>,
    input_filesystem: Arc<dyn ReadableFileSystem>,
  ) -> Self {
    let incremental = Incremental::new(options.experiments.incremental);
    Self {
      id: CompilationId::new(),
      hot_index: 0,
      records,
      options,
      other_module_graph: None,
      dependency_factories: Default::default(),
      runtime_modules: Default::default(),
      runtime_modules_hash: Default::default(),
      runtime_modules_code_generation_source: Default::default(),
      chunk_by_ukey: Default::default(),
      chunk_group_by_ukey: Default::default(),
      entries: Default::default(),
      global_entry: Default::default(),
      chunk_graph: Default::default(),
      entrypoints: Default::default(),
      async_entrypoints: Default::default(),
      assets: Default::default(),
      module_assets: Default::default(),
      emitted_assets: Default::default(),
      diagnostics: Default::default(),
      logging: Default::default(),
      plugin_driver,
      buildtime_plugin_driver,
      resolver_factory,
      loader_resolver_factory,
      named_chunks: Default::default(),
      named_chunk_groups: Default::default(),

      async_modules: Default::default(),
      dependencies_diagnostics: Default::default(),
      code_generation_results: Default::default(),
      cgm_hash_results: Default::default(),
      cgm_runtime_requirements_results: Default::default(),
      built_modules: Default::default(),
      code_generated_modules: Default::default(),
      build_time_executed_modules: Default::default(),
      old_cache,
      incremental,
      code_splitting_cache: Default::default(),
      hash: None,
      used_chunk_ids: Default::default(),

      file_dependencies: Default::default(),
      context_dependencies: Default::default(),
      missing_dependencies: Default::default(),
      build_dependencies: Default::default(),

      value_cache_versions: ValueCacheVersions::default(),

      import_var_map: IdentifierDashMap::default(),

      module_executor,

      make_artifact: Default::default(),
      modified_files,
      removed_files,
      input_filesystem,
    }
  }

  pub fn id(&self) -> CompilationId {
    self.id
  }

  pub fn swap_make_artifact_with_compilation(&mut self, other: &mut Compilation) {
    std::mem::swap(&mut self.make_artifact, &mut other.make_artifact);
  }
  pub fn swap_make_artifact(&mut self, make_artifact: &mut MakeArtifact) {
    std::mem::swap(&mut self.make_artifact, make_artifact);
  }

  pub fn get_module_graph(&self) -> ModuleGraph {
    if let Some(other_module_graph) = &self.other_module_graph {
      ModuleGraph::new(
        vec![
          self.make_artifact.get_module_graph_partial(),
          other_module_graph,
        ],
        None,
      )
    } else {
      ModuleGraph::new(vec![self.make_artifact.get_module_graph_partial()], None)
    }
  }

  // FIXME: find a better way to do this.
  pub fn module_by_identifier(&self, identifier: &ModuleIdentifier) -> Option<&BoxModule> {
    if let Some(other_module_graph) = &self.other_module_graph
      && let Some(module) = other_module_graph.modules.get(identifier)
    {
      return module.as_ref();
    };

    if let Some(module) = self
      .make_artifact
      .get_module_graph_partial()
      .modules
      .get(identifier)
    {
      return module.as_ref();
    }

    None
  }

  pub fn get_module_graph_mut(&mut self) -> ModuleGraph {
    if let Some(other) = &mut self.other_module_graph {
      ModuleGraph::new(
        vec![self.make_artifact.get_module_graph_partial()],
        Some(other),
      )
    } else {
      ModuleGraph::new(
        vec![],
        Some(self.make_artifact.get_module_graph_partial_mut()),
      )
    }
  }

  pub fn file_dependencies(
    &self,
  ) -> (
    impl Iterator<Item = &PathBuf>,
    impl Iterator<Item = &PathBuf>,
    impl Iterator<Item = &PathBuf>,
  ) {
    let all_files = self
      .make_artifact
      .file_dependencies
      .files()
      .chain(&self.file_dependencies);
    let added_files = self
      .make_artifact
      .file_dependencies
      .added_files()
      .iter()
      .chain(&self.file_dependencies);
    let removed_files = self.make_artifact.file_dependencies.removed_files().iter();
    (all_files, added_files, removed_files)
  }

  pub fn context_dependencies(
    &self,
  ) -> (
    impl Iterator<Item = &PathBuf>,
    impl Iterator<Item = &PathBuf>,
    impl Iterator<Item = &PathBuf>,
  ) {
    let all_files = self
      .make_artifact
      .context_dependencies
      .files()
      .chain(&self.context_dependencies);
    let added_files = self
      .make_artifact
      .context_dependencies
      .added_files()
      .iter()
      .chain(&self.file_dependencies);
    let removed_files = self
      .make_artifact
      .context_dependencies
      .removed_files()
      .iter();
    (all_files, added_files, removed_files)
  }

  pub fn missing_dependencies(
    &self,
  ) -> (
    impl Iterator<Item = &PathBuf>,
    impl Iterator<Item = &PathBuf>,
    impl Iterator<Item = &PathBuf>,
  ) {
    let all_files = self
      .make_artifact
      .missing_dependencies
      .files()
      .chain(&self.missing_dependencies);
    let added_files = self
      .make_artifact
      .missing_dependencies
      .added_files()
      .iter()
      .chain(&self.file_dependencies);
    let removed_files = self
      .make_artifact
      .missing_dependencies
      .removed_files()
      .iter();
    (all_files, added_files, removed_files)
  }

  pub fn build_dependencies(
    &self,
  ) -> (
    impl Iterator<Item = &PathBuf>,
    impl Iterator<Item = &PathBuf>,
    impl Iterator<Item = &PathBuf>,
  ) {
    let all_files = self
      .make_artifact
      .build_dependencies
      .files()
      .chain(&self.build_dependencies);
    let added_files = self
      .make_artifact
      .build_dependencies
      .added_files()
      .iter()
      .chain(&self.file_dependencies);
    let removed_files = self.make_artifact.build_dependencies.removed_files().iter();
    (all_files, added_files, removed_files)
  }

  // TODO move out from compilation
  pub fn get_import_var(&self, dep_id: &DependencyId) -> String {
    let module_graph = self.get_module_graph();
    let parent_module_id = module_graph
      .get_parent_module(dep_id)
      .expect("should have parent module");
    let module_id = module_graph
      .module_identifier_by_dependency_id(dep_id)
      .copied();
    let module_dep = module_graph
      .dependency_by_id(dep_id)
      .and_then(|dep| dep.as_module_dependency())
      .expect("should be module dependency");
    let user_request = to_identifier(module_dep.user_request());
    let mut import_var_map_of_module = self.import_var_map.entry(*parent_module_id).or_default();
    let len = import_var_map_of_module.len();

    let import_var = match import_var_map_of_module.entry(module_id) {
      hash_map::Entry::Occupied(occ) => occ.get().clone(),
      hash_map::Entry::Vacant(vac) => {
        let import_var = format!("{}__WEBPACK_IMPORTED_MODULE_{}__", user_request, itoa!(len));
        vac.insert(import_var.clone());
        import_var
      }
    };
    import_var
  }

  pub async fn add_entry(&mut self, entry: BoxDependency, options: EntryOptions) -> Result<()> {
    let entry_id = *entry.id();
    let entry_name = options.name.clone();
    self.get_module_graph_mut().add_dependency(entry);
    if let Some(name) = &entry_name {
      if let Some(data) = self.entries.get_mut(name) {
        data.dependencies.push(entry_id);
        data.options.merge(options)?;
      } else {
        let data = EntryData {
          dependencies: vec![entry_id],
          include_dependencies: vec![],
          options,
        };
        self.entries.insert(name.to_owned(), data);
      }
    } else {
      self.global_entry.dependencies.push(entry_id);
    }

    self
      .plugin_driver
      .clone()
      .compilation_hooks
      .add_entry
      .call(self, entry_name.as_deref())
      .await?;
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

    Ok(())
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
  pub fn splice_diagnostic(
    &mut self,
    s: usize,
    e: usize,
    replace_with: Vec<Diagnostic>,
  ) -> Vec<Diagnostic> {
    self.diagnostics.splice(s..e, replace_with).collect()
  }

  pub fn extend_diagnostics(&mut self, diagnostics: impl IntoIterator<Item = Diagnostic>) {
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
  ///   Rspack assumes for each offset, there is only one error.
  ///   However, when it comes to the case that there are multiple errors with the same offset,
  ///   the order of these errors will not be guaranteed.
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
  ///   Rspack assumes for each offset, there is only one error.
  ///   However, when it comes to the case that there are multiple errors with the same offset,
  ///   the order of these errors will not be guaranteed.
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
  pub async fn make(&mut self) -> Result<()> {
    self.make_artifact.reset_dependencies_incremental_info();
    //        self.module_executor.
    // run module_executor
    if let Some(module_executor) = &mut self.module_executor {
      let mut module_executor = std::mem::take(module_executor);
      module_executor.hook_before_make(self).await;
      self.module_executor = Some(module_executor);
    }

    let artifact = std::mem::take(&mut self.make_artifact);
    self.make_artifact = make_module_graph(self, artifact).await?;
    Ok(())
  }

  pub async fn rebuild_module<T>(
    &mut self,
    module_identifiers: IdentifierSet,
    f: impl Fn(Vec<&BoxModule>) -> T,
  ) -> Result<T> {
    let artifact = std::mem::take(&mut self.make_artifact);
    self.make_artifact = update_module_graph(
      self,
      artifact,
      vec![MakeParam::ForceBuildModules(module_identifiers.clone())],
    )
    .await?;

    let module_graph = self.get_module_graph();
    Ok(f(module_identifiers
      .into_iter()
      .filter_map(|id| module_graph.module_by_identifier(&id))
      .collect::<Vec<_>>()))
  }

  #[instrument(name = "compilation:code_generation", skip_all)]
  fn code_generation(&mut self, modules: IdentifierSet) -> Result<()> {
    let logger = self.get_logger("rspack.Compilation");
    let mut codegen_cache_counter = match self.options.cache {
      CacheOptions::Disabled => None,
      _ => Some(logger.cache("module code generation cache")),
    };

    let module_graph = self.get_module_graph();
    let mut no_codegen_dependencies_modules = IdentifierSet::default();
    let mut has_codegen_dependencies_modules = IdentifierSet::default();
    for module_identifier in modules {
      let module = module_graph
        .module_by_identifier(&module_identifier)
        .expect("should have module");
      if module.get_code_generation_dependencies().is_none() {
        no_codegen_dependencies_modules.insert(module_identifier);
      } else {
        has_codegen_dependencies_modules.insert(module_identifier);
      }
    }

    self.code_generation_modules(&mut codegen_cache_counter, no_codegen_dependencies_modules)?;
    self.code_generation_modules(&mut codegen_cache_counter, has_codegen_dependencies_modules)?;

    if let Some(counter) = codegen_cache_counter {
      logger.cache_end(counter);
    }

    Ok(())
  }

  pub(crate) fn code_generation_modules(
    &mut self,
    cache_counter: &mut Option<CacheCount>,
    modules: IdentifierSet,
  ) -> Result<()> {
    let chunk_graph = &self.chunk_graph;
    let module_graph = self.get_module_graph();
    let mut jobs = Vec::new();
    for module in modules {
      let runtimes = chunk_graph.get_module_runtimes(module, &self.chunk_by_ukey);
      if runtimes.is_empty() {
        continue;
      }
      if runtimes.len() == 1 {
        let runtime = runtimes
          .into_values()
          .next()
          .expect("should have first value");
        let hash = ChunkGraph::get_module_hash(self, module, &runtime)
          .expect("should have cgm.hash in code generation")
          .clone();
        jobs.push(CodeGenerationJob {
          module,
          hash,
          runtime: runtime.clone(),
          runtimes: vec![runtime],
        })
      } else {
        let mut map: HashMap<RspackHashDigest, CodeGenerationJob> = HashMap::default();
        for runtime in runtimes.into_values() {
          let hash = ChunkGraph::get_module_hash(self, module, &runtime)
            .expect("should have cgm.hash in code generation")
            .clone();
          if let Some(job) = map.get_mut(&hash) {
            job.runtimes.push(runtime);
          } else {
            map.insert(
              hash.clone(),
              CodeGenerationJob {
                module,
                hash,
                runtime: runtime.clone(),
                runtimes: vec![runtime],
              },
            );
          }
        }
        jobs.extend(map.into_values());
      }
    }
    let results = jobs
      .into_par_iter()
      .map(|job| {
        let module = job.module;
        let runtimes = job.runtimes.clone();
        (
          module,
          runtimes,
          self
            .old_cache
            .code_generate_occasion
            .use_cache(job, |module, runtime| {
              let module = module_graph
                .module_by_identifier(&module)
                .expect("should have module");
              module.code_generation(self, Some(runtime), None)
            }),
        )
      })
      .map(|(module, runtime, result)| match result {
        Ok((result, runtime, from_cache)) => (module, result, runtime, from_cache, None),
        Err(err) => (
          module,
          CodeGenerationResult::default(),
          runtime,
          false,
          Some(err),
        ),
      })
      .collect::<Vec<_>>();
    for (module, mut codegen_res, runtimes, from_cache, err) in results {
      if let Some(counter) = cache_counter {
        if from_cache {
          counter.hit();
        } else {
          counter.miss();
        }
      }
      codegen_res.set_hash(
        &self.options.output.hash_function,
        &self.options.output.hash_digest,
        &self.options.output.hash_salt,
      );
      self
        .code_generation_results
        .insert(module, codegen_res, runtimes);
      self.code_generated_modules.insert(module);

      if let Some(err) = err {
        self.push_diagnostic(Diagnostic::from(err).with_module_identifier(Some(module)));
      }
    }
    Ok(())
  }

  #[instrument(name = "compilation::create_module_assets", skip_all)]
  async fn create_module_assets(&mut self, _plugin_driver: SharedPluginDriver) {
    let mut temp = vec![];
    for (module_identifier, assets) in self.module_assets.iter() {
      // assets of executed modules are not in this compilation
      if self
        .chunk_graph
        .chunk_graph_module_by_module_identifier
        .contains_key(module_identifier)
      {
        for chunk in self
          .chunk_graph
          .get_module_chunks(*module_identifier)
          .iter()
        {
          for asset in assets {
            temp.push((*chunk, asset.clone()))
          }
        }
      }
    }

    for (chunk, asset) in temp {
      let chunk = self.chunk_by_ukey.expect_get_mut(&chunk);
      chunk.auxiliary_files.insert(asset);
    }
  }

  #[instrument(skip_all)]
  async fn create_chunk_assets(&mut self, plugin_driver: SharedPluginDriver) -> Result<()> {
    let results = self
      .chunk_by_ukey
      .values()
      .map(|chunk| async {
        let mut manifest = Vec::new();
        let mut diagnostics = Vec::new();
        plugin_driver
          .compilation_hooks
          .render_manifest
          .call(self, &chunk.ukey, &mut manifest, &mut diagnostics)
          .await?;

        Ok((chunk.ukey, manifest, diagnostics))
      })
      .collect::<FuturesResults<Result<_>>>();

    let chunk_ukey_and_manifest = results.into_inner();

    for result in chunk_ukey_and_manifest {
      let (chunk_ukey, manifest, diagnostics) = result?;
      self.extend_diagnostics(diagnostics);

      for file_manifest in manifest {
        let filename = file_manifest.filename().to_string();
        let current_chunk = self.chunk_by_ukey.expect_get_mut(&chunk_ukey);

        current_chunk.rendered = true;
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
          .chunk_asset(chunk_ukey, &filename, plugin_driver.clone())
          .await;
      }
    }

    // TODO: add code_generated_modules in render_runtime_modules
    for (identifier, _) in self.runtime_modules.iter() {
      self.code_generated_modules.insert(*identifier);
    }
    Ok(())
  }

  #[instrument(name = "compilation:after_process_asssets", skip_all)]
  async fn after_process_assets(&mut self, plugin_driver: SharedPluginDriver) -> Result<()> {
    plugin_driver
      .compilation_hooks
      .after_process_assets
      .call(self)
      .await
  }

  #[instrument(name = "compilation:after_seal", skip_all)]
  async fn after_seal(&mut self, plugin_driver: SharedPluginDriver) -> Result<()> {
    plugin_driver.compilation_hooks.after_seal.call(self).await
  }

  #[instrument(
    name = "compilation:chunk_asset",
    skip(self, plugin_driver, chunk_ukey)
  )]
  async fn chunk_asset(
    &mut self,
    chunk_ukey: ChunkUkey,
    filename: &str,
    plugin_driver: SharedPluginDriver,
  ) -> Result<()> {
    let current_chunk = self.chunk_by_ukey.expect_get_mut(&chunk_ukey);
    plugin_driver
      .compilation_hooks
      .chunk_asset
      .call(current_chunk, filename)
      .await?;
    Ok(())
  }

  pub fn entry_modules(&self) -> IdentifierSet {
    let module_graph = self.get_module_graph();
    self
      .entries
      .values()
      .flat_map(|item| item.all_dependencies())
      .chain(self.global_entry.all_dependencies())
      .filter_map(|dep_id| {
        // some entry dependencies may not find module because of resolve failed
        // so use filter_map to ignore them
        module_graph
          .module_identifier_by_dependency_id(dep_id)
          .copied()
      })
      .collect()
  }

  pub fn entrypoint_by_name(&self, name: &str) -> &Entrypoint {
    let ukey = self.entrypoints.get(name).expect("entrypoint not found");
    self.chunk_group_by_ukey.expect_get(ukey)
  }

  #[instrument(name = "compilation:finish", skip_all)]
  pub async fn finish(&mut self, plugin_driver: SharedPluginDriver) -> Result<()> {
    let logger = self.get_logger("rspack.Compilation");

    // Recheck entry and clean useless entry
    // This should before finish_modules hook is called, ensure providedExports effects on new added modules
    let make_artifact = std::mem::take(&mut self.make_artifact);
    self.make_artifact = update_module_graph(
      self,
      make_artifact,
      vec![MakeParam::BuildEntryAndClean(
        self
          .entries
          .values()
          .flat_map(|item| item.all_dependencies())
          .chain(self.global_entry.all_dependencies())
          .copied()
          .collect(),
      )],
    )
    .await?;

    // sync assets to compilation from module_executor
    if let Some(module_executor) = &mut self.module_executor {
      let mut module_executor = std::mem::take(module_executor);
      module_executor.hook_after_finish_modules(self).await;
      self.module_executor = Some(module_executor);
    }

    // take built_modules
    let revoked_modules = self.make_artifact.take_revoked_modules();
    let built_modules = self.make_artifact.take_built_modules();
    if let Some(mutations) = self.incremental.mutations_write() {
      mutations.extend(
        revoked_modules
          .iter()
          .map(|&module| Mutation::ModuleRevoke { module }),
      );
      mutations.extend(
        built_modules
          .iter()
          .map(|&module| Mutation::ModuleBuild { module }),
      );
    }
    self.built_modules.extend(built_modules);

    let start = logger.time("finish modules");
    plugin_driver
      .compilation_hooks
      .finish_modules
      .call(self)
      .await?;
    logger.time_end(start);
    // Collect dependencies diagnostics at here to make sure:
    // 1. after finish_modules: has provide exports info
    // 2. before optimize dependencies: side effects free module hasn't been skipped
    self.collect_dependencies_diagnostics();

    // take make diagnostics
    let diagnostics = self.make_artifact.take_diagnostics();
    self.extend_diagnostics(diagnostics);

    Ok(())
  }

  #[tracing::instrument(skip_all)]
  fn collect_dependencies_diagnostics(&mut self) {
    let mutations = self
      .incremental
      .mutations_read(IncrementalPasses::DEPENDENCIES_DIAGNOSTICS);
    let modules = if let Some(mutations) = mutations {
      let revoked_modules = mutations.iter().filter_map(|mutation| match mutation {
        Mutation::ModuleRevoke { module } => Some(*module),
        _ => None,
      });
      for revoked_module in revoked_modules {
        self.dependencies_diagnostics.remove(&revoked_module);
      }
      mutations.get_affected_modules_with_module_graph(&self.get_module_graph())
    } else {
      self.get_module_graph().modules().keys().copied().collect()
    };
    let module_graph = self.get_module_graph();
    let dependencies_diagnostics: IdentifierMap<Vec<Diagnostic>> = modules
      .par_iter()
      .map(|module_identifier| {
        let mgm = module_graph
          .module_graph_module_by_identifier(module_identifier)
          .expect("should have mgm");
        let diagnostics = mgm
          .all_dependencies
          .iter()
          .filter_map(|dependency_id| module_graph.dependency_by_id(dependency_id))
          .filter_map(|dependency| dependency.get_diagnostics(&module_graph))
          .flatten()
          .collect::<Vec<_>>();
        (*module_identifier, diagnostics)
      })
      .collect();
    let all_modules_diagnostics = if mutations.is_some() {
      self
        .dependencies_diagnostics
        .extend(dependencies_diagnostics);
      self.dependencies_diagnostics.clone()
    } else {
      dependencies_diagnostics
    };
    self.extend_diagnostics(all_modules_diagnostics.into_values().flatten());
  }

  #[instrument(name = "compilation:seal", skip_all)]
  pub async fn seal(&mut self, plugin_driver: SharedPluginDriver) -> Result<()> {
    self.other_module_graph = Some(ModuleGraphPartial::default());
    let logger = self.get_logger("rspack.Compilation");

    // https://github.com/webpack/webpack/blob/main/lib/Compilation.js#L2809
    plugin_driver.compilation_hooks.seal.call(self).await?;

    let start = logger.time("optimize dependencies");
    // https://github.com/webpack/webpack/blob/d15c73469fd71cf98734685225250148b68ddc79/lib/Compilation.js#L2812-L2814
    while matches!(
      plugin_driver
        .compilation_hooks
        .optimize_dependencies
        .call(self)?,
      Some(true)
    ) {}
    logger.time_end(start);

    let start = logger.time("create chunks");
    use_code_splitting_cache(self, |compilation| async {
      build_chunk_graph(compilation)?;
      Ok(compilation)
    })
    .await?;

    while matches!(
      plugin_driver
        .compilation_hooks
        .optimize_modules
        .call(self)
        .await?,
      Some(true)
    ) {}
    plugin_driver
      .compilation_hooks
      .after_optimize_modules
      .call(self)
      .await?;
    while matches!(
      plugin_driver.compilation_hooks.optimize_chunks.call(self)?,
      Some(true)
    ) {}

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
    plugin_driver.compilation_hooks.module_ids.call(self)?;
    logger.time_end(start);

    let start = logger.time("chunk ids");
    plugin_driver.compilation_hooks.chunk_ids.call(self)?;

    logger.time_end(start);

    self.assign_runtime_ids();

    let create_module_hashes_modules = if let Some(mutations) = self
      .incremental
      .mutations_read(IncrementalPasses::MODULES_HASHES)
    {
      let revoked_modules = mutations.iter().filter_map(|mutation| match mutation {
        Mutation::ModuleRevoke { module } => Some(*module),
        _ => None,
      });
      for revoked_module in revoked_modules {
        self.cgm_hash_results.remove(&revoked_module);
      }
      mutations.get_affected_modules_with_chunk_graph(self)
    } else {
      self.get_module_graph().modules().keys().copied().collect()
    };
    self.create_module_hashes(create_module_hashes_modules)?;

    let start = logger.time("optimize code generation");
    plugin_driver
      .compilation_hooks
      .optimize_code_generation
      .call(self)?;
    logger.time_end(start);

    let start = logger.time("code generation");
    let code_generation_modules = if let Some(mutations) = self
      .incremental
      .mutations_read(IncrementalPasses::MODULES_CODEGEN)
    {
      let revoked_modules = mutations.iter().filter_map(|mutation| match mutation {
        Mutation::ModuleRevoke { module } => Some(*module),
        _ => None,
      });
      for revoked_module in revoked_modules {
        self.code_generation_results.remove(&revoked_module);
      }
      mutations.get_affected_modules_with_chunk_graph(self)
    } else {
      self.get_module_graph().modules().keys().copied().collect()
    };
    self.code_generation(code_generation_modules)?;
    logger.time_end(start);

    let start = logger.time("runtime requirements");
    let process_runtime_requirements_modules = if let Some(mutations) = self
      .incremental
      .mutations_read(IncrementalPasses::MODULES_RUNTIME_REQUIREMENTS)
    {
      let revoked_modules = mutations.iter().filter_map(|mutation| match mutation {
        Mutation::ModuleRevoke { module } => Some(*module),
        _ => None,
      });
      for revoked_module in revoked_modules {
        self
          .cgm_runtime_requirements_results
          .remove(&revoked_module);
      }
      mutations.get_affected_modules_with_chunk_graph(self)
    } else {
      self.get_module_graph().modules().keys().copied().collect()
    };
    self
      .process_runtime_requirements(
        process_runtime_requirements_modules,
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
    self.runtime_modules_code_generation()?;
    logger.time_end(start);

    let start = logger.time("create module assets");
    self.create_module_assets(plugin_driver.clone()).await;
    logger.time_end(start);

    let start = logger.time("create chunk assets");
    self.create_chunk_assets(plugin_driver.clone()).await?;
    logger.time_end(start);

    let start = logger.time("process assets");
    plugin_driver
      .compilation_hooks
      .process_assets
      .call(self)
      .await?;
    logger.time_end(start);

    let start = logger.time("after process assets");
    self.after_process_assets(plugin_driver.clone()).await?;
    logger.time_end(start);

    let start = logger.time("after seal");
    self.after_seal(plugin_driver).await?;
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
        .and_then(|o| match &o.runtime {
          Some(EntryRuntime::String(s)) => Some(s.to_owned()),
          _ => None,
        })
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

  pub fn get_chunk_graph_entries(&self) -> UkeySet<ChunkUkey> {
    let entries = self.entrypoints.values().map(|entrypoint_ukey| {
      let entrypoint = self.chunk_group_by_ukey.expect_get(entrypoint_ukey);
      entrypoint.get_runtime_chunk(&self.chunk_group_by_ukey)
    });
    let async_entries = self.async_entrypoints.iter().map(|entrypoint_ukey| {
      let entrypoint = self.chunk_group_by_ukey.expect_get(entrypoint_ukey);
      entrypoint.get_runtime_chunk(&self.chunk_group_by_ukey)
    });
    entries.chain(async_entries).collect()
  }

  #[allow(clippy::unwrap_in_result)]
  #[instrument(name = "compilation:process_runtime_requirements", skip_all)]
  pub async fn process_runtime_requirements(
    &mut self,
    modules: IdentifierSet,
    chunks: impl Iterator<Item = ChunkUkey>,
    chunk_graph_entries: impl Iterator<Item = ChunkUkey>,
    plugin_driver: SharedPluginDriver,
  ) -> Result<()> {
    fn process_runtime_requirement_hook(
      requirements: &mut RuntimeGlobals,
      mut call_hook: impl FnMut(&RuntimeGlobals, &RuntimeGlobals, &mut RuntimeGlobals) -> Result<()>,
    ) -> Result<()> {
      let mut runtime_requirements_mut = *requirements;
      let mut runtime_requirements;

      loop {
        runtime_requirements = runtime_requirements_mut;
        runtime_requirements_mut = RuntimeGlobals::default();
        call_hook(
          requirements,
          &runtime_requirements,
          &mut runtime_requirements_mut,
        )?;
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

    let logger = self.get_logger("rspack.Compilation");
    let start = logger.time("runtime requirements.modules");
    let results: Vec<(ModuleIdentifier, RuntimeSpecMap<RuntimeGlobals>)> = modules
      .into_par_iter()
      .filter(|module| self.chunk_graph.get_number_of_module_chunks(*module) > 0)
      .map(|module| {
        let runtimes = self
          .chunk_graph
          .get_module_runtimes(module, &self.chunk_by_ukey);
        let mut map = RuntimeSpecMap::new();
        for runtime in runtimes.into_values() {
          let runtime_requirements = self
            .old_cache
            .process_runtime_requirements_occasion
            .use_cache(module, &runtime, self, |module, runtime| {
              let mut runtime_requirements = self
                .code_generation_results
                .get_runtime_requirements(&module, Some(runtime));
              process_runtime_requirement_hook(
                &mut runtime_requirements,
                |all_runtime_requirements, runtime_requirements, runtime_requirements_mut| {
                  plugin_driver
                    .compilation_hooks
                    .runtime_requirement_in_module
                    .call(
                      self,
                      &module,
                      all_runtime_requirements,
                      runtime_requirements,
                      runtime_requirements_mut,
                    )?;
                  Ok(())
                },
              )?;
              Ok(runtime_requirements)
            })?;
          map.set(runtime, runtime_requirements);
        }
        Ok((module, map))
      })
      .collect::<Result<_>>()?;
    for (module, map) in results {
      ChunkGraph::set_module_runtime_requirements(self, module, map);
    }
    logger.time_end(start);

    let start = logger.time("runtime requirements.chunks");
    let mut chunk_requirements = HashMap::default();
    for chunk_ukey in chunks {
      let mut set = RuntimeGlobals::default();
      for module in self
        .chunk_graph
        .get_chunk_modules(&chunk_ukey, &self.get_module_graph())
      {
        let chunk = self.chunk_by_ukey.expect_get(&chunk_ukey);
        if let Some(runtime_requirements) =
          ChunkGraph::get_module_runtime_requirements(self, module.identifier(), &chunk.runtime)
        {
          set.insert(*runtime_requirements);
        }
      }
      chunk_requirements.insert(chunk_ukey, set);
    }
    for (chunk_ukey, set) in chunk_requirements.iter_mut() {
      plugin_driver
        .compilation_hooks
        .additional_chunk_runtime_requirements
        .call(self, chunk_ukey, set)?;

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
        .compilation_hooks
        .additional_tree_runtime_requirements
        .call(self, &entry_ukey, &mut set)
        .await?;

      process_runtime_requirement_hook(
        &mut set,
        |all_runtime_requirements, runtime_requirements, runtime_requirements_mut| {
          plugin_driver
            .compilation_hooks
            .runtime_requirement_in_tree
            .call(
              self,
              &entry_ukey,
              all_runtime_requirements,
              runtime_requirements,
              runtime_requirements_mut,
            )?;
          Ok(())
        },
      )?;

      self
        .chunk_graph
        .add_tree_runtime_requirements(&entry_ukey, set);
    }

    // NOTE: webpack runs hooks.runtime_module in compilation.add_runtime_module
    // and overwrite the runtime_module.generate() to get new source in create_chunk_assets
    // this needs full runtime requirements, so run hooks.runtime_module after runtime_requirements_in_tree
    for entry_ukey in self.get_chunk_graph_entries() {
      let runtime_module_ids: Vec<_> = self
        .chunk_graph
        .get_chunk_runtime_modules_iterable(&entry_ukey)
        .copied()
        .collect();
      for runtime_module_id in runtime_module_ids {
        self
          .plugin_driver
          .clone()
          .compilation_hooks
          .runtime_module
          .call(self, &runtime_module_id, &entry_ukey)
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

    let unordered_runtime_chunks = self.get_chunk_graph_entries();
    let start = logger.time("hashing: hash chunks");
    let other_chunks = self
      .chunk_by_ukey
      .keys()
      .filter(|key| !unordered_runtime_chunks.contains(key));
    //   .collect();
    // // create hash for runtime modules in other chunks
    // for chunk in &other_chunks {
    //   for runtime_module_identifier in self.chunk_graph.get_chunk_runtime_modules_iterable(chunk) {
    //     let runtime_module = &self.runtime_modules[runtime_module_identifier];
    //     let mut hasher = RspackHash::from(&self.options.output);
    //     runtime_module.update_hash(&mut hasher, self, None)?;
    //     let digest = hasher.digest(&self.options.output.hash_digest);
    //     self
    //       .runtime_modules_hash
    //       .insert(*runtime_module_identifier, digest);
    //   }
    // }
    // create hash for other chunks
    let other_chunks_hash_results: Vec<Result<(ChunkUkey, (RspackHashDigest, ChunkContentHash))>> =
      other_chunks
        // .into_iter()
        .map(|chunk| async {
          let hash_result = self.process_chunk_hash(*chunk, &plugin_driver).await?;
          Ok((*chunk, hash_result))
        })
        .collect::<FuturesResults<_>>()
        .into_inner();
    try_process_chunk_hash_results(self, other_chunks_hash_results)?;
    logger.time_end(start);

    // collect references for runtime chunks
    let mut runtime_chunks_map: HashMap<ChunkUkey, (Vec<ChunkUkey>, u32)> =
      unordered_runtime_chunks
        .into_iter()
        .map(|runtime_chunk| (runtime_chunk, (Vec::new(), 0)))
        .collect();
    let mut remaining: u32 = 0;
    for runtime_chunk_ukey in runtime_chunks_map.keys().copied().collect::<Vec<_>>() {
      let runtime_chunk = self.chunk_by_ukey.expect_get(&runtime_chunk_ukey);
      let groups = runtime_chunk.get_all_referenced_async_entrypoints(&self.chunk_group_by_ukey);
      for other in groups
        .into_iter()
        .map(|group| self.chunk_group_by_ukey.expect_get(&group))
        .map(|group| group.get_runtime_chunk(&self.chunk_group_by_ukey))
      {
        let (other_referenced_by, _) = runtime_chunks_map
          .get_mut(&other)
          .expect("should in runtime_chunks_map");
        other_referenced_by.push(runtime_chunk_ukey);
        let info = runtime_chunks_map
          .get_mut(&runtime_chunk_ukey)
          .expect("should in runtime_chunks_map");
        info.1 += 1;
        remaining += 1;
      }
    }
    // sort runtime chunks by its references
    let mut runtime_chunks = Vec::with_capacity(runtime_chunks_map.len());
    for (runtime_chunk, (_, remaining)) in &runtime_chunks_map {
      if *remaining == 0 {
        runtime_chunks.push(*runtime_chunk);
      }
    }
    let mut ready_chunks = Vec::new();
    let mut full_hash_chunks = UkeySet::default();
    let mut i = 0;
    while i < runtime_chunks.len() {
      let chunk_ukey = runtime_chunks[i];
      let has_full_hash_modules = full_hash_chunks.contains(&chunk_ukey)
        || self
          .chunk_graph
          .has_chunk_full_hash_modules(&chunk_ukey, &self.runtime_modules);
      if has_full_hash_modules {
        full_hash_chunks.insert(chunk_ukey);
      }
      let referenced_by = runtime_chunks_map
        .get(&chunk_ukey)
        .expect("should in runtime_chunks_map")
        .0
        .clone();
      for other in referenced_by {
        if has_full_hash_modules {
          for runtime_module in self.chunk_graph.get_chunk_runtime_modules_iterable(&other) {
            let runtime_module = self
              .runtime_modules
              .get(runtime_module)
              .expect("should have runtime_module");
            if runtime_module.dependent_hash() {
              full_hash_chunks.insert(other);
              break;
            }
          }
        }
        remaining -= 1;
        let (_, other_remaining) = runtime_chunks_map
          .get_mut(&other)
          .expect("should in runtime_chunks_map");
        *other_remaining -= 1;
        if *other_remaining == 0 {
          ready_chunks.push(other);
        }
      }
      if !ready_chunks.is_empty() {
        runtime_chunks.append(&mut ready_chunks);
      }
      i += 1;
    }
    // create warning for remaining circular references
    if remaining > 0 {
      let mut circular: Vec<_> = runtime_chunks_map
        .iter()
        .filter(|(_, (_, remaining))| *remaining != 0)
        .map(|(chunk_ukey, _)| self.chunk_by_ukey.expect_get(chunk_ukey))
        .collect();
      circular.sort_unstable_by(|a, b| a.id.cmp(&b.id));
      runtime_chunks.extend(circular.iter().map(|chunk| chunk.ukey));
      let circular_names = circular
        .iter()
        .map(|chunk| {
          chunk
            .name
            .as_deref()
            .or(chunk.id.as_deref())
            .unwrap_or("no id chunk")
        })
        .join(", ");
      self.push_diagnostic(diagnostic!(severity = Severity::Warn, "Circular dependency between chunks with runtime ({})\nThis prevents using hashes of each other and should be avoided.", circular_names).boxed().into());
    }

    // create hash for runtime chunks and the runtime modules within them
    // The subsequent runtime chunks and runtime modules will depend on
    // the hash results of the previous runtime chunks and runtime modules.
    // Therefore, create hashes one by one in sequence.
    let start = logger.time("hashing: hash runtime chunks");
    for runtime_chunk_ukey in runtime_chunks {
      for runtime_module_identifier in self
        .chunk_graph
        .get_chunk_runtime_modules_iterable(&runtime_chunk_ukey)
      {
        let runtime_module = &self.runtime_modules[runtime_module_identifier];
        let mut hasher = RspackHash::from(&self.options.output);
        runtime_module.update_hash(&mut hasher, self, None)?;
        let digest = hasher.digest(&self.options.output.hash_digest);
        self
          .runtime_modules_hash
          .insert(*runtime_module_identifier, digest);
      }
      let (chunk_hash, content_hash) = self
        .process_chunk_hash(runtime_chunk_ukey, &plugin_driver)
        .await?;
      let chunk = self.chunk_by_ukey.expect_get_mut(&runtime_chunk_ukey);
      chunk.rendered_hash = Some(
        chunk_hash
          .rendered(self.options.output.hash_digest_length)
          .into(),
      );
      chunk.hash = Some(chunk_hash);
      chunk.content_hash = content_hash;
    }
    logger.time_end(start);

    // create full hash
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

    // re-create runtime chunk hash that depend on full hash
    let start = logger.time("hashing: process full hash chunks");
    for runtime_chunk_ukey in full_hash_chunks {
      for runtime_module_identifier in self
        .chunk_graph
        .get_chunk_runtime_modules_iterable(&runtime_chunk_ukey)
      {
        let runtime_module = &self.runtime_modules[runtime_module_identifier];
        if runtime_module.full_hash() || runtime_module.dependent_hash() {
          let mut hasher = RspackHash::from(&self.options.output);
          runtime_module.update_hash(&mut hasher, self, None)?;
          let digest = hasher.digest(&self.options.output.hash_digest);
          self
            .runtime_modules_hash
            .insert(*runtime_module_identifier, digest);
        }
      }
      let chunk = self.chunk_by_ukey.expect_get_mut(&runtime_chunk_ukey);
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
    logger.time_end(start);
    Ok(())
  }

  pub fn runtime_modules_code_generation(&mut self) -> Result<()> {
    self.runtime_modules_code_generation_source = self
      .runtime_modules
      .par_iter()
      .map(|(runtime_module_identifier, runtime_module)| -> Result<_> {
        let result = runtime_module.code_generation(self, None, None)?;
        let source = result
          .get(&SourceType::Runtime)
          .expect("should have source");
        Ok((*runtime_module_identifier, source.clone()))
      })
      .collect::<Result<_>>()?;
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
      .compilation_hooks
      .chunk_hash
      .call(self, &chunk_ukey, &mut hasher)
      .await?;
    let chunk_hash = hasher.digest(&self.options.output.hash_digest);

    let mut content_hashes = HashMap::default();
    plugin_driver
      .compilation_hooks
      .content_hash
      .call(self, &chunk_ukey, &mut content_hashes)
      .await?;
    let content_hashes = content_hashes
      .into_iter()
      .map(|(t, hasher)| (t, hasher.digest(&self.options.output.hash_digest)))
      .collect();

    Ok((chunk_hash, content_hashes))
  }

  #[instrument(name = "compilation:create_module_hashes", skip_all)]
  pub fn create_module_hashes(&mut self, modules: IdentifierSet) -> Result<()> {
    let results: Vec<(ModuleIdentifier, RuntimeSpecMap<RspackHashDigest>)> = modules
      .into_par_iter()
      .map(|module| {
        (
          module,
          self
            .chunk_graph
            .get_module_runtimes(module, &self.chunk_by_ukey),
        )
      })
      .map(|(module_identifier, runtimes)| {
        let mut hashes = RuntimeSpecMap::new();
        for runtime in runtimes.into_values() {
          let mut hasher = RspackHash::from(&self.options.output);
          let mg = self.get_module_graph();
          let module = mg
            .module_by_identifier(&module_identifier)
            .expect("should have module");
          module.update_hash(&mut hasher, self, Some(&runtime))?;
          hashes.set(runtime, hasher.digest(&self.options.output.hash_digest));
        }
        Ok((module_identifier, hashes))
      })
      .collect::<Result<_>>()?;
    for (module, hashes) in results {
      ChunkGraph::set_module_hashes(self, module, hashes);
    }
    Ok(())
  }

  pub fn add_runtime_module(
    &mut self,
    chunk_ukey: &ChunkUkey,
    mut module: Box<dyn RuntimeModule>,
  ) -> Result<()> {
    // add chunk runtime to prefix module identifier to avoid multiple entry runtime modules conflict
    let chunk = self.chunk_by_ukey.expect_get(chunk_ukey);
    let runtime_module_identifier =
      ModuleIdentifier::from(format!("{}/{}", &chunk.runtime, module.identifier()));
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
    filename.render(data, None, self.options.output.hash_digest_length)
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
    let path = filename.render(
      data,
      Some(&mut info),
      self.options.output.hash_digest_length,
    )?;
    Ok((path, info))
  }

  pub fn get_asset_path<F: LocalFilenameFn>(
    &self,
    filename: &Filename<F>,
    data: PathData,
  ) -> Result<String, F::Error> {
    filename.render(data, None, self.options.output.hash_digest_length)
  }

  pub fn get_asset_path_with_info<F: LocalFilenameFn>(
    &self,
    filename: &Filename<F>,
    data: PathData,
  ) -> Result<(String, AssetInfo), F::Error> {
    let mut info = AssetInfo::default();
    let path = filename.render(
      data,
      Some(&mut info),
      self.options.output.hash_digest_length,
    )?;
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
      .get(dependency_type)
      .unwrap_or_else(|| {
        panic!(
          "No module factory available for dependency type: {}, resourceIdentifier: {:?}",
          dependency_type,
          dependency.resource_identifier()
        )
      })
      .clone()
  }

  // TODO remove it after code splitting support incremental rebuild
  pub fn has_module_import_export_change(&self) -> bool {
    self.make_artifact.has_module_graph_change
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
  pub immutable: Option<bool>,
  /// whether the asset is minimized
  pub minimized: Option<bool>,
  /// the value(s) of the full hash used for this asset
  pub full_hash: HashSet<String>,
  /// the value(s) of the chunk hash used for this asset
  pub chunk_hash: HashSet<String>,
  /// the value(s) of the module hash used for this asset
  // pub module_hash:
  /// the value(s) of the content hash used for this asset
  pub content_hash: HashSet<String>,
  /// when asset was created from a source file (potentially transformed), the original filename relative to compilation context
  pub source_filename: Option<String>,
  /// when asset was created from a source file (potentially transformed), it should be flagged as copied
  pub copied: Option<bool>,
  /// size in bytes, only set after asset has been emitted
  // pub size: f64,
  /// when asset is only used for development and doesn't count towards user-facing assets
  pub development: Option<bool>,
  /// when asset ships data for updating an existing application (HMR)
  pub hot_module_replacement: Option<bool>,
  /// when asset is javascript and an ESM
  pub javascript_module: Option<bool>,
  /// related object to other assets, keyed by type of relation (only points from parent to child)
  pub related: AssetInfoRelated,
  /// the asset version, emit can be skipped when both filename and version are the same
  /// An empty string means no version, it will always emit
  pub version: String,
  /// unused local idents of the chunk
  pub css_unused_idents: Option<HashSet<String>>,
  /// Webpack: AssetInfo = KnownAssetInfo & Record<string, any>
  /// But Napi.rs does not support Intersectiont types. This is a hack to store the additional fields
  /// in the rust struct and have the Js side to reshape and align with webpack.
  /// Related: packages/rspack/src/Compilation.ts
  pub extras: serde_json::Map<String, serde_json::Value>,
  /// whether this asset is over the size limit
  pub is_over_size_limit: Option<bool>,
}

impl AssetInfo {
  pub fn with_minimized(mut self, v: Option<bool>) -> Self {
    self.minimized = v;
    self
  }

  pub fn with_development(mut self, v: Option<bool>) -> Self {
    self.development = v;
    self
  }

  pub fn with_hot_module_replacement(mut self, v: Option<bool>) -> Self {
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

  pub fn set_full_hash(&mut self, v: String) {
    self.full_hash.insert(v);
  }

  pub fn set_content_hash(&mut self, v: String) {
    self.content_hash.insert(v);
  }

  pub fn set_chunk_hash(&mut self, v: String) {
    self.chunk_hash.insert(v);
  }

  pub fn set_immutable(&mut self, v: Option<bool>) {
    self.immutable = v;
  }

  pub fn set_source_filename(&mut self, v: String) {
    self.source_filename = Some(v);
  }

  pub fn set_javascript_module(&mut self, v: bool) {
    self.javascript_module = Some(v);
  }

  pub fn set_css_unused_idents(&mut self, v: HashSet<String>) {
    self.css_unused_idents = Some(v);
  }

  pub fn set_is_over_size_limit(&mut self, v: bool) {
    self.is_over_size_limit = Some(v);
  }
  // another should have high priority than self
  // self = { immutable:true}
  // merge_another_asset({immutable: false})
  // self == { immutable: false}
  // align with https://github.com/webpack/webpack/blob/899f06934391baede59da3dcd35b5ef51c675dbe/lib/Compilation.js#L4554
  pub fn merge_another_asset(&mut self, another: AssetInfo) {
    // "another" first fields
    self.minimized = another.minimized;

    self.source_filename = another.source_filename.or(self.source_filename.take());
    self.version = another.version;
    self.related.merge_another(another.related);

    // merge vec fields
    self.chunk_hash.extend(another.chunk_hash);
    self.content_hash.extend(another.content_hash);
    self.extras.extend(another.extras);
    // self.full_hash.extend(another.full_hash.iter().cloned());
    // self.module_hash.extend(another.module_hash.iter().cloned());

    // old first fields or truthy first fields
    self.javascript_module = another.javascript_module.or(self.javascript_module.take());
    self.immutable = another.immutable.or(self.immutable);
    self.development = another.development.or(self.development);
    self.hot_module_replacement = another
      .hot_module_replacement
      .or(self.hot_module_replacement);
    self.is_over_size_limit = another.is_over_size_limit.or(self.is_over_size_limit);
  }
}

#[derive(Debug, Default, Clone)]
pub struct AssetInfoRelated {
  pub source_map: Option<String>,
}

impl AssetInfoRelated {
  pub fn merge_another(&mut self, another: AssetInfoRelated) {
    if let Some(source_map) = another.source_map {
      self.source_map = Some(source_map);
    }
  }
}

/// level order, the impl is different from webpack, since we can't iterate a set and mutate it at
/// the same time.
pub fn assign_depths(
  assign_map: &mut IdentifierMap<usize>,
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
  assign_map: &mut IdentifierMap<usize>,
  mg: &ModuleGraph,
  module_id: ModuleIdentifier,
) {
  // https://github.com/webpack/webpack/blob/1f99ad6367f2b8a6ef17cce0e058f7a67fb7db18/lib/Compilation.js#L3720
  let mut q = VecDeque::new();
  q.push_back(module_id);
  let mut depth;
  assign_map.insert(module_id, 0);
  let process_module = |m: ModuleIdentifier,
                        depth: usize,
                        q: &mut VecDeque<ModuleIdentifier>,
                        assign_map: &mut IdentifierMap<usize>| {
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
  assign_map: &mut IdentifierMap<usize>,
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

#[derive(Debug, Clone)]
pub struct RenderManifestEntry {
  pub source: BoxSource,
  filename: String,
  pub info: AssetInfo,
  // pub identifier: String,
  // hash?: string;
  pub(crate) auxiliary: bool,
  has_filename: bool, /* webpack only asset has filename, js/css/wasm has filename template */
}

impl RenderManifestEntry {
  pub fn new(
    source: BoxSource,
    filename: String,
    info: AssetInfo,
    auxiliary: bool,
    has_filename: bool,
  ) -> Self {
    Self {
      source,
      filename,
      info,
      auxiliary,
      has_filename,
    }
  }

  pub fn source(&self) -> &BoxSource {
    &self.source
  }

  pub fn filename(&self) -> &str {
    &self.filename
  }

  pub fn has_filename(&self) -> bool {
    self.has_filename
  }
}
