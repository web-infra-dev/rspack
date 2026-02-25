mod after_process_assets;
mod after_seal;
mod assign_runtime_ids;
pub mod build_chunk_graph;
pub mod build_module_graph;
mod chunk_ids;
mod code_generation;
mod create_chunk_assets;
mod create_hash;
mod create_module_assets;
mod create_module_hashes;
mod finish_make;
mod finish_module_graph;
mod finish_modules;
mod make;
mod module_ids;
mod optimize_chunk_modules;
mod optimize_chunks;
mod optimize_code_generation;
mod optimize_dependencies;
mod optimize_modules;
mod optimize_tree;
pub mod pass;
mod process_assets;
mod run_passes;
mod runtime_requirements;
mod seal;
use std::{
  collections::{VecDeque, hash_map},
  fmt::{self, Debug},
  hash::{BuildHasherDefault, Hash},
  mem,
  sync::{
    Arc,
    atomic::{AtomicBool, AtomicU32, Ordering},
  },
};

use dashmap::DashSet;
use futures::future::BoxFuture;
use indexmap::IndexMap;
use itertools::Itertools;
use rayon::prelude::*;
use rspack_cacheable::{
  cacheable,
  with::{AsOption, AsPreset},
};
use rspack_collections::{
  DatabaseItem, IdentifierDashMap, IdentifierMap, IdentifierSet, UkeyMap, UkeySet,
};
use rspack_error::{Diagnostic, Result, ToStringResultToRspackResultExt};
use rspack_fs::{IntermediateFileSystem, ReadableFileSystem, WritableFileSystem};
use rspack_hash::{RspackHash, RspackHashDigest};
use rspack_hook::define_hook;
use rspack_paths::{ArcPath, ArcPathIndexSet, ArcPathSet};
use rspack_sources::BoxSource;
use rspack_tasks::CompilerContext;
#[cfg(allocative)]
use rspack_util::allocative;
use rspack_util::{itoa, tracing_preset::TRACING_BENCH_TARGET};
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet, FxHasher};
use tracing::instrument;
use ustr::Ustr;

use crate::{
  AsyncModulesArtifact, BindingCell, BoxDependency, BoxModule, BuildChunkGraphArtifact, CacheCount,
  CacheOptions, CgcRuntimeRequirementsArtifact, CgmHashArtifact, CgmRuntimeRequirementsArtifact,
  Chunk, ChunkByUkey, ChunkContentHash, ChunkGraph, ChunkGroupByUkey, ChunkGroupUkey,
  ChunkHashesArtifact, ChunkKind, ChunkNamedIdArtifact, ChunkRenderArtifact,
  ChunkRenderCacheArtifact, ChunkRenderResult, ChunkUkey, CodeGenerateCacheArtifact,
  CodeGenerationJob, CodeGenerationResult, CodeGenerationResults, CompilationLogger,
  CompilationLogging, CompilerOptions, CompilerPlatform, ConcatenationScope,
  DependenciesDiagnosticsArtifact, DependencyId, DependencyTemplate, DependencyTemplateType,
  DependencyType, Entry, EntryData, EntryOptions, EntryRuntime, Entrypoint, ExecuteModuleId,
  ExportsInfoArtifact, ExtendedReferencedExport, Filename, ImportPhase, ImportVarMap,
  ImportedByDeferModulesArtifact, MemoryGCStorage, ModuleFactory, ModuleGraph,
  ModuleGraphCacheArtifact, ModuleIdentifier, ModuleIdsArtifact, ModuleStaticCache, PathData,
  ProcessRuntimeRequirementsCacheArtifact, ResolverFactory, RuntimeGlobals, RuntimeKeyMap,
  RuntimeMode, RuntimeModule, RuntimeSpec, RuntimeSpecMap, RuntimeTemplate, SharedPluginDriver,
  SideEffectsOptimizeArtifact, SourceType, Stats, StealCell, ValueCacheVersions,
  compilation::build_module_graph::{
    BuildModuleGraphArtifact, ModuleExecutor, UpdateParam, update_module_graph,
  },
  compiler::{CompilationRecords, CompilerId},
  get_runtime_key,
  incremental::{self, Incremental, IncrementalPasses, Mutation},
  is_source_equal, to_identifier,
};

define_hook!(CompilationAddEntry: Series(compilation: &mut Compilation, entry_name: Option<&str>));
define_hook!(CompilationBuildModule: Series(compiler_id: CompilerId, compilation_id: CompilationId, module: &mut BoxModule),tracing=false);
define_hook!(CompilationRevokedModules: Series(compilation: &Compilation, revoked_modules: &IdentifierSet));
define_hook!(CompilationStillValidModule: Series(compiler_id: CompilerId, compilation_id: CompilationId, module: &mut BoxModule));
define_hook!(CompilationSucceedModule: Series(compiler_id: CompilerId, compilation_id: CompilationId, module: &mut BoxModule),tracing=false);
define_hook!(CompilationExecuteModule:
  Series(module: &ModuleIdentifier, runtime_modules: &IdentifierSet, code_generation_results: &BindingCell<CodeGenerationResults>, execute_module_id: &ExecuteModuleId));
define_hook!(CompilationFinishModules: Series(compilation: &Compilation, async_modules_artifact: &mut AsyncModulesArtifact, exports_info_artifact: &mut ExportsInfoArtifact));
define_hook!(CompilationSeal: Series(compilation: &Compilation, diagnostics: &mut Vec<Diagnostic>));
define_hook!(CompilationDependencyReferencedExports: Sync(
  compilation: &Compilation,
  dependency: &DependencyId,
  referenced_exports: &Option<Vec<ExtendedReferencedExport>>,
  runtime: Option<&RuntimeSpec>,
  module_graph: Option<&ModuleGraph>
));
define_hook!(CompilationConcatenationScope: SeriesBail(compilation: &Compilation, curr_module: ModuleIdentifier) -> ConcatenationScope);
define_hook!(CompilationOptimizeDependencies: SeriesBail(compilation: &Compilation, side_effects_optimize_artifact: &mut SideEffectsOptimizeArtifact,  build_module_graph_artifact: &mut BuildModuleGraphArtifact, exports_info_artifact: &mut ExportsInfoArtifact,
 diagnostics: &mut Vec<Diagnostic>) -> bool);
define_hook!(CompilationOptimizeModules: SeriesBail(compilation: &Compilation, diagnostics: &mut Vec<Diagnostic>) -> bool);
define_hook!(CompilationAfterOptimizeModules: Series(compilation: &Compilation));
define_hook!(CompilationOptimizeChunks: SeriesBail(compilation: &mut Compilation) -> bool);
define_hook!(CompilationOptimizeTree: Series(compilation: &Compilation));
define_hook!(CompilationOptimizeChunkModules: SeriesBail(compilation: &mut Compilation) -> bool);
define_hook!(CompilationBeforeModuleIds: Series(compilation: &Compilation, modules: &IdentifierSet, module_ids: &mut ModuleIdsArtifact));
define_hook!(CompilationModuleIds: Series(compilation: &Compilation, module_ids: &mut ModuleIdsArtifact, diagnostics: &mut Vec<Diagnostic>));
define_hook!(CompilationChunkIds: Series(compilation: &Compilation, chunk_by_ukey: &mut ChunkByUkey, named_chunk_ids_artifact: &mut ChunkNamedIdArtifact, diagnostics: &mut Vec<Diagnostic>));
define_hook!(CompilationRuntimeModule: Series(compilation: &Compilation, module: &ModuleIdentifier, chunk: &ChunkUkey, runtime_modules: &mut IdentifierMap<Box<dyn RuntimeModule>>));
define_hook!(CompilationAdditionalModuleRuntimeRequirements: Series(compilation: &Compilation, module_identifier: &ModuleIdentifier, runtime_requirements: &mut RuntimeGlobals),tracing=false);
define_hook!(CompilationRuntimeRequirementInModule: SeriesBail(compilation: &Compilation, module_identifier: &ModuleIdentifier, all_runtime_requirements: &RuntimeGlobals, runtime_requirements: &RuntimeGlobals, runtime_requirements_mut: &mut RuntimeGlobals),tracing=false);
define_hook!(CompilationAdditionalChunkRuntimeRequirements: Series(compilation: &Compilation, chunk_ukey: &ChunkUkey, runtime_requirements: &mut RuntimeGlobals, runtime_modules: &mut Vec<Box<dyn RuntimeModule>>));
define_hook!(CompilationRuntimeRequirementInChunk: SeriesBail(compilation: &Compilation, chunk_ukey: &ChunkUkey, all_runtime_requirements: &RuntimeGlobals, runtime_requirements: &RuntimeGlobals, runtime_requirements_mut: &mut RuntimeGlobals, runtime_modules_to_add: &mut Vec<Box<dyn RuntimeModule>>));
define_hook!(CompilationAdditionalTreeRuntimeRequirements: Series(compilation: &Compilation, chunk_ukey: &ChunkUkey, runtime_requirements: &mut RuntimeGlobals, runtime_modules: &mut Vec<Box<dyn RuntimeModule>>));
define_hook!(CompilationRuntimeRequirementInTree: SeriesBail(compilation: &Compilation, chunk_ukey: &ChunkUkey, all_runtime_requirements: &RuntimeGlobals, runtime_requirements: &RuntimeGlobals, runtime_requirements_mut: &mut RuntimeGlobals, runtime_modules_to_add: &mut Vec<(ChunkUkey, Box<dyn RuntimeModule>)>));
define_hook!(CompilationOptimizeCodeGeneration: Series(compilation: &Compilation, build_module_graph_artifact: &mut BuildModuleGraphArtifact, exports_info_artifact: &mut ExportsInfoArtifact, diagnostics: &mut Vec<Diagnostic>));
define_hook!(CompilationAfterCodeGeneration: Series(compilation: &Compilation, diagnostics: &mut Vec<Diagnostic>));
define_hook!(CompilationChunkHash: Series(compilation: &Compilation, chunk_ukey: &ChunkUkey, hasher: &mut RspackHash),tracing=false);
define_hook!(CompilationContentHash: Series(compilation: &Compilation, chunk_ukey: &ChunkUkey, hashes: &mut HashMap<SourceType, RspackHash>));
define_hook!(CompilationDependentFullHash: SeriesBail(compilation: &Compilation, chunk_ukey: &ChunkUkey) -> bool);
define_hook!(CompilationRenderManifest: Series(compilation: &Compilation, chunk_ukey: &ChunkUkey, manifest: &mut Vec<RenderManifestEntry>, diagnostics: &mut Vec<Diagnostic>),tracing=false);
define_hook!(CompilationChunkAsset: Series(compilation: &Compilation, chunk_ukey: &ChunkUkey, filename: &str));
define_hook!(CompilationProcessAssets: Series(compilation: &mut Compilation));
define_hook!(CompilationAfterProcessAssets: Series(compilation: &Compilation, diagnostics: &mut Vec<Diagnostic>));
define_hook!(CompilationAfterSeal: Series(compilation: &Compilation),tracing=true);

#[derive(Debug, Default)]
pub struct CompilationHooks {
  pub add_entry: CompilationAddEntryHook,
  pub build_module: CompilationBuildModuleHook,
  pub revoked_modules: CompilationRevokedModulesHook,
  pub concatenation_scope: CompilationConcatenationScopeHook,
  pub still_valid_module: CompilationStillValidModuleHook,
  pub succeed_module: CompilationSucceedModuleHook,
  pub execute_module: CompilationExecuteModuleHook,
  pub finish_modules: CompilationFinishModulesHook,
  pub dependency_referenced_exports: CompilationDependencyReferencedExportsHook,
  pub seal: CompilationSealHook,
  pub optimize_dependencies: CompilationOptimizeDependenciesHook,
  pub optimize_modules: CompilationOptimizeModulesHook,
  pub after_optimize_modules: CompilationAfterOptimizeModulesHook,
  pub optimize_chunks: CompilationOptimizeChunksHook,
  pub optimize_tree: CompilationOptimizeTreeHook,
  pub optimize_chunk_modules: CompilationOptimizeChunkModulesHook,
  pub before_module_ids: CompilationBeforeModuleIdsHook,
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

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, Ord, PartialOrd)]
#[cfg_attr(allocative, derive(allocative::Allocative))]
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

static COMPILATION_ID: AtomicU32 = AtomicU32::new(0);

/// Use macro to prevent cargo shear from failing and reporting errors
/// due to the inability to parse the async closure syntax
/// https://github.com/Boshen/cargo-shear/issues/143
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
  pub platform: Arc<CompilerPlatform>,
  pub entries: Entry,
  pub global_entry: EntryData,
  pub dependency_factories: HashMap<DependencyType, Arc<dyn ModuleFactory>>,
  pub dependency_templates: HashMap<DependencyTemplateType, Arc<dyn DependencyTemplate>>,
  pub runtime_modules: IdentifierMap<Box<dyn RuntimeModule>>,
  pub runtime_modules_hash: IdentifierMap<RspackHashDigest>,
  pub runtime_modules_code_generation_source: IdentifierMap<BoxSource>,
  assets: CompilationAssets,
  assets_related_in: HashMap<String, HashSet<String>>,
  pub emitted_assets: DashSet<String, BuildHasherDefault<FxHasher>>,
  diagnostics: Vec<Diagnostic>,
  logging: CompilationLogging,
  pub plugin_driver: SharedPluginDriver,
  pub buildtime_plugin_driver: SharedPluginDriver,
  pub resolver_factory: Arc<ResolverFactory>,
  pub loader_resolver_factory: Arc<ResolverFactory>,
  pub runtime_template: RuntimeTemplate,

  // artifact for infer_async_modules_plugin
  pub async_modules_artifact: StealCell<AsyncModulesArtifact>,
  // artifact for collect_dependencies_diagnostics
  pub dependencies_diagnostics_artifact: StealCell<DependenciesDiagnosticsArtifact>,
  // artifact for exports info
  pub exports_info_artifact: StealCell<ExportsInfoArtifact>,
  // artifact for side_effects_flag_plugin
  pub side_effects_optimize_artifact: StealCell<SideEffectsOptimizeArtifact>,
  // artifact for module_ids
  pub module_ids_artifact: StealCell<ModuleIdsArtifact>,
  // artifact for named_chunk_ids
  pub named_chunk_ids_artifact: StealCell<ChunkNamedIdArtifact>,
  // artifact for code_generation
  pub code_generation_results: BindingCell<CodeGenerationResults>,
  // artifact for create_module_hashes
  pub cgm_hash_artifact: StealCell<CgmHashArtifact>,
  // artifact for process_modules_runtime_requirements
  pub cgm_runtime_requirements_artifact: StealCell<CgmRuntimeRequirementsArtifact>,
  // artifact for process_chunks_runtime_requirements
  pub cgc_runtime_requirements_artifact: StealCell<CgcRuntimeRequirementsArtifact>,
  // artifact for create_hash
  pub chunk_hashes_artifact: StealCell<ChunkHashesArtifact>,
  // artifact for create_chunk_assets
  pub chunk_render_artifact: StealCell<ChunkRenderArtifact>,
  // artifact for caching get_mode
  pub module_graph_cache_artifact: StealCell<ModuleGraphCacheArtifact>,
  // transient cache for module static info
  pub module_static_cache: ModuleStaticCache,
  // artifact for chunk render cache
  pub chunk_render_cache_artifact: StealCell<ChunkRenderCacheArtifact>,
  // artifact for code generate cache
  pub code_generate_cache_artifact: StealCell<CodeGenerateCacheArtifact>,
  // artifact for process runtime requirements cache
  pub process_runtime_requirements_cache_artifact:
    StealCell<ProcessRuntimeRequirementsCacheArtifact>,
  pub imported_by_defer_modules_artifact: StealCell<ImportedByDeferModulesArtifact>,

  pub code_generated_modules: IdentifierSet,
  pub build_time_executed_modules: IdentifierSet,
  pub build_chunk_graph_artifact: BuildChunkGraphArtifact,
  pub incremental: Incremental,

  pub hash: Option<RspackHashDigest>,

  pub file_dependencies: ArcPathIndexSet,
  pub context_dependencies: ArcPathIndexSet,
  pub missing_dependencies: ArcPathIndexSet,
  pub build_dependencies: ArcPathIndexSet,

  pub value_cache_versions: ValueCacheVersions,

  import_var_map: IdentifierDashMap<RuntimeKeyMap<ImportVarMap>>,

  // TODO move to MakeArtifact
  pub module_executor: Option<ModuleExecutor>,
  in_finish_make: AtomicBool,

  pub modified_files: ArcPathSet,
  pub removed_files: ArcPathSet,
  pub build_module_graph_artifact: StealCell<BuildModuleGraphArtifact>,
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
  pub const PROCESS_ASSETS_STAGE_AFTER_OPTIMIZE_HASH: i32 = 2600;
  pub const PROCESS_ASSETS_STAGE_OPTIMIZE_TRANSFER: i32 = 3000;
  pub const PROCESS_ASSETS_STAGE_ANALYSE: i32 = 4000;
  pub const PROCESS_ASSETS_STAGE_REPORT: i32 = 5000;

  #[allow(clippy::too_many_arguments)]
  pub fn new(
    compiler_id: CompilerId,
    options: Arc<CompilerOptions>,
    platform: Arc<CompilerPlatform>,
    plugin_driver: SharedPluginDriver,
    buildtime_plugin_driver: SharedPluginDriver,
    resolver_factory: Arc<ResolverFactory>,
    loader_resolver_factory: Arc<ResolverFactory>,
    records: Option<CompilationRecords>,
    incremental: Incremental,
    module_executor: Option<ModuleExecutor>,
    modified_files: ArcPathSet,
    removed_files: ArcPathSet,
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
      runtime_template: RuntimeTemplate::new(options.clone()),
      records,
      options: options.clone(),
      platform,
      dependency_factories: Default::default(),
      dependency_templates: Default::default(),
      runtime_modules: Default::default(),
      runtime_modules_hash: Default::default(),
      runtime_modules_code_generation_source: Default::default(),
      entries: Default::default(),
      global_entry: Default::default(),
      assets: Default::default(),
      assets_related_in: Default::default(),
      emitted_assets: Default::default(),
      diagnostics: Default::default(),
      logging: Default::default(),
      plugin_driver,
      buildtime_plugin_driver,
      resolver_factory,
      loader_resolver_factory,

      async_modules_artifact: StealCell::new(AsyncModulesArtifact::default()),
      imported_by_defer_modules_artifact: StealCell::new(Default::default()),
      dependencies_diagnostics_artifact: StealCell::new(DependenciesDiagnosticsArtifact::default()),
      exports_info_artifact: StealCell::new(ExportsInfoArtifact::default()),
      side_effects_optimize_artifact: StealCell::new(Default::default()),
      module_ids_artifact: StealCell::new(Default::default()),
      named_chunk_ids_artifact: StealCell::new(Default::default()),
      code_generation_results: Default::default(),
      cgm_hash_artifact: StealCell::new(Default::default()),
      cgm_runtime_requirements_artifact: StealCell::new(Default::default()),
      cgc_runtime_requirements_artifact: StealCell::new(Default::default()),
      chunk_hashes_artifact: StealCell::new(Default::default()),
      chunk_render_artifact: StealCell::new(Default::default()),
      module_graph_cache_artifact: StealCell::new(Default::default()),
      module_static_cache: Default::default(),
      code_generated_modules: Default::default(),
      chunk_render_cache_artifact: StealCell::new(ChunkRenderCacheArtifact::new(
        MemoryGCStorage::new(match &options.cache {
          CacheOptions::Disabled => 0, // FIXME: this should be removed in future
          CacheOptions::Memory { max_generations } => *max_generations,
          CacheOptions::Persistent(_) => 1,
        }),
      )),
      code_generate_cache_artifact: StealCell::new(CodeGenerateCacheArtifact::new(&options)),
      process_runtime_requirements_cache_artifact: StealCell::new(
        ProcessRuntimeRequirementsCacheArtifact::new(&options),
      ),
      build_time_executed_modules: Default::default(),
      incremental,
      build_chunk_graph_artifact: Default::default(),

      hash: None,

      file_dependencies: Default::default(),
      context_dependencies: Default::default(),
      missing_dependencies: Default::default(),
      build_dependencies: Default::default(),

      value_cache_versions: ValueCacheVersions::default(),

      import_var_map: IdentifierDashMap::default(),

      module_executor,
      in_finish_make: AtomicBool::new(false),

      build_module_graph_artifact: StealCell::new(BuildModuleGraphArtifact::new()),
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

  pub fn get_module_graph(&self) -> &ModuleGraph {
    self.build_module_graph_artifact.get_module_graph()
  }

  // it will return None during make phase since mg is incomplete
  pub fn module_by_identifier(&self, identifier: &ModuleIdentifier) -> Option<&BoxModule> {
    if self.build_module_graph_artifact.is_stolen() {
      return None;
    }
    if let Some(module) = self.get_module_graph().module_by_identifier(identifier) {
      return Some(module);
    };

    self
      .build_module_graph_artifact
      .get_module_graph()
      .module_by_identifier(identifier)
  }
  pub fn get_module_graph_mut(&mut self) -> &mut ModuleGraph {
    self.build_module_graph_artifact.get_module_graph_mut()
  }

  pub fn file_dependencies(
    &self,
  ) -> (
    impl Iterator<Item = &ArcPath>,
    impl Iterator<Item = &ArcPath>,
    impl Iterator<Item = &ArcPath>,
    impl Iterator<Item = &ArcPath>,
  ) {
    let all_files = self
      .build_module_graph_artifact
      .file_dependencies
      .files()
      .chain(&self.file_dependencies);
    let added_files = self
      .build_module_graph_artifact
      .file_dependencies
      .added_files()
      .chain(&self.file_dependencies);
    let updated_files = self
      .build_module_graph_artifact
      .file_dependencies
      .updated_files();
    let removed_files = self
      .build_module_graph_artifact
      .file_dependencies
      .removed_files();
    (all_files, added_files, updated_files, removed_files)
  }

  pub fn context_dependencies(
    &self,
  ) -> (
    impl Iterator<Item = &ArcPath>,
    impl Iterator<Item = &ArcPath>,
    impl Iterator<Item = &ArcPath>,
    impl Iterator<Item = &ArcPath>,
  ) {
    let all_files = self
      .build_module_graph_artifact
      .context_dependencies
      .files()
      .chain(&self.context_dependencies);
    let added_files = self
      .build_module_graph_artifact
      .context_dependencies
      .added_files()
      .chain(&self.file_dependencies);
    let updated_files = self
      .build_module_graph_artifact
      .context_dependencies
      .updated_files();
    let removed_files = self
      .build_module_graph_artifact
      .context_dependencies
      .removed_files();
    (all_files, added_files, updated_files, removed_files)
  }

  pub fn missing_dependencies(
    &self,
  ) -> (
    impl Iterator<Item = &ArcPath>,
    impl Iterator<Item = &ArcPath>,
    impl Iterator<Item = &ArcPath>,
    impl Iterator<Item = &ArcPath>,
  ) {
    let all_files = self
      .build_module_graph_artifact
      .missing_dependencies
      .files()
      .chain(&self.missing_dependencies);
    let added_files = self
      .build_module_graph_artifact
      .missing_dependencies
      .added_files()
      .chain(&self.file_dependencies);
    let updated_files = self
      .build_module_graph_artifact
      .missing_dependencies
      .updated_files();
    let removed_files = self
      .build_module_graph_artifact
      .missing_dependencies
      .removed_files();
    (all_files, added_files, updated_files, removed_files)
  }

  pub fn build_dependencies(
    &self,
  ) -> (
    impl Iterator<Item = &ArcPath>,
    impl Iterator<Item = &ArcPath>,
    impl Iterator<Item = &ArcPath>,
    impl Iterator<Item = &ArcPath>,
  ) {
    let all_files = self
      .build_module_graph_artifact
      .build_dependencies
      .files()
      .chain(&self.build_dependencies);
    let added_files = self
      .build_module_graph_artifact
      .build_dependencies
      .added_files()
      .chain(&self.file_dependencies);
    let updated_files = self
      .build_module_graph_artifact
      .build_dependencies
      .updated_files();
    let removed_files = self
      .build_module_graph_artifact
      .build_dependencies
      .removed_files();
    (all_files, added_files, updated_files, removed_files)
  }

  // TODO move out from compilation
  pub fn get_import_var(
    &self,
    module: ModuleIdentifier,
    target_module: Option<&BoxModule>,
    user_request: &str,
    phase: ImportPhase,
    runtime: Option<&RuntimeSpec>,
  ) -> String {
    let mut runtime_map = self.import_var_map.entry(module).or_default();
    let import_var_map_of_module = runtime_map
      .entry(
        runtime
          .map(|r| get_runtime_key(r).clone())
          .unwrap_or_default(),
      )
      .or_default();
    let len = import_var_map_of_module.len();
    let is_deferred = phase.is_defer()
      && !target_module
        .map(|m| m.build_meta().has_top_level_await)
        .unwrap_or_default();

    match import_var_map_of_module.entry((target_module.map(|m| m.identifier()), is_deferred)) {
      hash_map::Entry::Occupied(occ) => occ.get().clone(),
      hash_map::Entry::Vacant(vac) => {
        let mut b = itoa::Buffer::new();
        let import_var = format!(
          "{}__rspack_{}import_{}",
          to_identifier(user_request),
          match phase {
            ImportPhase::Evaluation => "",
            ImportPhase::Source => "",
            ImportPhase::Defer => "DEFERRED_",
          },
          b.format(len)
        );
        vac.insert(import_var.clone());
        import_var
      }
    }
  }

  pub async fn add_entry(&mut self, entry: BoxDependency, options: EntryOptions) -> Result<()> {
    let entry_id = *entry.id();
    let entry_name: Option<String> = options.name.clone();
    self
      .build_module_graph_artifact
      .get_module_graph_mut()
      .add_dependency(entry);
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

    let make_artifact = self.build_module_graph_artifact.steal();
    let exports_info_artifact = self.exports_info_artifact.steal();
    let (make_artifact, exports_info_artifact) = update_module_graph(
      self,
      make_artifact,
      exports_info_artifact,
      vec![UpdateParam::BuildEntry(
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
    self.build_module_graph_artifact = make_artifact.into();
    self.exports_info_artifact = exports_info_artifact.into();

    Ok(())
  }

  pub async fn add_include(&mut self, args: Vec<(BoxDependency, EntryOptions)>) -> Result<()> {
    if !self.in_finish_make.load(Ordering::Acquire) {
      return Err(rspack_error::Error::error(
        "You can only call `add_include` during the finish make stage".into(),
      ));
    }

    for (entry, options) in args {
      let entry_id = *entry.id();
      self
        .build_module_graph_artifact
        .get_module_graph_mut()
        .add_dependency(entry);
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
    let make_artifact = self.build_module_graph_artifact.steal();
    let exports_info_artifact = self.exports_info_artifact.steal();
    let (make_artifact, exports_info_artifact) = update_module_graph(
      self,
      make_artifact,
      exports_info_artifact,
      vec![UpdateParam::BuildEntry(
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
    self.build_module_graph_artifact = make_artifact.into();
    self.exports_info_artifact = exports_info_artifact.into();

    Ok(())
  }

  fn set_asset_info(
    &mut self,
    name: &str,
    new_info: Option<&AssetInfo>,
    old_info: Option<&AssetInfo>,
  ) {
    if let Some(old_info) = old_info
      && let Some(source_map) = &old_info.related.source_map
      && let Some(entry) = self.assets_related_in.get_mut(source_map)
    {
      entry.remove(name);
    }
    if let Some(new_info) = new_info
      && let Some(source_map) = new_info.related.source_map.clone()
    {
      let entry = self.assets_related_in.entry(source_map).or_default();
      entry.insert(name.to_string());
    }
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

    let (old_info, new_source, new_info) = match assets.remove(filename) {
      Some(CompilationAsset {
        source: Some(source),
        info: old_info,
      }) => {
        let (new_source, new_info) = updater(source, old_info.clone())?;
        (old_info, new_source, new_info)
      }
      _ => {
        return Err(rspack_error::error!(
          "Called Compilation.updateAsset for not existing filename {}",
          filename
        ));
      }
    };
    self.set_asset_info(filename, Some(&new_info), Some(&old_info));
    self.assets.insert(
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
          rspack_error::error!(
            "Conflict: Multiple assets emit different content to the same filename {}{}",
            filename,
            // TODO: source file name
            ""
          )
          .into(),
        );
        self.set_asset_info(&filename, Some(asset.get_info()), None);
        self.assets.insert(filename, asset);
        return;
      }
      self.set_asset_info(&filename, Some(asset.get_info()), Some(original.get_info()));
      original.info = asset.info;
      self.assets.insert(filename, original);
    } else {
      self.set_asset_info(&filename, Some(asset.get_info()), None);
      self.assets.insert(filename, asset);
    }
  }

  pub fn delete_asset(&mut self, filename: &str) {
    if let Some(asset) = self.assets.remove(filename) {
      self.set_asset_info(filename, None, Some(asset.get_info()));

      if let Some(source_map) = &asset.info.related.source_map {
        self.delete_asset(source_map);
      }
      self
        .build_chunk_graph_artifact
        .chunk_by_ukey
        .iter_mut()
        .for_each(|(_, chunk)| {
          chunk.remove_file(filename);
          chunk.remove_auxiliary_file(filename);
        });
    }
  }

  pub fn rename_asset(&mut self, filename: &str, new_name: String) {
    if let Some(asset) = self.assets.remove(filename) {
      // Update related in all other assets
      if let Some(related_in_info) = self.assets_related_in.get(filename) {
        for name in related_in_info {
          if let Some(asset) = self.assets.get_mut(name) {
            asset.get_info_mut().related.source_map = Some(new_name.clone());
          }
        }
      }
      self.set_asset_info(filename, None, Some(asset.get_info()));
      self.set_asset_info(&new_name, Some(asset.get_info()), None);

      self.assets.insert(new_name.clone(), asset);

      self
        .build_chunk_graph_artifact
        .chunk_by_ukey
        .iter_mut()
        .for_each(|(_, chunk)| {
          if chunk.remove_file(filename) {
            chunk.add_file(new_name.clone());
          }

          if chunk.remove_auxiliary_file(filename) {
            chunk.add_auxiliary_file(new_name.clone());
          }
        });
    }
  }

  // Batch version of rename_asset with parallel optimization.
  // Multiple calls to rename_asset would cause performance degradation due to
  // repeated full traversals of chunk_by_ukey. This method uses parallel iteration
  // over chunk_by_ukey to reduce traversal frequency and improve performance.
  pub fn par_rename_assets(&mut self, renames: Vec<(String, String)>) {
    self
      .build_chunk_graph_artifact
      .chunk_by_ukey
      .values_mut()
      .par_bridge()
      .for_each(|chunk| {
        for (old_name, new_name) in renames.iter() {
          if chunk.remove_file(old_name) {
            chunk.add_file(new_name.clone());
          }

          if chunk.remove_auxiliary_file(old_name) {
            chunk.add_auxiliary_file(new_name.clone());
          }
        }
      });

    for (old_name, new_name) in renames {
      if let Some(asset) = self.assets.remove(&old_name) {
        // Update related in all other assets
        if let Some(related_in_info) = self.assets_related_in.get(&old_name) {
          for related_in_name in related_in_info {
            if let Some(asset) = self.assets.get_mut(related_in_name) {
              asset.get_info_mut().related.source_map = Some(new_name.clone());
            }
          }
        }
        self.set_asset_info(&old_name, None, Some(asset.get_info()));
        self.set_asset_info(&new_name, Some(asset.get_info()), None);

        self.assets.insert(new_name, asset);
      }
    }
  }

  pub fn assets(&self) -> &CompilationAssets {
    &self.assets
  }

  pub fn assets_mut(&mut self) -> &mut CompilationAssets {
    &mut self.assets
  }

  pub fn entrypoints(&self) -> &IndexMap<String, ChunkGroupUkey> {
    &self.build_chunk_graph_artifact.entrypoints
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
    self.diagnostics.iter().filter(|d| d.is_error())
  }

  /// Get sorted errors based on the factors as follows in order:
  /// - module identifier
  /// - error offset
  ///   Rspack assumes for each offset, there is only one error.
  ///   However, when it comes to the case that there are multiple errors with the same offset,
  ///   the order of these errors will not be guaranteed.
  pub fn get_errors_sorted(&self) -> impl Iterator<Item = &Diagnostic> {
    let get_offset = |d: &Diagnostic| {
      d.labels
        .as_ref()
        .and_then(|l| l.first())
        .map(|l| l.offset)
        .unwrap_or_default()
    };
    self
      .get_errors()
      .sorted_by(|a, b| match a.module_identifier.cmp(&b.module_identifier) {
        std::cmp::Ordering::Equal => get_offset(a).cmp(&get_offset(b)),
        other => other,
      })
  }

  pub fn get_warnings(&self) -> impl Iterator<Item = &Diagnostic> {
    self.diagnostics.iter().filter(|d| d.is_warn())
  }

  /// Get sorted warnings based on the factors as follows in order:
  /// - module identifier
  /// - error offset
  ///   Rspack assumes for each offset, there is only one error.
  ///   However, when it comes to the case that there are multiple errors with the same offset,
  ///   the order of these errors will not be guaranteed.
  pub fn get_warnings_sorted(&self) -> impl Iterator<Item = &Diagnostic> {
    let get_offset = |d: &Diagnostic| {
      d.labels
        .as_ref()
        .and_then(|l| l.first())
        .map(|l| l.offset)
        .unwrap_or_default()
    };
    self
      .get_warnings()
      .sorted_by(|a, b| match a.module_identifier.cmp(&b.module_identifier) {
        std::cmp::Ordering::Equal => get_offset(a).cmp(&get_offset(b)),
        other => other,
      })
  }

  pub fn get_logging(&self) -> &CompilationLogging {
    &self.logging
  }

  pub fn get_stats(&self) -> Stats<'_> {
    self.get_stats_with_exports_info_artifact(&self.exports_info_artifact)
  }

  pub fn get_stats_with_exports_info_artifact<'a>(
    &'a self,
    exports_info_artifact: &'a ExportsInfoArtifact,
  ) -> Stats<'a> {
    Stats::new(self, exports_info_artifact)
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

  pub async fn rebuild_module<T>(
    &mut self,
    module_identifiers: IdentifierSet,
    exports_info_artifact: &mut ExportsInfoArtifact,
    f: impl Fn(Vec<&BoxModule>) -> T,
  ) -> Result<T> {
    let artifact = self.build_module_graph_artifact.steal();

    // https://github.com/webpack/webpack/blob/19ca74127f7668aaf60d59f4af8fcaee7924541a/lib/Compilation.js#L2462C21-L2462C25
    self.module_graph_cache_artifact.unfreeze();

    let (artifact, updated_exports_info_artifact) = update_module_graph(
      self,
      artifact,
      std::mem::take(exports_info_artifact),
      vec![UpdateParam::ForceBuildModules(module_identifiers.clone())],
    )
    .await?;
    *exports_info_artifact = updated_exports_info_artifact;
    self.build_module_graph_artifact = artifact.into();

    let module_graph = self.get_module_graph();
    Ok(f(module_identifiers
      .into_iter()
      .filter_map(|id| module_graph.module_by_identifier(&id))
      .collect::<Vec<_>>()))
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
    let ukey = self
      .build_chunk_graph_artifact
      .entrypoints
      .get(name)
      .expect("entrypoint not found");
    self
      .build_chunk_graph_artifact
      .chunk_group_by_ukey
      .expect_get(ukey)
  }

  pub fn entrypoint_by_name_mut(&mut self, name: &str) -> &mut Entrypoint {
    let ukey = self
      .build_chunk_graph_artifact
      .entrypoints
      .get(name)
      .expect("entrypoint not found");
    self
      .build_chunk_graph_artifact
      .chunk_group_by_ukey
      .expect_get_mut(ukey)
  }

  pub fn get_chunk_graph_entries(&self) -> impl Iterator<Item = ChunkUkey> + use<'_> {
    let entries = self
      .build_chunk_graph_artifact
      .entrypoints
      .values()
      .map(|entrypoint_ukey| {
        let entrypoint = self
          .build_chunk_graph_artifact
          .chunk_group_by_ukey
          .expect_get(entrypoint_ukey);
        entrypoint.get_runtime_chunk(&self.build_chunk_graph_artifact.chunk_group_by_ukey)
      });
    let async_entries = self
      .build_chunk_graph_artifact
      .async_entrypoints
      .iter()
      .map(|entrypoint_ukey| {
        let entrypoint = self
          .build_chunk_graph_artifact
          .chunk_group_by_ukey
          .expect_get(entrypoint_ukey);
        entrypoint.get_runtime_chunk(&self.build_chunk_graph_artifact.chunk_group_by_ukey)
      });
    entries.chain(async_entries)
  }
  pub fn add_runtime_module(
    &mut self,
    chunk_ukey: &ChunkUkey,
    mut module: Box<dyn RuntimeModule>,
  ) -> Result<()> {
    // add chunk runtime to prefix module identifier to avoid multiple entry runtime modules conflict
    let chunk = self
      .build_chunk_graph_artifact
      .chunk_by_ukey
      .expect_get(chunk_ukey);
    let runtime_module_identifier = ModuleIdentifier::from(format!(
      "{}/{}",
      get_runtime_key(chunk.runtime()),
      module.identifier()
    ));
    module.attach(*chunk_ukey);

    self
      .build_chunk_graph_artifact
      .chunk_graph
      .add_module(runtime_module_identifier);
    self.runtime_template.add_templates(module.template());
    self
      .build_chunk_graph_artifact
      .chunk_graph
      .connect_chunk_and_module(*chunk_ukey, runtime_module_identifier);
    self
      .build_chunk_graph_artifact
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
    template_type: DependencyTemplateType,
  ) -> Option<Arc<dyn DependencyTemplate>> {
    self.dependency_templates.get(&template_type).cloned()
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
  /// the plugin that created the asset
  pub asset_type: ManifestAssetType,

  /// Webpack: AssetInfo = KnownAssetInfo & Record<string, any>
  /// This is a hack to store the additional fields in the rust struct.
  /// Related: packages/rspack/src/Compilation.ts
  #[cacheable(with=AsPreset)]
  pub extras: serde_json::Map<String, serde_json::Value>,
}

impl AssetInfo {
  pub fn with_development(mut self, v: Option<bool>) -> Self {
    self.development = v;
    self
  }

  pub fn with_hot_module_replacement(mut self, v: Option<bool>) -> Self {
    self.hot_module_replacement = v;
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

  pub fn with_asset_type(mut self, v: ManifestAssetType) -> Self {
    self.asset_type = v;
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

    self.source_filename = another
      .source_filename
      .or_else(|| self.source_filename.take());
    self.version = another.version;
    self.related.merge_another(another.related);

    // merge vec fields
    self.chunk_hash.extend(another.chunk_hash);
    self.content_hash.extend(another.content_hash);
    self.extras.extend(another.extras);
    // self.full_hash.extend(another.full_hash.iter().cloned());
    // self.module_hash.extend(another.module_hash.iter().cloned());

    // old first fields or truthy first fields
    self.javascript_module = another
      .javascript_module
      .or_else(|| self.javascript_module.take());
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
pub fn assign_depths<'a>(
  assign_map: &mut IdentifierMap<usize>,
  modules: impl Iterator<Item = &'a ModuleIdentifier>,
  outgoings: &IdentifierMap<Vec<ModuleIdentifier>>,
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
    for con in outgoings.get(&id).expect("should have outgoings").iter() {
      q.push_back((*con, depth + 1));
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

#[cacheable]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ManifestAssetType {
  #[default]
  Unknown,
  Asset,
  Css,
  JavaScript,
  Wasm,
  Custom(#[cacheable(with=AsPreset)] Ustr),
}

impl fmt::Display for ManifestAssetType {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      ManifestAssetType::Unknown => write!(f, "unknown"),
      ManifestAssetType::Asset => write!(f, "asset"),
      ManifestAssetType::Css => write!(f, "css"),
      ManifestAssetType::JavaScript => write!(f, "javascript"),
      ManifestAssetType::Wasm => write!(f, "wasm"),
      ManifestAssetType::Custom(custom) => write!(f, "{custom}"),
    }
  }
}

impl From<String> for ManifestAssetType {
  fn from(value: String) -> Self {
    match value.as_str() {
      "unknown" => ManifestAssetType::Unknown,
      "asset" => ManifestAssetType::Asset,
      "css" => ManifestAssetType::Css,
      "javascript" => ManifestAssetType::JavaScript,
      "wasm" => ManifestAssetType::Wasm,
      _ => ManifestAssetType::Custom(value.into()),
    }
  }
}
