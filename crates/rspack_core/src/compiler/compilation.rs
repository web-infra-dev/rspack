use std::{
  collections::{hash_map, VecDeque},
  fmt::Debug,
  hash::{BuildHasherDefault, Hash},
  sync::{
    atomic::{AtomicBool, AtomicU32, Ordering},
    Arc,
  },
};

use dashmap::DashSet;
use indexmap::{IndexMap, IndexSet};
use itertools::Itertools;
use rayon::prelude::*;
use rspack_cacheable::{
  cacheable,
  with::{AsOption, AsPreset},
};
use rspack_collections::{
  DatabaseItem, Identifiable, IdentifierDashMap, IdentifierMap, IdentifierSet, UkeyMap, UkeySet,
};
use rspack_error::{
  error, miette::diagnostic, Diagnostic, DiagnosticExt, InternalError, Result, RspackSeverity,
  Severity, ToStringResultToRspackResultExt,
};
use rspack_fs::{IntermediateFileSystem, ReadableFileSystem, WritableFileSystem};
use rspack_hash::{RspackHash, RspackHashDigest};
use rspack_hook::define_hook;
use rspack_paths::ArcPath;
use rspack_sources::{BoxSource, CachedSource, SourceExt};
use rspack_tasks::CompilerContext;
use rspack_util::{itoa, tracing_preset::TRACING_BENCH_TARGET};
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet, FxHasher};
use tracing::instrument;

use super::{
  make::{make_module_graph, update_module_graph, MakeArtifact, MakeParam},
  module_executor::ModuleExecutor,
  rebuild::CompilationRecords,
  CompilerId,
};
use crate::{
  build_chunk_graph::{build_chunk_graph, build_chunk_graph_new},
  cache::Cache,
  get_runtime_key,
  incremental::{self, Incremental, IncrementalPasses, Mutation},
  is_source_equal,
  old_cache::{use_code_splitting_cache, Cache as OldCache, CodeSplittingCache},
  to_identifier, AsyncModulesArtifact, BindingCell, BoxDependency, BoxModule, CacheCount,
  CacheOptions, CgcRuntimeRequirementsArtifact, CgmHashArtifact, CgmRuntimeRequirementsArtifact,
  Chunk, ChunkByUkey, ChunkContentHash, ChunkGraph, ChunkGroupByUkey, ChunkGroupUkey,
  ChunkHashesArtifact, ChunkIdsArtifact, ChunkKind, ChunkRenderArtifact, ChunkRenderResult,
  ChunkUkey, CodeGenerationJob, CodeGenerationResult, CodeGenerationResults, CompilationLogger,
  CompilationLogging, CompilerOptions, DependenciesDiagnosticsArtifact, DependencyCodeGeneration,
  DependencyId, DependencyTemplate, DependencyTemplateType, DependencyType, Entry, EntryData,
  EntryOptions, EntryRuntime, Entrypoint, ExecuteModuleId, Filename, ImportVarMap, Logger,
  ModuleFactory, ModuleGraph, ModuleGraphCacheArtifact, ModuleGraphPartial, ModuleIdentifier,
  ModuleIdsArtifact, ModuleStaticCacheArtifact, PathData, ResolverFactory, RuntimeGlobals,
  RuntimeMode, RuntimeModule, RuntimeSpecMap, RuntimeTemplate, SharedPluginDriver,
  SideEffectsOptimizeArtifact, SourceType, Stats,
};

define_hook!(CompilationAddEntry: Series(compilation: &mut Compilation, entry_name: Option<&str>));
define_hook!(CompilationBuildModule: Series(compiler_id: CompilerId, compilation_id: CompilationId, module: &mut BoxModule),tracing=false);
define_hook!(CompilationRevokedModules: Series(compilation: &Compilation, revoked_modules: &IdentifierSet));
define_hook!(CompilationStillValidModule: Series(compiler_id: CompilerId, compilation_id: CompilationId, module: &mut BoxModule));
define_hook!(CompilationSucceedModule: Series(compiler_id: CompilerId, compilation_id: CompilationId, module: &mut BoxModule),tracing=false);
define_hook!(CompilationExecuteModule:
  Series(module: &ModuleIdentifier, runtime_modules: &IdentifierSet, code_generation_results: &BindingCell<CodeGenerationResults>, execute_module_id: &ExecuteModuleId));
define_hook!(CompilationFinishModules: Series(compilation: &mut Compilation));
define_hook!(CompilationSeal: Series(compilation: &mut Compilation));
define_hook!(CompilationOptimizeDependencies: SeriesBail(compilation: &mut Compilation) -> bool);
define_hook!(CompilationOptimizeModules: SeriesBail(compilation: &mut Compilation) -> bool);
define_hook!(CompilationAfterOptimizeModules: Series(compilation: &mut Compilation));
define_hook!(CompilationOptimizeChunks: SeriesBail(compilation: &mut Compilation) -> bool);
define_hook!(CompilationOptimizeTree: Series(compilation: &mut Compilation));
define_hook!(CompilationOptimizeChunkModules: SeriesBail(compilation: &mut Compilation) -> bool);
define_hook!(CompilationModuleIds: Series(compilation: &mut Compilation));
define_hook!(CompilationChunkIds: Series(compilation: &mut Compilation));
define_hook!(CompilationRuntimeModule: Series(compilation: &mut Compilation, module: &ModuleIdentifier, chunk: &ChunkUkey));
define_hook!(CompilationAdditionalModuleRuntimeRequirements: Series(compilation: &Compilation, module_identifier: &ModuleIdentifier, runtime_requirements: &mut RuntimeGlobals),tracing=false);
define_hook!(CompilationRuntimeRequirementInModule: SeriesBail(compilation: &Compilation, module_identifier: &ModuleIdentifier, all_runtime_requirements: &RuntimeGlobals, runtime_requirements: &RuntimeGlobals, runtime_requirements_mut: &mut RuntimeGlobals),tracing=false);
define_hook!(CompilationAdditionalChunkRuntimeRequirements: Series(compilation: &mut Compilation, chunk_ukey: &ChunkUkey, runtime_requirements: &mut RuntimeGlobals));
define_hook!(CompilationRuntimeRequirementInChunk: SeriesBail(compilation: &mut Compilation, chunk_ukey: &ChunkUkey, all_runtime_requirements: &RuntimeGlobals, runtime_requirements: &RuntimeGlobals, runtime_requirements_mut: &mut RuntimeGlobals));
define_hook!(CompilationAdditionalTreeRuntimeRequirements: Series(compilation: &mut Compilation, chunk_ukey: &ChunkUkey, runtime_requirements: &mut RuntimeGlobals));
define_hook!(CompilationRuntimeRequirementInTree: SeriesBail(compilation: &mut Compilation, chunk_ukey: &ChunkUkey, all_runtime_requirements: &RuntimeGlobals, runtime_requirements: &RuntimeGlobals, runtime_requirements_mut: &mut RuntimeGlobals));
define_hook!(CompilationOptimizeCodeGeneration: Series(compilation: &mut Compilation));
define_hook!(CompilationAfterCodeGeneration: Series(compilation: &mut Compilation));
define_hook!(CompilationChunkHash: Series(compilation: &Compilation, chunk_ukey: &ChunkUkey, hasher: &mut RspackHash),tracing=false);
define_hook!(CompilationContentHash: Series(compilation: &Compilation, chunk_ukey: &ChunkUkey, hashes: &mut HashMap<SourceType, RspackHash>));
define_hook!(CompilationDependentFullHash: SeriesBail(compilation: &Compilation, chunk_ukey: &ChunkUkey) -> bool);
define_hook!(CompilationRenderManifest: Series(compilation: &Compilation, chunk_ukey: &ChunkUkey, manifest: &mut Vec<RenderManifestEntry>, diagnostics: &mut Vec<Diagnostic>),tracing=false);
define_hook!(CompilationChunkAsset: Series(compilation: &Compilation, chunk_ukey: &ChunkUkey, filename: &str));
define_hook!(CompilationProcessAssets: Series(compilation: &mut Compilation));
define_hook!(CompilationAfterProcessAssets: Series(compilation: &mut Compilation));
define_hook!(CompilationAfterSeal: Series(compilation: &mut Compilation),tracing=true);

#[derive(Debug, Default)]
pub struct CompilationHooks {
  pub add_entry: CompilationAddEntryHook,
  pub build_module: CompilationBuildModuleHook,
  pub revoked_modules: CompilationRevokedModulesHook,
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
  pub additional_module_runtime_requirements: CompilationAdditionalModuleRuntimeRequirementsHook,
  pub runtime_requirement_in_module: CompilationRuntimeRequirementInModuleHook,
  pub additional_chunk_runtime_requirements: CompilationAdditionalChunkRuntimeRequirementsHook,
  pub runtime_requirement_in_chunk: CompilationRuntimeRequirementInChunkHook,
  pub additional_tree_runtime_requirements: CompilationAdditionalTreeRuntimeRequirementsHook,
  pub runtime_requirement_in_tree: CompilationRuntimeRequirementInTreeHook,
  pub optimize_code_generation: CompilationOptimizeCodeGenerationHook,
  pub after_code_generation: CompilationAfterCodeGenerationHook,
  pub chunk_hash: CompilationChunkHashHook,
  pub content_hash: CompilationContentHashHook,
  pub dependent_full_hash: CompilationDependentFullHashHook,
  pub render_manifest: CompilationRenderManifestHook,
  pub chunk_asset: CompilationChunkAssetHook,
  pub process_assets: CompilationProcessAssetsHook,
  pub after_process_assets: CompilationAfterProcessAssetsHook,
  pub after_seal: CompilationAfterSealHook,
}

#[cacheable]
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct CompilationId(pub u32);

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

/// Use macro to prevent cargo shear from failing and reporting errors
/// due to the inability to parse the async closure syntax
/// https://github.com/Boshen/cargo-shear/issues/143
macro_rules! process_runtime_requirement_hook_macro {
  ($name: ident, $s: ty, $c: ty) => {
    async fn $name(
      self: $s,
      requirements: &mut RuntimeGlobals,
      call_hook: impl async Fn(
        $c,
        &RuntimeGlobals,
        &RuntimeGlobals,
        &mut RuntimeGlobals,
      ) -> Result<()>,
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

#[derive(Debug)]
pub struct Compilation {
  /// get_compilation_hooks(compilation.id)
  id: CompilationId,
  compiler_id: CompilerId,
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
  pub dependency_templates: HashMap<DependencyTemplateType, Arc<dyn DependencyTemplate>>,
  pub runtime_modules: IdentifierMap<Box<dyn RuntimeModule>>,
  pub runtime_modules_hash: IdentifierMap<RspackHashDigest>,
  pub runtime_modules_code_generation_source: IdentifierMap<BoxSource>,
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
  pub buildtime_plugin_driver: SharedPluginDriver,
  pub resolver_factory: Arc<ResolverFactory>,
  pub loader_resolver_factory: Arc<ResolverFactory>,
  pub named_chunks: HashMap<String, ChunkUkey>,
  pub named_chunk_groups: HashMap<String, ChunkGroupUkey>,
  pub runtime_template: RuntimeTemplate,

  // artifact for infer_async_modules_plugin
  pub async_modules_artifact: AsyncModulesArtifact,
  // artifact for collect_dependencies_diagnostics
  pub dependencies_diagnostics_artifact: DependenciesDiagnosticsArtifact,
  // artifact for side_effects_flag_plugin
  pub side_effects_optimize_artifact: SideEffectsOptimizeArtifact,
  // artifact for module_ids
  pub module_ids_artifact: ModuleIdsArtifact,
  // artifact for chunk_ids
  pub chunk_ids_artifact: ChunkIdsArtifact,
  // artifact for code_generation
  pub code_generation_results: BindingCell<CodeGenerationResults>,
  // artifact for create_module_hashes
  pub cgm_hash_artifact: CgmHashArtifact,
  // artifact for process_modules_runtime_requirements
  pub cgm_runtime_requirements_artifact: CgmRuntimeRequirementsArtifact,
  // artifact for process_chunks_runtime_requirements
  pub cgc_runtime_requirements_artifact: CgcRuntimeRequirementsArtifact,
  // artifact for create_hash
  pub chunk_hashes_artifact: ChunkHashesArtifact,
  // artifact for create_chunk_assets
  pub chunk_render_artifact: ChunkRenderArtifact,
  // artifact for caching get_mode
  pub module_graph_cache_artifact: ModuleGraphCacheArtifact,
  // artiface for caching module static info
  pub module_static_cache_artifact: ModuleStaticCacheArtifact,

  pub code_generated_modules: IdentifierSet,
  pub build_time_executed_modules: IdentifierSet,
  pub cache: Arc<dyn Cache>,
  pub old_cache: Arc<OldCache>,
  pub code_splitting_cache: CodeSplittingCache,
  pub incremental: Incremental,

  pub hash: Option<RspackHashDigest>,

  pub file_dependencies: IndexSet<ArcPath, BuildHasherDefault<FxHasher>>,
  pub context_dependencies: IndexSet<ArcPath, BuildHasherDefault<FxHasher>>,
  pub missing_dependencies: IndexSet<ArcPath, BuildHasherDefault<FxHasher>>,
  pub build_dependencies: IndexSet<ArcPath, BuildHasherDefault<FxHasher>>,

  pub value_cache_versions: ValueCacheVersions,

  import_var_map: IdentifierDashMap<ImportVarMap>,

  pub module_executor: Option<ModuleExecutor>,
  in_finish_make: AtomicBool,

  pub modified_files: HashSet<ArcPath>,
  pub removed_files: HashSet<ArcPath>,
  pub make_artifact: MakeArtifact,
  pub input_filesystem: Arc<dyn ReadableFileSystem>,

  pub intermediate_filesystem: Arc<dyn IntermediateFileSystem>,
  pub output_filesystem: Arc<dyn WritableFileSystem>,

  /// A flag indicating whether the current compilation is being rebuilt.
  ///
  /// Rebuild will include previous compilation data, so persistent cache will not recovery anything
  pub is_rebuild: bool,
  pub compiler_context: Arc<CompilerContext>,
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
    compiler_id: CompilerId,
    options: Arc<CompilerOptions>,
    plugin_driver: SharedPluginDriver,
    buildtime_plugin_driver: SharedPluginDriver,
    resolver_factory: Arc<ResolverFactory>,
    loader_resolver_factory: Arc<ResolverFactory>,
    records: Option<CompilationRecords>,
    cache: Arc<dyn Cache>,
    old_cache: Arc<OldCache>,
    incremental: Incremental,
    module_executor: Option<ModuleExecutor>,
    modified_files: HashSet<ArcPath>,
    removed_files: HashSet<ArcPath>,
    input_filesystem: Arc<dyn ReadableFileSystem>,
    intermediate_filesystem: Arc<dyn IntermediateFileSystem>,
    output_filesystem: Arc<dyn WritableFileSystem>,
    is_rebuild: bool,
    compiler_context: Arc<CompilerContext>,
  ) -> Self {
    Self {
      id: CompilationId::new(),
      compiler_id,
      hot_index: 0,
      runtime_template: RuntimeTemplate::new(options.output.environment),
      records,
      options,
      other_module_graph: None,
      dependency_factories: Default::default(),
      dependency_templates: Default::default(),
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
      emitted_assets: Default::default(),
      diagnostics: Default::default(),
      logging: Default::default(),
      plugin_driver,
      buildtime_plugin_driver,
      resolver_factory,
      loader_resolver_factory,
      named_chunks: Default::default(),
      named_chunk_groups: Default::default(),

      async_modules_artifact: Default::default(),
      dependencies_diagnostics_artifact: Default::default(),
      side_effects_optimize_artifact: Default::default(),
      module_ids_artifact: Default::default(),
      chunk_ids_artifact: Default::default(),
      code_generation_results: Default::default(),
      cgm_hash_artifact: Default::default(),
      cgm_runtime_requirements_artifact: Default::default(),
      cgc_runtime_requirements_artifact: Default::default(),
      chunk_hashes_artifact: Default::default(),
      chunk_render_artifact: Default::default(),
      module_graph_cache_artifact: Default::default(),
      module_static_cache_artifact: Default::default(),
      code_generated_modules: Default::default(),
      build_time_executed_modules: Default::default(),
      cache,
      old_cache,
      incremental,
      code_splitting_cache: Default::default(),

      hash: None,

      file_dependencies: Default::default(),
      context_dependencies: Default::default(),
      missing_dependencies: Default::default(),
      build_dependencies: Default::default(),

      value_cache_versions: ValueCacheVersions::default(),

      import_var_map: IdentifierDashMap::default(),

      module_executor,
      in_finish_make: AtomicBool::new(false),

      make_artifact: Default::default(),
      modified_files,
      removed_files,
      input_filesystem,

      intermediate_filesystem,
      output_filesystem,
      is_rebuild,
      compiler_context,
    }
  }

  pub fn id(&self) -> CompilationId {
    self.id
  }

  pub fn compiler_id(&self) -> CompilerId {
    self.compiler_id
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
        [
          Some(self.make_artifact.get_module_graph_partial()),
          Some(other_module_graph),
        ],
        None,
      )
    } else {
      ModuleGraph::new(
        [Some(self.make_artifact.get_module_graph_partial()), None],
        None,
      )
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
        [Some(self.make_artifact.get_module_graph_partial()), None],
        Some(other),
      )
    } else {
      ModuleGraph::new(
        [None, None],
        Some(self.make_artifact.get_module_graph_partial_mut()),
      )
    }
  }

  pub fn file_dependencies(
    &self,
  ) -> (
    impl Iterator<Item = &ArcPath>,
    impl Iterator<Item = &ArcPath>,
    impl Iterator<Item = &ArcPath>,
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
    impl Iterator<Item = &ArcPath>,
    impl Iterator<Item = &ArcPath>,
    impl Iterator<Item = &ArcPath>,
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
    impl Iterator<Item = &ArcPath>,
    impl Iterator<Item = &ArcPath>,
    impl Iterator<Item = &ArcPath>,
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
    impl Iterator<Item = &ArcPath>,
    impl Iterator<Item = &ArcPath>,
    impl Iterator<Item = &ArcPath>,
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

  pub async fn add_entry_batch(&mut self, args: Vec<(BoxDependency, EntryOptions)>) -> Result<()> {
    for (entry, options) in args {
      self.add_entry(entry, options).await?;
    }

    let make_artifact = std::mem::take(&mut self.make_artifact);
    self.make_artifact = update_module_graph(
      self,
      make_artifact,
      vec![MakeParam::BuildEntry(
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

    Ok(())
  }

  pub async fn add_include(&mut self, args: Vec<(BoxDependency, EntryOptions)>) -> Result<()> {
    if !self.in_finish_make.load(Ordering::Acquire) {
      return Err(
        InternalError::new(
          "You can only call `add_include` during the finish make stage".to_string(),
          RspackSeverity::Error,
        )
        .into(),
      );
    }

    for (entry, options) in args {
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
    }

    // Recheck entry and clean useless entry
    // This should before finish_modules hook is called, ensure providedExports effects on new added modules
    let make_artifact = std::mem::take(&mut self.make_artifact);
    self.make_artifact = update_module_graph(
      self,
      make_artifact,
      vec![MakeParam::BuildEntry(
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

    Ok(())
  }

  pub fn update_asset(
    &mut self,
    filename: &str,
    updater: impl FnOnce(
      BoxSource,
      BindingCell<AssetInfo>,
    ) -> Result<(BoxSource, BindingCell<AssetInfo>)>,
  ) -> Result<()> {
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
  #[instrument("Compilation:emit_asset",skip_all, fields(filename = filename))]
  pub fn emit_asset(&mut self, filename: String, asset: CompilationAsset) {
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
      if let Some(source_map) = &asset.info.related.source_map {
        self.delete_asset(source_map);
      }
      self.chunk_by_ukey.iter_mut().for_each(|(_, chunk)| {
        chunk.remove_file(filename);
        chunk.remove_auxiliary_file(filename);
      });
    }
  }

  pub fn rename_asset(&mut self, filename: &str, new_name: String) {
    if let Some(asset) = self.assets.remove(filename) {
      self.assets.insert(new_name.clone(), asset);
      self.chunk_by_ukey.iter_mut().for_each(|(_, chunk)| {
        if chunk.remove_file(filename) {
          chunk.add_file(new_name.clone());
        }

        if chunk.remove_auxiliary_file(filename) {
          chunk.add_auxiliary_file(new_name.clone());
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

  pub fn extend_diagnostics(&mut self, diagnostics: impl IntoIterator<Item = Diagnostic>) {
    self.diagnostics.extend(diagnostics);
  }

  pub fn diagnostics(&self) -> &[Diagnostic] {
    &self.diagnostics
  }

  pub fn diagnostics_mut(&mut self) -> &mut Vec<Diagnostic> {
    &mut self.diagnostics
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
  ) -> (ChunkUkey, bool) {
    let existed_chunk_ukey = named_chunks.get(&name);
    if let Some(chunk_ukey) = existed_chunk_ukey {
      assert!(chunk_by_ukey.contains(chunk_ukey));
      (*chunk_ukey, false)
    } else {
      let chunk = Chunk::new(Some(name.clone()), ChunkKind::Normal);
      let ukey = chunk.ukey();
      named_chunks.insert(name, ukey);
      chunk_by_ukey.entry(ukey).or_insert_with(|| chunk);
      (ukey, true)
    }
  }

  pub fn add_chunk(chunk_by_ukey: &mut ChunkByUkey) -> ChunkUkey {
    let chunk = Chunk::new(None, ChunkKind::Normal);
    let ukey = chunk.ukey();
    chunk_by_ukey.add(chunk);
    ukey
  }

  #[instrument("Compilation:make",target=TRACING_BENCH_TARGET, skip_all)]
  pub async fn make(&mut self) -> Result<()> {
    self.make_artifact.reset_dependencies_incremental_info();
    // run module_executor
    if let Some(module_executor) = &mut self.module_executor {
      let mut module_executor = std::mem::take(module_executor);
      module_executor.hook_before_make(self).await?;
      self.module_executor = Some(module_executor);
    }

    let artifact = std::mem::take(&mut self.make_artifact);
    self.make_artifact = make_module_graph(self, artifact).await?;

    self.in_finish_make.store(true, Ordering::Release);

    Ok(())
  }

  pub async fn rebuild_module<T>(
    &mut self,
    module_identifiers: IdentifierSet,
    f: impl Fn(Vec<&BoxModule>) -> T,
  ) -> Result<T> {
    let artifact = std::mem::take(&mut self.make_artifact);

    // https://github.com/webpack/webpack/blob/19ca74127f7668aaf60d59f4af8fcaee7924541a/lib/Compilation.js#L2462C21-L2462C25
    self.module_graph_cache_artifact.unfreeze();

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

  #[instrument("Compilation:code_generation",target=TRACING_BENCH_TARGET, skip_all)]
  async fn code_generation(&mut self, modules: IdentifierSet) -> Result<()> {
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

    self
      .code_generation_modules(&mut codegen_cache_counter, no_codegen_dependencies_modules)
      .await?;
    self
      .code_generation_modules(&mut codegen_cache_counter, has_codegen_dependencies_modules)
      .await?;

    if let Some(counter) = codegen_cache_counter {
      logger.cache_end(counter);
    }

    Ok(())
  }

  pub(crate) async fn code_generation_modules(
    &mut self,
    cache_counter: &mut Option<CacheCount>,
    modules: IdentifierSet,
  ) -> Result<()> {
    let chunk_graph = &self.chunk_graph;
    let module_graph = self.get_module_graph();
    let mut jobs = Vec::new();
    for module in modules {
      let mut map: HashMap<RspackHashDigest, CodeGenerationJob> = HashMap::default();
      for runtime in chunk_graph.get_module_runtimes_iter(module, &self.chunk_by_ukey) {
        let hash = ChunkGraph::get_module_hash(self, module, runtime)
          .expect("should have cgm.hash in code generation");
        if let Some(job) = map.get_mut(hash) {
          job.runtimes.push(runtime.clone());
        } else {
          map.insert(
            hash.clone(),
            CodeGenerationJob {
              module,
              hash: hash.clone(),
              runtime: runtime.clone(),
              runtimes: vec![runtime.clone()],
            },
          );
        }
      }
      jobs.extend(map.into_values());
    }

    let results = rspack_futures::scope::<_, _>(|token| {
      jobs.into_iter().for_each(|job| {
        // SAFETY: await immediately and trust caller to poll future entirely
        let s = unsafe { token.used((&self, &module_graph, job)) };

        s.spawn(|(this, module_graph, job)| async {
          let options = &this.options;
          let old_cache = &this.old_cache;

          let module = module_graph
            .module_by_identifier(&job.module)
            .expect("should have module");
          let codegen_res = old_cache
            .code_generate_occasion
            .use_cache(&job, || async {
              module
                .code_generation(this, Some(&job.runtime), None)
                .await
                .map(|mut codegen_res| {
                  codegen_res.set_hash(
                    &options.output.hash_function,
                    &options.output.hash_digest,
                    &options.output.hash_salt,
                  );
                  codegen_res
                })
            })
            .await;

          (job.module, job.runtimes, codegen_res)
        })
      })
    })
    .await;
    let results = results
      .into_iter()
      .map(|res| res.to_rspack_result())
      .collect::<Result<Vec<_>>>()?;

    for (module, runtimes, (codegen_res, from_cache)) in results {
      if let Some(counter) = cache_counter {
        if from_cache {
          counter.hit();
        } else {
          counter.miss();
        }
      }
      let codegen_res = match codegen_res {
        Ok(codegen_res) => codegen_res,
        Err(err) => {
          self.push_diagnostic(Diagnostic::from(err).with_module_identifier(Some(module)));
          let mut codegen_res = CodeGenerationResult::default();
          codegen_res.set_hash(
            &self.options.output.hash_function,
            &self.options.output.hash_digest,
            &self.options.output.hash_salt,
          );
          codegen_res
        }
      };
      self
        .code_generation_results
        .insert(module, codegen_res, runtimes);
      self.code_generated_modules.insert(module);
    }
    Ok(())
  }

  #[instrument("Compilation:create_module_assets",target=TRACING_BENCH_TARGET, skip_all)]
  async fn create_module_assets(&mut self, _plugin_driver: SharedPluginDriver) {
    let mut chunk_asset_map = vec![];
    let mut module_assets = vec![];
    let mg = self.get_module_graph();
    for (identifier, module) in mg.modules() {
      let assets = &module.build_info().assets;
      if assets.is_empty() {
        continue;
      }

      for (name, asset) in assets.as_ref() {
        module_assets.push((name.clone(), asset.clone()));
      }
      // assets of executed modules are not in this compilation
      if self
        .chunk_graph
        .chunk_graph_module_by_module_identifier
        .contains_key(&identifier)
      {
        for chunk in self.chunk_graph.get_module_chunks(identifier).iter() {
          for name in assets.keys() {
            chunk_asset_map.push((*chunk, name.clone()))
          }
        }
      }
    }

    for (name, asset) in module_assets {
      self.emit_asset(name, asset);
    }

    for (chunk, asset_name) in chunk_asset_map {
      let chunk = self.chunk_by_ukey.expect_get_mut(&chunk);
      chunk.add_auxiliary_file(asset_name);
    }
  }

  #[instrument("Compilation::create_chunk_assets",target=TRACING_BENCH_TARGET, skip_all)]
  async fn create_chunk_assets(&mut self, plugin_driver: SharedPluginDriver) -> Result<()> {
    if self.options.output.filename.has_hash_placeholder()
      || self.options.output.chunk_filename.has_hash_placeholder()
      || self.options.output.css_filename.has_hash_placeholder()
      || self
        .options
        .output
        .css_chunk_filename
        .has_hash_placeholder()
    {
      if let Some(diagnostic) = self.incremental.disable_passes(
        IncrementalPasses::CHUNKS_RENDER,
        "Chunk filename that dependent on full hash",
        "chunk filename that dependent on full hash is not supported in incremental compilation",
      ) {
        if let Some(diagnostic) = diagnostic {
          self.push_diagnostic(diagnostic);
        }
        self.chunk_render_artifact.clear();
      }
    }

    let chunks = if let Some(mutations) = self
      .incremental
      .mutations_read(IncrementalPasses::CHUNKS_RENDER)
      && !self.chunk_render_artifact.is_empty()
    {
      let removed_chunks = mutations.iter().filter_map(|mutation| match mutation {
        Mutation::ChunkRemove { chunk } => Some(*chunk),
        _ => None,
      });
      for removed_chunk in removed_chunks {
        self.chunk_render_artifact.remove(&removed_chunk);
      }
      self
        .chunk_render_artifact
        .retain(|chunk, _| self.chunk_by_ukey.contains(chunk));
      let chunks: UkeySet<ChunkUkey> = mutations
        .iter()
        .filter_map(|mutation| match mutation {
          Mutation::ChunkSetHashes { chunk } => Some(*chunk),
          _ => None,
        })
        .collect();
      tracing::debug!(target: incremental::TRACING_TARGET, passes = %IncrementalPasses::CHUNKS_RENDER, %mutations);
      let logger = self.get_logger("rspack.incremental.chunksRender");
      logger.log(format!(
        "{} chunks are affected, {} in total",
        chunks.len(),
        self.chunk_by_ukey.len()
      ));
      chunks
    } else {
      self.chunk_by_ukey.keys().copied().collect()
    };
    let results = rspack_futures::scope::<_, Result<_>>(|token| {
      chunks.iter().for_each(|chunk| {
        // SAFETY: await immediately and trust caller to poll future entirely
        let s = unsafe { token.used((&self, &plugin_driver, chunk)) };

        s.spawn(|(this, plugin_driver, chunk)| async {
          let mut manifests = Vec::new();
          let mut diagnostics = Vec::new();
          plugin_driver
            .compilation_hooks
            .render_manifest
            .call(this, chunk, &mut manifests, &mut diagnostics)
            .await?;

          rspack_error::Result::Ok((
            *chunk,
            ChunkRenderResult {
              manifests,
              diagnostics,
            },
          ))
        });
      })
    })
    .await;

    let mut chunk_render_results: UkeyMap<ChunkUkey, ChunkRenderResult> = Default::default();
    for result in results {
      let item = result.to_rspack_result()?;
      let (key, value) = item?;
      chunk_render_results.insert(key, value);
    }
    let chunk_ukey_and_manifest = if self
      .incremental
      .passes_enabled(IncrementalPasses::CHUNKS_RENDER)
    {
      self.chunk_render_artifact.extend(chunk_render_results);
      self.chunk_render_artifact.clone()
    } else {
      chunk_render_results
    };

    for (
      chunk_ukey,
      ChunkRenderResult {
        manifests,
        diagnostics,
      },
    ) in chunk_ukey_and_manifest
    {
      self.extend_diagnostics(diagnostics);

      for file_manifest in manifests {
        let filename = file_manifest.filename;
        let current_chunk = self.chunk_by_ukey.expect_get_mut(&chunk_ukey);

        current_chunk.set_rendered(true);
        if file_manifest.auxiliary {
          current_chunk.add_auxiliary_file(filename.clone());
        } else {
          current_chunk.add_file(filename.clone());
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

    Ok(())
  }

  #[instrument("Compilation:process_assets",target=TRACING_BENCH_TARGET, skip_all)]
  async fn process_assets(&mut self, plugin_driver: SharedPluginDriver) -> Result<()> {
    plugin_driver
      .compilation_hooks
      .process_assets
      .call(self)
      .await
      .map_err(|e| e.wrap_err("caused by plugins in Compilation.hooks.processAssets"))
  }

  #[instrument("Compilation:after_process_asssets", skip_all)]
  async fn after_process_assets(&mut self, plugin_driver: SharedPluginDriver) -> Result<()> {
    plugin_driver
      .compilation_hooks
      .after_process_assets
      .call(self)
      .await
  }

  #[instrument("Compilation:after_seal", target=TRACING_BENCH_TARGET,skip_all)]
  async fn after_seal(&mut self, plugin_driver: SharedPluginDriver) -> Result<()> {
    plugin_driver.compilation_hooks.after_seal.call(self).await
  }

  // #[instrument(
  //   name = "Compilation:chunk_asset",
  //   skip(self, plugin_driver, chunk_ukey)
  // )]
  async fn chunk_asset(
    &self,
    chunk_ukey: ChunkUkey,
    filename: &str,
    plugin_driver: SharedPluginDriver,
  ) -> Result<()> {
    plugin_driver
      .compilation_hooks
      .chunk_asset
      .call(self, &chunk_ukey, filename)
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

  #[instrument("Compilation:finish",target=TRACING_BENCH_TARGET, skip_all)]
  pub async fn finish(&mut self, plugin_driver: SharedPluginDriver) -> Result<()> {
    // clean up the entry deps
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

    let logger = self.get_logger("rspack.Compilation");

    self.in_finish_make.store(false, Ordering::Release);

    // take built_modules
    if let Some(mutations) = self.incremental.mutations_write() {
      mutations.extend(
        self
          .make_artifact
          .revoked_modules
          .difference(&self.make_artifact.built_modules)
          .map(|&module| Mutation::ModuleRemove { module }),
      );
      mutations.extend(
        self
          .make_artifact
          .built_modules
          .intersection(&self.make_artifact.revoked_modules)
          .map(|&module| Mutation::ModuleUpdate { module }),
      );
      mutations.extend(
        self
          .make_artifact
          .built_modules
          .difference(&self.make_artifact.revoked_modules)
          .map(|&module| Mutation::ModuleAdd { module }),
      );
      tracing::debug!(target: incremental::TRACING_TARGET, passes = %IncrementalPasses::MAKE, %mutations);
    }

    let start = logger.time("finish modules");
    // finish_modules means the module graph (modules, connections, dependencies) are
    // frozen and start to optimize (provided exports, infer async, etc.) based on the
    // module graph, so any kind of change that affect these should be done before the
    // finish_modules
    plugin_driver
      .compilation_hooks
      .finish_modules
      .call(self)
      .await?;

    // sync assets to compilation from module_executor
    if let Some(module_executor) = &mut self.module_executor {
      let mut module_executor = std::mem::take(module_executor);
      module_executor.hook_after_finish_modules(self).await?;
      self.module_executor = Some(module_executor);
    }

    logger.time_end(start);

    // https://github.com/webpack/webpack/blob/19ca74127f7668aaf60d59f4af8fcaee7924541a/lib/Compilation.js#L2988
    self.module_graph_cache_artifact.freeze();
    // Collect dependencies diagnostics at here to make sure:
    // 1. after finish_modules: has provide exports info
    // 2. before optimize dependencies: side effects free module hasn't been skipped
    self.collect_dependencies_diagnostics();
    self.module_graph_cache_artifact.unfreeze();

    // take make diagnostics
    let diagnostics = self.make_artifact.diagnostics();
    self.extend_diagnostics(diagnostics);
    Ok(())
  }

  #[tracing::instrument("Compilation:collect_dependencies_diagnostics", skip_all)]
  fn collect_dependencies_diagnostics(&mut self) {
    let mutations = self
      .incremental
      .mutations_read(IncrementalPasses::DEPENDENCIES_DIAGNOSTICS);
    // TODO move diagnostic collect to make
    let modules = if let Some(mutations) = mutations
      && !self.dependencies_diagnostics_artifact.is_empty()
    {
      let revoked_modules = mutations.iter().filter_map(|mutation| match mutation {
        Mutation::ModuleRemove { module } => Some(*module),
        _ => None,
      });
      for revoked_module in revoked_modules {
        self
          .dependencies_diagnostics_artifact
          .remove(&revoked_module);
      }
      let modules = mutations.get_affected_modules_with_module_graph(&self.get_module_graph());
      let logger = self.get_logger("rspack.incremental.dependenciesDiagnostics");
      logger.log(format!(
        "{} modules are affected, {} in total",
        modules.len(),
        self.get_module_graph().modules().len()
      ));
      modules
    } else {
      self.get_module_graph().modules().keys().copied().collect()
    };
    let module_graph = self.get_module_graph();
    let module_graph_cache = &self.module_graph_cache_artifact;
    let dependencies_diagnostics: DependenciesDiagnosticsArtifact = modules
      .par_iter()
      .map(|module_identifier| {
        let mgm = module_graph
          .module_graph_module_by_identifier(module_identifier)
          .expect("should have mgm");
        let diagnostics = mgm
          .all_dependencies
          .iter()
          .filter_map(|dependency_id| module_graph.dependency_by_id(dependency_id))
          .filter_map(|dependency| {
            dependency
              .get_diagnostics(&module_graph, module_graph_cache)
              .map(|diagnostics| {
                diagnostics.into_iter().map(|diagnostic| {
                  diagnostic
                    .with_module_identifier(Some(*module_identifier))
                    .with_loc(dependency.loc())
                })
              })
          })
          .flatten()
          .collect::<Vec<_>>();
        (*module_identifier, diagnostics)
      })
      .collect();
    let all_modules_diagnostics = if mutations.is_some() {
      self
        .dependencies_diagnostics_artifact
        .extend(dependencies_diagnostics);
      self.dependencies_diagnostics_artifact.clone()
    } else {
      dependencies_diagnostics
    };
    self.extend_diagnostics(all_modules_diagnostics.into_values().flatten());
  }

  #[instrument("Compilation:seal", skip_all)]
  pub async fn seal(&mut self, plugin_driver: SharedPluginDriver) -> Result<()> {
    self.other_module_graph = Some(ModuleGraphPartial::default());

    if !self.options.mode.is_development() {
      self.module_static_cache_artifact.freeze();
    }

    let logger = self.get_logger("rspack.Compilation");

    // https://github.com/webpack/webpack/blob/main/lib/Compilation.js#L2809
    plugin_driver
      .compilation_hooks
      .seal
      .call(self)
      .await
      .map_err(|e| e.wrap_err("caused by plugins in Compilation.hooks.seal"))?;

    let start = logger.time("optimize dependencies");
    // https://github.com/webpack/webpack/blob/d15c73469fd71cf98734685225250148b68ddc79/lib/Compilation.js#L2812-L2814

    while matches!(
      plugin_driver
        .compilation_hooks
        .optimize_dependencies
        .call(self)
        .await
        .map_err(|e| e.wrap_err("caused by plugins in Compilation.hooks.optimizeDependencies"))?,
      Some(true)
    ) {}

    logger.time_end(start);

    // ModuleGraph is frozen for now on, we have a module graph that won't change
    // so now we can start to create a chunk graph based on the module graph

    let start = logger.time("create chunks");
    self.module_graph_cache_artifact.freeze();
    use_code_splitting_cache(self, |compilation| async {
      let start = logger.time("rebuild chunk graph");
      if compilation.options.experiments.parallel_code_splitting {
        build_chunk_graph_new(compilation)?;
      } else {
        build_chunk_graph(compilation)?;
      }
      compilation
        .chunk_graph
        .generate_dot(compilation, "after-code-splitting");
      logger.time_end(start);
      Ok(compilation)
    })
    .await?;

    while matches!(
      plugin_driver
        .compilation_hooks
        .optimize_modules
        .call(self)
        .await
        .map_err(|e| e.wrap_err("caused by plugins in Compilation.hooks.optimizeModules"))?,
      Some(true)
    ) {}

    plugin_driver
      .compilation_hooks
      .after_optimize_modules
      .call(self)
      .await
      .map_err(|e| e.wrap_err("caused by plugins in Compilation.hooks.afterOptimizeModules"))?;

    while matches!(
      plugin_driver
        .compilation_hooks
        .optimize_chunks
        .call(self)
        .await
        .map_err(|e| e.wrap_err("caused by plugins in Compilation.hooks.optimizeChunks"))?,
      Some(true)
    ) {}

    logger.time_end(start);

    let start = logger.time("optimize");
    plugin_driver
      .compilation_hooks
      .optimize_tree
      .call(self)
      .await
      .map_err(|e| e.wrap_err("caused by plugins in Compilation.hooks.optimizeTree"))?;

    plugin_driver
      .compilation_hooks
      .optimize_chunk_modules
      .call(self)
      .await
      .map_err(|e| e.wrap_err("caused by plugins in Compilation.hooks.optimizeChunkModules"))?;
    logger.time_end(start);

    // ChunkGraph is frozen for now on, we have a chunk graph that won't change
    // so now we can start to generate assets based on the chunk graph

    let start = logger.time("module ids");

    plugin_driver
      .compilation_hooks
      .module_ids
      .call(self)
      .await
      .map_err(|e| e.wrap_err("caused by plugins in Compilation.hooks.moduleIds"))?;
    logger.time_end(start);

    let start = logger.time("chunk ids");
    plugin_driver
      .compilation_hooks
      .chunk_ids
      .call(self)
      .await
      .map_err(|e| e.wrap_err("caused by plugins in Compilation.hooks.chunkIds"))?;
    logger.time_end(start);

    self.assign_runtime_ids();

    let start = logger.time("optimize code generation");
    plugin_driver
      .compilation_hooks
      .optimize_code_generation
      .call(self)
      .await
      .map_err(|e| e.wrap_err("caused by plugins in Compilation.hooks.optimizeCodeGeneration"))?;
    logger.time_end(start);

    let create_module_hashes_modules = if let Some(mutations) = self
      .incremental
      .mutations_read(IncrementalPasses::MODULES_HASHES)
      && !self.cgm_hash_artifact.is_empty()
    {
      let revoked_modules = mutations.iter().filter_map(|mutation| match mutation {
        Mutation::ModuleRemove { module } => Some(*module),
        _ => None,
      });
      for revoked_module in revoked_modules {
        self.cgm_hash_artifact.remove(&revoked_module);
      }
      let mg = self.get_module_graph();
      let mut modules = mutations.get_affected_modules_with_module_graph(&mg);
      for mutation in mutations.iter() {
        match mutation {
          Mutation::ModuleSetAsync { module } => {
            modules.insert(*module);
          }
          Mutation::ModuleSetId { module } => {
            modules.insert(*module);
            modules.extend(
              mg.get_incoming_connections(module)
                .filter_map(|c| c.original_module_identifier),
            );
          }
          Mutation::ChunkAdd { chunk } => {
            modules.extend(self.chunk_graph.get_chunk_modules_identifier(chunk));
          }
          Mutation::ChunkSetId { chunk } => {
            let chunk = self.chunk_by_ukey.expect_get(chunk);
            modules.extend(
              chunk
                .groups()
                .iter()
                .flat_map(|group| {
                  let group = self.chunk_group_by_ukey.expect_get(group);
                  group.origins()
                })
                .filter_map(|origin| origin.module),
            );
          }
          _ => {}
        }
      }

      // check if module runtime changes
      for mi in mg.modules().keys() {
        let module_runtimes = self
          .chunk_graph
          .get_module_runtimes(*mi, &self.chunk_by_ukey);
        let module_runtime_keys = module_runtimes
          .values()
          .map(get_runtime_key)
          .collect::<Vec<_>>();

        if let Some(runtime_map) = self.cgm_hash_artifact.get_runtime_map(mi) {
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
              if !module_runtime_keys.contains(&runtime_key.as_str()) {
                modules.insert(*mi);
                break;
              }
            }
          }
        }
      }

      tracing::debug!(target: incremental::TRACING_TARGET, passes = %IncrementalPasses::MODULES_HASHES, %mutations, ?modules);
      let logger = self.get_logger("rspack.incremental.modulesHashes");
      logger.log(format!(
        "{} modules are affected, {} in total",
        modules.len(),
        mg.modules().len()
      ));

      modules
    } else {
      self.get_module_graph().modules().keys().copied().collect()
    };
    self
      .create_module_hashes(create_module_hashes_modules)
      .await?;

    let start = logger.time("code generation");
    let code_generation_modules = if let Some(mutations) = self
      .incremental
      .mutations_read(IncrementalPasses::MODULES_CODEGEN)
      && !self.code_generation_results.is_empty()
    {
      let revoked_modules = mutations.iter().filter_map(|mutation| match mutation {
        Mutation::ModuleRemove { module } => Some(*module),
        _ => None,
      });
      for revoked_module in revoked_modules {
        self.code_generation_results.remove(&revoked_module);
      }
      let modules: IdentifierSet = mutations
        .iter()
        .filter_map(|mutation| match mutation {
          Mutation::ModuleSetHashes { module } => Some(*module),
          _ => None,
        })
        .collect();
      // also cleanup for updated modules, for `insert(); insert();` the second insert() won't override the first insert() on code_generation_results
      for module in &modules {
        self.code_generation_results.remove(module);
      }
      tracing::debug!(target: incremental::TRACING_TARGET, passes = %IncrementalPasses::MODULES_CODEGEN, %mutations);
      let logger = self.get_logger("rspack.incremental.modulesCodegen");
      logger.log(format!(
        "{} modules are affected, {} in total",
        modules.len(),
        self.get_module_graph().modules().len()
      ));
      modules
    } else {
      self.get_module_graph().modules().keys().copied().collect()
    };
    self.code_generation(code_generation_modules).await?;

    plugin_driver
      .compilation_hooks
      .after_code_generation
      .call(self)
      .await
      .map_err(|e| e.wrap_err("caused by plugins in Compilation.hooks.afterCodeGeneration"))?;
    logger.time_end(start);

    let start = logger.time("runtime requirements");
    let process_runtime_requirements_modules = if let Some(mutations) = self
      .incremental
      .mutations_read(IncrementalPasses::MODULES_RUNTIME_REQUIREMENTS)
      && !self.cgm_runtime_requirements_artifact.is_empty()
    {
      let revoked_modules = mutations.iter().filter_map(|mutation| match mutation {
        Mutation::ModuleRemove { module } => Some(*module),
        _ => None,
      });
      for revoked_module in revoked_modules {
        self
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
      let logger = self.get_logger("rspack.incremental.modulesRuntimeRequirements");
      logger.log(format!(
        "{} modules are affected, {} in total",
        modules.len(),
        self.get_module_graph().modules().len()
      ));
      modules
    } else {
      self.get_module_graph().modules().keys().copied().collect()
    };
    self
      .process_modules_runtime_requirements(
        process_runtime_requirements_modules,
        plugin_driver.clone(),
      )
      .await?;
    let runtime_chunks = self.get_chunk_graph_entries().collect();
    let process_runtime_requirements_chunks = if let Some(mutations) = self
      .incremental
      .mutations_read(IncrementalPasses::CHUNKS_RUNTIME_REQUIREMENTS)
      && !self.cgc_runtime_requirements_artifact.is_empty()
    {
      let removed_chunks = mutations.iter().filter_map(|mutation| match mutation {
        Mutation::ChunkRemove { chunk } => Some(chunk),
        _ => None,
      });
      for removed_chunk in removed_chunks {
        self.cgc_runtime_requirements_artifact.remove(removed_chunk);
      }
      let affected_chunks = mutations.get_affected_chunks_with_chunk_graph(self);
      for affected_chunk in &affected_chunks {
        self
          .cgc_runtime_requirements_artifact
          .remove(affected_chunk);
      }
      for runtime_chunk in &runtime_chunks {
        self.cgc_runtime_requirements_artifact.remove(runtime_chunk);
      }
      self
        .cgc_runtime_requirements_artifact
        .retain(|chunk, _| self.chunk_by_ukey.contains(chunk));
      let logger = self.get_logger("rspack.incremental.chunksRuntimeRequirements");
      logger.log(format!(
        "{} chunks are affected, {} in total",
        affected_chunks.len(),
        self.chunk_by_ukey.len()
      ));
      affected_chunks
    } else {
      self.chunk_by_ukey.keys().copied().collect()
    };
    self
      .process_chunks_runtime_requirements(
        process_runtime_requirements_chunks,
        runtime_chunks,
        plugin_driver.clone(),
      )
      .await?;
    logger.time_end(start);

    let start = logger.time("hashing");
    self.create_hash(plugin_driver.clone()).await?;
    self.runtime_modules_code_generation().await?;
    logger.time_end(start);

    let start = logger.time("create module assets");
    self.create_module_assets(plugin_driver.clone()).await;
    logger.time_end(start);

    let start = logger.time("create chunk assets");
    self.create_chunk_assets(plugin_driver.clone()).await?;
    logger.time_end(start);

    let start = logger.time("process assets");
    self.process_assets(plugin_driver.clone()).await?;
    logger.time_end(start);

    let start = logger.time("after process assets");
    self.after_process_assets(plugin_driver.clone()).await?;
    logger.time_end(start);

    let start = logger.time("after seal");
    self.after_seal(plugin_driver).await?;
    logger.time_end(start);

    if !self.options.mode.is_development() {
      self.module_static_cache_artifact.unfreeze();
    }
    Ok(())
  }

  pub fn assign_runtime_ids(&mut self) {
    fn process_entrypoint(
      entrypoint_ukey: &ChunkGroupUkey,
      chunk_group_by_ukey: &ChunkGroupByUkey,
      chunk_by_ukey: &ChunkByUkey,
      chunk_ids: &ChunkIdsArtifact,
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
        chunk_by_ukey.get(&entrypoint.get_runtime_chunk(chunk_group_by_ukey)),
      ) {
        chunk_graph.set_runtime_id(runtime, chunk.id(chunk_ids).map(|id| id.to_string()));
      }
    }
    for i in self.entrypoints.iter() {
      process_entrypoint(
        i.1,
        &self.chunk_group_by_ukey,
        &self.chunk_by_ukey,
        &self.chunk_ids_artifact,
        &mut self.chunk_graph,
      )
    }
    for i in self.async_entrypoints.iter() {
      process_entrypoint(
        i,
        &self.chunk_group_by_ukey,
        &self.chunk_by_ukey,
        &self.chunk_ids_artifact,
        &mut self.chunk_graph,
      )
    }
  }

  pub fn get_chunk_graph_entries(&self) -> impl Iterator<Item = ChunkUkey> + use<'_> {
    let entries = self.entrypoints.values().map(|entrypoint_ukey| {
      let entrypoint = self.chunk_group_by_ukey.expect_get(entrypoint_ukey);
      entrypoint.get_runtime_chunk(&self.chunk_group_by_ukey)
    });
    let async_entries = self.async_entrypoints.iter().map(|entrypoint_ukey| {
      let entrypoint = self.chunk_group_by_ukey.expect_get(entrypoint_ukey);
      entrypoint.get_runtime_chunk(&self.chunk_group_by_ukey)
    });
    entries.chain(async_entries)
  }

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
        .filter(|module| self.chunk_graph.get_number_of_module_chunks(*module) > 0)
        .for_each(|module| {
          let s = unsafe { token.used((&self, &plugin_driver)) };
          s.spawn(move |(compilation, plugin_driver)| async move {
            let mut map = RuntimeSpecMap::new();
            let runtimes = compilation
              .chunk_graph
              .get_module_runtimes_iter(module, &compilation.chunk_by_ukey);
            for runtime in runtimes {
              let runtime_requirements = compilation
                .old_cache
                .process_runtime_requirements_occasion
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
                    .process_runtime_requirement_hook(&mut runtime_requirements, {
                      let plugin_driver = plugin_driver.clone();
                      async move |compilation,
                                  all_runtime_requirements,
                                  runtime_requirements,
                                  runtime_requirements_mut| {
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
    let mut chunk_requirements = HashMap::default();
    for chunk_ukey in chunks.iter().chain(entries.iter()) {
      let mut set = RuntimeGlobals::default();
      for module in self
        .chunk_graph
        .get_chunk_modules(chunk_ukey, &self.get_module_graph())
      {
        let chunk = self.chunk_by_ukey.expect_get(chunk_ukey);
        if let Some(runtime_requirements) =
          ChunkGraph::get_module_runtime_requirements(self, module.identifier(), chunk.runtime())
        {
          set.insert(*runtime_requirements);
        }
      }
      chunk_requirements.insert(chunk_ukey, set);
    }
    for (&chunk_ukey, mut set) in chunk_requirements {
      plugin_driver
        .compilation_hooks
        .additional_chunk_runtime_requirements
        .call(self, &chunk_ukey, &mut set)
        .await
        .map_err(|e| {
          e.wrap_err("caused by plugins in Compilation.hooks.additionalChunkRuntimeRequirements")
        })?;

      self
        .process_runtime_requirement_hook_mut(&mut set, {
          let plugin_driver = plugin_driver.clone();
          async move |compilation,
                      all_runtime_requirements,
                      runtime_requirements,
                      runtime_requirements_mut| {
            plugin_driver
              .compilation_hooks
              .runtime_requirement_in_chunk
              .call(
                compilation,
                &chunk_ukey,
                all_runtime_requirements,
                runtime_requirements,
                runtime_requirements_mut,
              )
              .await
              .map_err(|e| {
                e.wrap_err("caused by plugins in Compilation.hooks.runtimeRequirementInChunk")
              })?;
            Ok(())
          }
        })
        .await?;

      ChunkGraph::set_chunk_runtime_requirements(self, chunk_ukey, set);
    }
    logger.time_end(start);

    let start = logger.time("runtime requirements.entries");
    for &entry_ukey in &entries {
      let entry = self.chunk_by_ukey.expect_get(&entry_ukey);
      let mut set = RuntimeGlobals::default();
      for chunk_ukey in entry
        .get_all_referenced_chunks(&self.chunk_group_by_ukey)
        .iter()
      {
        let runtime_requirements = ChunkGraph::get_chunk_runtime_requirements(self, chunk_ukey);
        set.insert(*runtime_requirements);
      }

      plugin_driver
        .compilation_hooks
        .additional_tree_runtime_requirements
        .call(self, &entry_ukey, &mut set)
        .await
        .map_err(|e| {
          e.wrap_err("caused by plugins in Compilation.hooks.additionalTreeRuntimeRequirements")
        })?;

      self
        .process_runtime_requirement_hook_mut(&mut set, {
          let plugin_driver = plugin_driver.clone();
          async move |compilation,
                      all_runtime_requirements,
                      runtime_requirements,
                      runtime_requirements_mut| {
            plugin_driver
              .compilation_hooks
              .runtime_requirement_in_tree
              .call(
                compilation,
                &entry_ukey,
                all_runtime_requirements,
                runtime_requirements,
                runtime_requirements_mut,
              )
              .await
              .map_err(|e| {
                e.wrap_err("caused by plugins in Compilation.hooks.runtimeRequirementInTree")
              })?;
            Ok(())
          }
        })
        .await?;

      ChunkGraph::set_tree_runtime_requirements(self, entry_ukey, set);
    }

    // NOTE: webpack runs hooks.runtime_module in compilation.add_runtime_module
    // and overwrite the runtime_module.generate() to get new source in create_chunk_assets
    // this needs full runtime requirements, so run hooks.runtime_module after runtime_requirements_in_tree
    for entry_ukey in &entries {
      let runtime_module_ids: Vec<_> = self
        .chunk_graph
        .get_chunk_runtime_modules_iterable(entry_ukey)
        .copied()
        .collect();
      for runtime_module_id in runtime_module_ids {
        plugin_driver
          .compilation_hooks
          .runtime_module
          .call(self, &runtime_module_id, entry_ukey)
          .await
          .map_err(|e| e.wrap_err("caused by plugins in Compilation.hooks.runtimeModule"))?;
      }
    }

    logger.time_end(start);
    Ok(())
  }

  process_runtime_requirement_hook_macro!(
    process_runtime_requirement_hook,
    &Compilation,
    &Compilation
  );
  process_runtime_requirement_hook_macro!(
    process_runtime_requirement_hook_mut,
    &mut Compilation,
    &mut Compilation
  );

  #[instrument(name = "Compilation:create_hash",target=TRACING_BENCH_TARGET, skip_all)]
  pub async fn create_hash(&mut self, plugin_driver: SharedPluginDriver) -> Result<()> {
    let logger = self.get_logger("rspack.Compilation");

    // Check if there are any chunks that depend on full hash, usually only runtime chunks are
    // possible to depend on full hash, but for library type commonjs/module, it's possible to
    // have non-runtime chunks depend on full hash, the library format plugin is using
    // dependent_full_hash hook to declare it.
    let mut full_hash_chunks = UkeySet::default();
    for chunk_ukey in self.chunk_by_ukey.keys() {
      let chunk_dependent_full_hash = plugin_driver
        .compilation_hooks
        .dependent_full_hash
        .call(self, chunk_ukey)
        .await?
        .unwrap_or_default();
      if chunk_dependent_full_hash {
        full_hash_chunks.insert(*chunk_ukey);
      }
    }
    if !full_hash_chunks.is_empty()
      && let Some(diagnostic) = self.incremental.disable_passes(
        IncrementalPasses::CHUNKS_HASHES,
        "Chunk content that dependent on full hash",
        "it requires calculating the hashes of all the chunks, which is a global effect",
      )
    {
      if let Some(diagnostic) = diagnostic {
        self.push_diagnostic(diagnostic);
      }
      self.chunk_hashes_artifact.clear();
    }

    let create_hash_chunks = if let Some(mutations) = self
      .incremental
      .mutations_read(IncrementalPasses::CHUNKS_HASHES)
      && !self.chunk_hashes_artifact.is_empty()
    {
      let removed_chunks = mutations.iter().filter_map(|mutation| match mutation {
        Mutation::ChunkRemove { chunk } => Some(*chunk),
        _ => None,
      });
      for removed_chunk in removed_chunks {
        self.chunk_hashes_artifact.remove(&removed_chunk);
      }
      self
        .chunk_hashes_artifact
        .retain(|chunk, _| self.chunk_by_ukey.contains(chunk));
      let chunks = mutations.get_affected_chunks_with_chunk_graph(self);
      tracing::debug!(target: incremental::TRACING_TARGET, passes = %IncrementalPasses::CHUNKS_HASHES, %mutations, ?chunks);
      let logger = self.get_logger("rspack.incremental.chunksHashes");
      logger.log(format!(
        "{} chunks are affected, {} in total",
        chunks.len(),
        self.chunk_by_ukey.len(),
      ));
      chunks
    } else {
      self.chunk_by_ukey.keys().copied().collect()
    };

    let mut compilation_hasher = RspackHash::from(&self.options.output);

    fn try_process_chunk_hash_results(
      compilation: &mut Compilation,
      chunk_hash_results: Vec<Result<(ChunkUkey, ChunkHashResult)>>,
    ) -> Result<()> {
      for hash_result in chunk_hash_results {
        let (chunk_ukey, chunk_hash_result) = hash_result?;
        let chunk = compilation.chunk_by_ukey.expect_get(&chunk_ukey);
        let chunk_hashes_changed = chunk.set_hashes(
          &mut compilation.chunk_hashes_artifact,
          chunk_hash_result.hash,
          chunk_hash_result.content_hash,
        );
        if chunk_hashes_changed && let Some(mutations) = compilation.incremental.mutations_write() {
          mutations.add(Mutation::ChunkSetHashes { chunk: chunk_ukey });
        }
      }
      Ok(())
    }

    let unordered_runtime_chunks: UkeySet<ChunkUkey> = self.get_chunk_graph_entries().collect();
    let start = logger.time("hashing: hash chunks");
    let other_chunks: Vec<_> = create_hash_chunks
      .iter()
      .filter(|key| !unordered_runtime_chunks.contains(key))
      .collect();

    // create hash for runtime modules in other chunks
    let other_chunk_runtime_module_hashes = rspack_futures::scope::<_, Result<_>>(|token| {
      other_chunks
        .iter()
        .flat_map(|chunk| self.chunk_graph.get_chunk_runtime_modules_iterable(chunk))
        .for_each(|runtime_module_identifier| {
          let s = unsafe { token.used((&self, runtime_module_identifier)) };
          s.spawn(|(compilation, runtime_module_identifier)| async {
            let runtime_module = &compilation.runtime_modules[runtime_module_identifier];
            let digest = runtime_module.get_runtime_hash(compilation, None).await?;
            Ok((*runtime_module_identifier, digest))
          });
        })
    })
    .await
    .into_iter()
    .map(|res| res.to_rspack_result())
    .collect::<Result<Vec<_>>>()?;

    for res in other_chunk_runtime_module_hashes {
      let (runtime_module_identifier, digest) = res?;
      self
        .runtime_modules_hash
        .insert(runtime_module_identifier, digest);
    }

    // create hash for other chunks
    let other_chunks_hash_results = rspack_futures::scope::<_, Result<_>>(|token| {
      for chunk in other_chunks {
        let s = unsafe { token.used((&self, chunk, &plugin_driver)) };
        s.spawn(|(compilation, chunk, plugin_driver)| async {
          let hash_result = compilation
            .process_chunk_hash(*chunk, plugin_driver)
            .await?;
          Ok((*chunk, hash_result))
        });
      }
    })
    .await
    .into_iter()
    .map(|res| res.to_rspack_result())
    .collect::<Result<Vec<_>>>()?;

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
      circular.sort_unstable_by(|a, b| {
        a.id(&self.chunk_ids_artifact)
          .cmp(&b.id(&self.chunk_ids_artifact))
      });
      runtime_chunks.extend(circular.iter().map(|chunk| chunk.ukey()));
      let circular_names = circular
        .iter()
        .map(|chunk| {
          chunk
            .name()
            .or(chunk.id(&self.chunk_ids_artifact).map(|id| id.as_str()))
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
      let runtime_module_hashes = rspack_futures::scope::<_, Result<_>>(|token| {
        self
          .chunk_graph
          .get_chunk_runtime_modules_iterable(&runtime_chunk_ukey)
          .for_each(|runtime_module_identifier| {
            let s = unsafe { token.used((&self, runtime_module_identifier)) };
            s.spawn(|(compilation, runtime_module_identifier)| async {
              let runtime_module = &compilation.runtime_modules[runtime_module_identifier];
              let digest = runtime_module.get_runtime_hash(compilation, None).await?;
              Ok((*runtime_module_identifier, digest))
            });
          })
      })
      .await
      .into_iter()
      .map(|res| res.to_rspack_result())
      .collect::<Result<Vec<_>>>()?;

      for res in runtime_module_hashes {
        let (mid, digest) = res?;
        self.runtime_modules_hash.insert(mid, digest);
      }

      let chunk_hash_result = self
        .process_chunk_hash(runtime_chunk_ukey, &plugin_driver)
        .await?;
      let chunk = self.chunk_by_ukey.expect_get(&runtime_chunk_ukey);
      let chunk_hashes_changed = chunk.set_hashes(
        &mut self.chunk_hashes_artifact,
        chunk_hash_result.hash,
        chunk_hash_result.content_hash,
      );
      if chunk_hashes_changed && let Some(mutations) = self.incremental.mutations_write() {
        mutations.add(Mutation::ChunkSetHashes {
          chunk: runtime_chunk_ukey,
        });
      }
    }
    logger.time_end(start);

    // create full hash
    self
      .chunk_by_ukey
      .values()
      .sorted_unstable_by_key(|chunk| chunk.ukey())
      .filter_map(|chunk| chunk.hash(&self.chunk_hashes_artifact))
      .for_each(|hash| {
        hash.hash(&mut compilation_hasher);
      });
    self.hot_index.hash(&mut compilation_hasher);
    self.hash = Some(compilation_hasher.digest(&self.options.output.hash_digest));

    // re-create runtime chunk hash that depend on full hash
    let start = logger.time("hashing: process full hash chunks");
    for chunk_ukey in full_hash_chunks {
      for runtime_module_identifier in self
        .chunk_graph
        .get_chunk_runtime_modules_iterable(&chunk_ukey)
      {
        let runtime_module = &self.runtime_modules[runtime_module_identifier];
        if runtime_module.full_hash() || runtime_module.dependent_hash() {
          let digest = runtime_module.get_runtime_hash(self, None).await?;
          self
            .runtime_modules_hash
            .insert(*runtime_module_identifier, digest);
        }
      }
      let chunk = self.chunk_by_ukey.expect_get(&chunk_ukey);
      let new_chunk_hash = {
        let chunk_hash = chunk
          .hash(&self.chunk_hashes_artifact)
          .expect("should have chunk hash");
        let mut hasher = RspackHash::from(&self.options.output);
        chunk_hash.hash(&mut hasher);
        self.hash.hash(&mut hasher);
        hasher.digest(&self.options.output.hash_digest)
      };
      let new_content_hash = {
        let content_hash = chunk
          .content_hash(&self.chunk_hashes_artifact)
          .expect("should have content hash");
        content_hash
          .iter()
          .map(|(source_type, content_hash)| {
            let mut hasher = RspackHash::from(&self.options.output);
            content_hash.hash(&mut hasher);
            self.hash.hash(&mut hasher);
            (
              *source_type,
              hasher.digest(&self.options.output.hash_digest),
            )
          })
          .collect()
      };
      let chunk_hashes_changed = chunk.set_hashes(
        &mut self.chunk_hashes_artifact,
        new_chunk_hash,
        new_content_hash,
      );
      if chunk_hashes_changed && let Some(mutations) = self.incremental.mutations_write() {
        mutations.add(Mutation::ChunkSetHashes { chunk: chunk_ukey });
      }
    }
    logger.time_end(start);
    Ok(())
  }

  #[instrument(skip_all)]
  pub async fn runtime_modules_code_generation(&mut self) -> Result<()> {
    let results = rspack_futures::scope::<_, Result<_>>(|token| {
      self
        .runtime_modules
        .iter()
        .for_each(|(runtime_module_identifier, runtime_module)| {
          let s = unsafe { token.used((&self, runtime_module_identifier, runtime_module)) };
          s.spawn(
            |(compilation, runtime_module_identifier, runtime_module)| async {
              let result = runtime_module
                .code_generation(compilation, None, None)
                .await?;
              let source = result
                .get(&SourceType::Runtime)
                .expect("should have source");
              Ok((*runtime_module_identifier, source.clone()))
            },
          )
        })
    })
    .await
    .into_iter()
    .map(|res| res.to_rspack_result())
    .collect::<Result<Vec<_>>>()?;

    let mut runtime_module_sources = IdentifierMap::<BoxSource>::default();
    for result in results {
      let (runtime_module_identifier, source) = result?;
      runtime_module_sources.insert(runtime_module_identifier, source);
    }

    self.runtime_modules_code_generation_source = runtime_module_sources;
    self
      .code_generated_modules
      .extend(self.runtime_modules.keys().copied());
    Ok(())
  }

  async fn process_chunk_hash(
    &self,
    chunk_ukey: ChunkUkey,
    plugin_driver: &SharedPluginDriver,
  ) -> Result<ChunkHashResult> {
    let mut hasher = RspackHash::from(&self.options.output);
    if let Some(chunk) = self.chunk_by_ukey.get(&chunk_ukey) {
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

    Ok(ChunkHashResult {
      hash: chunk_hash,
      content_hash: content_hashes,
    })
  }

  #[instrument("Compilation:create_module_hashes", skip_all)]
  pub async fn create_module_hashes(&mut self, modules: IdentifierSet) -> Result<()> {
    let mg = self.get_module_graph();
    let results = rspack_futures::scope::<_, Result<_>>(|token| {
      modules
        .into_iter()
        .map(|module| {
          (
            module,
            self
              .chunk_graph
              .get_module_runtimes_iter(module, &self.chunk_by_ukey)
              .cloned()
              .collect::<Vec<_>>(),
          )
        })
        .for_each(|(module_identifier, runtimes)| {
          let s = unsafe { token.used((&*self, &mg)) };
          s.spawn(move |(compilation, mg)| async move {
            let mut hashes = RuntimeSpecMap::new();
            for runtime in runtimes {
              let module = mg
                .module_by_identifier(&module_identifier)
                .expect("should have module");
              let hash = module.get_runtime_hash(compilation, Some(&runtime)).await?;
              hashes.set(runtime, hash);
            }
            Ok((module_identifier, hashes))
          });
        });
    })
    .await
    .into_iter()
    .map(|r| r.to_rspack_result())
    .collect::<Result<Vec<_>>>()?;

    for result in results {
      let (module, hashes) = result?;
      if ChunkGraph::set_module_hashes(self, module, hashes)
        && let Some(mutations) = self.incremental.mutations_write()
      {
        mutations.add(Mutation::ModuleSetHashes { module });
      }
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
    let runtime_module_identifier = ModuleIdentifier::from(format!(
      "{}/{}",
      get_runtime_key(chunk.runtime()),
      module.identifier()
    ));
    module.attach(*chunk_ukey);

    self.chunk_graph.add_module(runtime_module_identifier);
    self.runtime_template.add_templates(module.template());
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

  pub async fn get_path<'b, 'a: 'b>(
    &'a self,
    filename: &Filename,
    mut data: PathData<'b>,
  ) -> Result<String> {
    if data.hash.is_none() {
      data.hash = self.get_hash();
    }
    filename.render(data, None).await
  }

  pub async fn get_path_with_info<'b, 'a: 'b>(
    &'a self,
    filename: &Filename,
    mut data: PathData<'b>,
    info: &mut AssetInfo,
  ) -> Result<String> {
    if data.hash.is_none() {
      data.hash = self.get_hash();
    }
    let path = filename.render(data, Some(info)).await?;
    Ok(path)
  }

  pub async fn get_asset_path(&self, filename: &Filename, data: PathData<'_>) -> Result<String> {
    filename.render(data, None).await
  }

  pub async fn get_asset_path_with_info(
    &self,
    filename: &Filename,
    data: PathData<'_>,
  ) -> Result<(String, AssetInfo)> {
    let mut info = AssetInfo::default();
    let path = filename.render(data, Some(&mut info)).await?;
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

  pub fn set_dependency_template(
    &mut self,
    template_type: DependencyTemplateType,
    template: Arc<dyn DependencyTemplate>,
  ) {
    self.dependency_templates.insert(template_type, template);
  }

  pub fn get_dependency_template(
    &self,
    dep: &dyn DependencyCodeGeneration,
  ) -> Option<Arc<dyn DependencyTemplate>> {
    dep
      .dependency_template()
      .and_then(|template_type| self.dependency_templates.get(&template_type).cloned())
  }

  pub fn built_modules(&self) -> &IdentifierSet {
    &self.make_artifact.built_modules
  }
}

pub type CompilationAssets = HashMap<String, CompilationAsset>;

#[cacheable]
#[derive(Debug, Clone)]
pub struct CompilationAsset {
  #[cacheable(with=AsOption<AsPreset>)]
  pub source: Option<BoxSource>,
  pub info: BindingCell<AssetInfo>,
}

impl From<BoxSource> for CompilationAsset {
  fn from(value: BoxSource) -> Self {
    Self::new(Some(value), Default::default())
  }
}

impl CompilationAsset {
  pub fn new(source: Option<BoxSource>, info: AssetInfo) -> Self {
    Self {
      source,
      info: BindingCell::from(info),
    }
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
    self.info = BindingCell::from(info);
  }
}

#[cacheable]
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
  /// whether this asset is over the size limit
  pub is_over_size_limit: Option<bool>,

  /// Webpack: AssetInfo = KnownAssetInfo & Record<string, any>
  /// This is a hack to store the additional fields in the rust struct.
  /// Related: packages/rspack/src/Compilation.ts
  #[cacheable(with=AsPreset)]
  pub extras: serde_json::Map<String, serde_json::Value>,
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

#[cacheable]
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
  modules: impl Iterator<Item = &ModuleIdentifier>,
) {
  // https://github.com/webpack/webpack/blob/1f99ad6367f2b8a6ef17cce0e058f7a67fb7db18/lib/Compilation.js#L3720
  let mut q = VecDeque::new();
  for item in modules {
    q.push_back((*item, 0));
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
  pub filename: String,
  pub has_filename: bool, /* webpack only asset has filename, js/css/wasm has filename template */
  pub info: AssetInfo,
  pub auxiliary: bool,
}

pub struct ChunkHashResult {
  pub hash: RspackHashDigest,
  pub content_hash: ChunkContentHash,
}
