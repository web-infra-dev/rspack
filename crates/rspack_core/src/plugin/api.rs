use std::{fmt::Debug, path::Path, sync::Arc};

use rspack_error::{IntoTWithDiagnosticArray, Result, TWithDiagnosticArray};
use rspack_hash::RspackHashDigest;
use rspack_loader_runner::{Content, LoaderContext, ResourceData};
use rspack_sources::{BoxSource, Source};
use rspack_util::fx_dashmap::FxDashMap;
use tokio::sync::mpsc::UnboundedSender;

use crate::{
  AdditionalChunkRuntimeRequirementsArgs, AdditionalModuleRequirementsArgs, AssetEmittedArgs,
  AssetInfo, BoxLoader, BoxModule, BuildTimeExecutionOption, Chunk, ChunkAssetArgs, ChunkHashArgs,
  CodeGenerationResults, Compilation, CompilationParams, CompilerHooks, CompilerOptions,
  ContentHashArgs, DependencyId, DoneArgs, FactorizeArgs, JsChunkHashArgs, LoaderRunnerContext,
  Module, ModuleFactoryResult, ModuleIdentifier, ModuleType, NormalModule,
  NormalModuleAfterResolveArgs, NormalModuleBeforeResolveArgs, NormalModuleCreateData,
  OptimizeChunksArgs, ParserAndGenerator, PluginContext, ProcessAssetsArgs, RenderArgs,
  RenderChunkArgs, RenderManifestArgs, RenderModuleContentArgs, RenderStartupArgs, Resolver,
  RuntimeModule, RuntimeRequirementsInTreeArgs, SourceType, ThisCompilationArgs,
};

// use anyhow::{Context, Result};
pub type PluginCompilationHookOutput = Result<()>;
pub type PluginThisCompilationHookOutput = Result<()>;
pub type PluginMakeHookOutput = Result<()>;
pub type PluginBuildEndHookOutput = Result<()>;
pub type PluginProcessAssetsHookOutput = Result<()>;
pub type PluginReadResourceOutput = Result<Option<Content>>;
pub type PluginFactorizeHookOutput = Result<Option<ModuleFactoryResult>>;
pub type PluginNormalModuleFactoryCreateModuleHookOutput = Result<Option<BoxModule>>;
pub type PluginNormalModuleFactoryModuleHookOutput = Result<BoxModule>;
pub type PluginNormalModuleFactoryResolveForSchemeOutput = Result<(ResourceData, bool)>;
pub type PluginNormalModuleFactoryBeforeResolveOutput = Result<Option<bool>>;
pub type PluginNormalModuleFactoryAfterResolveOutput = Result<Option<bool>>;
pub type PluginContentHashHookOutput = Result<Option<(SourceType, RspackHashDigest)>>;
pub type PluginChunkHashHookOutput = Result<()>;
pub type PluginRenderManifestHookOutput = Result<TWithDiagnosticArray<Vec<RenderManifestEntry>>>;
pub type PluginRenderChunkHookOutput = Result<Option<BoxSource>>;
pub type PluginProcessAssetsOutput = Result<()>;
pub type PluginOptimizeChunksOutput = Result<()>;
pub type PluginAdditionalChunkRuntimeRequirementsOutput = Result<()>;
pub type PluginRuntimeRequirementsInTreeOutput = Result<()>;
pub type PluginAdditionalModuleRequirementsOutput = Result<()>;
pub type PluginRenderModuleContentOutput<'a> = Result<RenderModuleContentArgs<'a>>;
pub type PluginRenderStartupHookOutput = Result<Option<BoxSource>>;
pub type PluginRenderHookOutput = Result<Option<BoxSource>>;
pub type PluginJsChunkHashHookOutput = Result<()>;
pub type PluginShouldEmitHookOutput = Result<Option<bool>>;

#[async_trait::async_trait]
pub trait Plugin: Debug + Send + Sync {
  fn name(&self) -> &'static str {
    "unknown"
  }

  fn apply(
    &self,
    _ctx: PluginContext<&mut ApplyContext>,
    _options: &mut CompilerOptions,
  ) -> Result<()> {
    Ok(())
  }

  async fn this_compilation(
    &self,
    _args: ThisCompilationArgs<'_>,
    _params: &CompilationParams,
  ) -> PluginThisCompilationHookOutput {
    Ok(())
  }

  async fn done<'s, 'c>(
    &self,
    _ctx: PluginContext,
    _args: DoneArgs<'s, 'c>,
  ) -> PluginBuildEndHookOutput {
    Ok(())
  }

  async fn read_resource(&self, _resource_data: &ResourceData) -> PluginReadResourceOutput {
    Ok(None)
  }
  /**
   * factorize hook will generate BoxModule which will be used to generate ModuleGraphModule.
   * It is used to handle the generation of those modules which are not normal, such as External Module
   * It behaves like a BailHook hook.
   * NOTICE: The factorize hook is a temporary solution and will be replaced with the real factorize hook later
   */
  async fn factorize(
    &self,
    _ctx: PluginContext,
    _args: &mut FactorizeArgs<'_>,
  ) -> PluginFactorizeHookOutput {
    Ok(None)
  }

  async fn before_resolve(
    &self,
    _ctx: PluginContext,
    _args: &mut NormalModuleBeforeResolveArgs,
  ) -> PluginNormalModuleFactoryBeforeResolveOutput {
    Ok(None)
  }

  async fn after_resolve(
    &self,
    _ctx: PluginContext,
    _args: &mut NormalModuleAfterResolveArgs<'_>,
  ) -> PluginNormalModuleFactoryAfterResolveOutput {
    Ok(None)
  }

  async fn context_module_before_resolve(
    &self,
    _ctx: PluginContext,
    _args: &mut NormalModuleBeforeResolveArgs,
  ) -> PluginNormalModuleFactoryBeforeResolveOutput {
    Ok(None)
  }

  async fn normal_module_factory_create_module(
    &self,
    _ctx: PluginContext,
    _args: &mut NormalModuleCreateData<'_>,
  ) -> PluginNormalModuleFactoryCreateModuleHookOutput {
    Ok(None)
  }

  async fn normal_module_factory_module(
    &self,
    _ctx: PluginContext,
    module: BoxModule,
    _args: &mut NormalModuleCreateData<'_>,
  ) -> PluginNormalModuleFactoryModuleHookOutput {
    Ok(module)
  }

  async fn normal_module_factory_resolve_for_scheme(
    &self,
    _ctx: PluginContext,
    args: ResourceData,
  ) -> PluginNormalModuleFactoryResolveForSchemeOutput {
    Ok((args, false))
  }

  fn normal_module_loader(
    &self,
    _ctx: PluginContext,
    _loader_context: &mut LoaderContext<LoaderRunnerContext>,
    _module: &NormalModule,
  ) -> Result<()> {
    Ok(())
  }

  async fn content_hash(
    &self,
    _ctx: PluginContext,
    _args: &ContentHashArgs<'_>,
  ) -> PluginContentHashHookOutput {
    Ok(None)
  }

  async fn chunk_hash(
    &self,
    _ctx: PluginContext,
    _args: &mut ChunkHashArgs<'_>,
  ) -> PluginChunkHashHookOutput {
    Ok(())
  }

  async fn render_manifest(
    &self,
    _ctx: PluginContext,
    _args: RenderManifestArgs<'_>,
  ) -> PluginRenderManifestHookOutput {
    Ok(vec![].with_empty_diagnostic())
  }

  // JavascriptModulesPlugin hook
  async fn render_chunk(
    &self,
    _ctx: PluginContext,
    _args: &RenderChunkArgs,
  ) -> PluginRenderChunkHookOutput {
    Ok(None)
  }

  async fn module_asset(&self, _module: ModuleIdentifier, _asset_name: String) -> Result<()> {
    Ok(())
  }

  /// webpack `compilation.hooks.chunkAsset`
  async fn chunk_asset(&self, _args: &ChunkAssetArgs) -> Result<()> {
    Ok(())
  }

  // JavascriptModulesPlugin hook
  fn render(&self, _ctx: PluginContext, _args: &RenderArgs) -> PluginRenderStartupHookOutput {
    Ok(None)
  }

  // JavascriptModulesPlugin hook
  fn render_startup(
    &self,
    _ctx: PluginContext,
    _args: &RenderStartupArgs,
  ) -> PluginRenderStartupHookOutput {
    Ok(None)
  }

  // JavascriptModulesPlugin hook
  fn render_module_content<'a>(
    &'a self,
    _ctx: PluginContext,
    args: RenderModuleContentArgs<'a>,
  ) -> PluginRenderModuleContentOutput<'a> {
    Ok(args)
  }

  // JavascriptModulesPlugin hook
  fn js_chunk_hash(
    &self,
    _ctx: PluginContext,
    _args: &mut JsChunkHashArgs,
  ) -> PluginJsChunkHashHookOutput {
    Ok(())
  }

  async fn additional_chunk_runtime_requirements(
    &self,
    _ctx: PluginContext,
    _args: &mut AdditionalChunkRuntimeRequirementsArgs,
  ) -> PluginAdditionalChunkRuntimeRequirementsOutput {
    Ok(())
  }

  async fn additional_tree_runtime_requirements(
    &self,
    _ctx: PluginContext,
    _args: &mut AdditionalChunkRuntimeRequirementsArgs,
  ) -> PluginAdditionalChunkRuntimeRequirementsOutput {
    Ok(())
  }

  fn runtime_requirements_in_module(
    &self,
    _ctx: PluginContext,
    _args: &mut AdditionalModuleRequirementsArgs,
  ) -> PluginAdditionalModuleRequirementsOutput {
    Ok(())
  }

  async fn runtime_requirements_in_tree(
    &self,
    _ctx: PluginContext,
    _args: &mut RuntimeRequirementsInTreeArgs,
  ) -> PluginRuntimeRequirementsInTreeOutput {
    Ok(())
  }

  async fn process_assets_stage_additional(
    &self,
    _ctx: PluginContext,
    _args: ProcessAssetsArgs<'_>,
  ) -> PluginProcessAssetsOutput {
    Ok(())
  }

  async fn process_assets_stage_pre_process(
    &self,
    _ctx: PluginContext,
    _args: ProcessAssetsArgs<'_>,
  ) -> PluginProcessAssetsOutput {
    Ok(())
  }

  async fn process_assets_stage_derived(
    &self,
    _ctx: PluginContext,
    _args: ProcessAssetsArgs<'_>,
  ) -> PluginProcessAssetsOutput {
    Ok(())
  }

  async fn process_assets_stage_additions(
    &self,
    _ctx: PluginContext,
    _args: ProcessAssetsArgs<'_>,
  ) -> PluginProcessAssetsOutput {
    Ok(())
  }

  async fn process_assets_stage_none(
    &self,
    _ctx: PluginContext,
    _args: ProcessAssetsArgs<'_>,
  ) -> PluginProcessAssetsOutput {
    Ok(())
  }

  async fn process_assets_stage_optimize(
    &self,
    _ctx: PluginContext,
    _args: ProcessAssetsArgs<'_>,
  ) -> PluginProcessAssetsOutput {
    Ok(())
  }

  async fn process_assets_stage_optimize_count(
    &self,
    _ctx: PluginContext,
    _args: ProcessAssetsArgs<'_>,
  ) -> PluginProcessAssetsOutput {
    Ok(())
  }

  async fn process_assets_stage_optimize_compatibility(
    &self,
    _ctx: PluginContext,
    _args: ProcessAssetsArgs<'_>,
  ) -> PluginProcessAssetsOutput {
    Ok(())
  }

  async fn process_assets_stage_optimize_size(
    &self,
    _ctx: PluginContext,
    _args: ProcessAssetsArgs<'_>,
  ) -> PluginProcessAssetsOutput {
    Ok(())
  }

  async fn process_assets_stage_dev_tooling(
    &self,
    _ctx: PluginContext,
    _args: ProcessAssetsArgs<'_>,
  ) -> PluginProcessAssetsOutput {
    Ok(())
  }

  async fn process_assets_stage_optimize_inline(
    &self,
    _ctx: PluginContext,
    _args: ProcessAssetsArgs<'_>,
  ) -> PluginProcessAssetsOutput {
    Ok(())
  }

  async fn process_assets_stage_summarize(
    &self,
    _ctx: PluginContext,
    _args: ProcessAssetsArgs<'_>,
  ) -> PluginProcessAssetsOutput {
    Ok(())
  }

  async fn process_assets_stage_optimize_hash(
    &self,
    _ctx: PluginContext,
    _args: ProcessAssetsArgs<'_>,
  ) -> PluginProcessAssetsOutput {
    Ok(())
  }

  async fn process_assets_stage_optimize_transfer(
    &self,
    _ctx: PluginContext,
    _args: ProcessAssetsArgs<'_>,
  ) -> PluginProcessAssetsOutput {
    Ok(())
  }

  async fn process_assets_stage_analyse(
    &self,
    _ctx: PluginContext,
    _args: ProcessAssetsArgs<'_>,
  ) -> PluginProcessAssetsOutput {
    Ok(())
  }

  async fn process_assets_stage_report(
    &self,
    _ctx: PluginContext,
    _args: ProcessAssetsArgs<'_>,
  ) -> PluginProcessAssetsOutput {
    Ok(())
  }

  async fn after_process_assets(
    &self,
    _ctx: PluginContext,
    _args: ProcessAssetsArgs<'_>,
  ) -> PluginProcessAssetsOutput {
    Ok(())
  }

  async fn optimize_chunks(
    &self,
    _ctx: PluginContext,
    _args: OptimizeChunksArgs<'_>,
  ) -> PluginOptimizeChunksOutput {
    Ok(())
  }

  async fn optimize_modules(&self, _compilation: &mut Compilation) -> Result<()> {
    Ok(())
  }

  async fn after_optimize_modules(&self, _compilation: &mut Compilation) -> Result<()> {
    Ok(())
  }

  async fn optimize_dependencies(&self, _compilation: &mut Compilation) -> Result<Option<()>> {
    Ok(None)
  }

  async fn optimize_code_generation(&self, _compilation: &mut Compilation) -> Result<Option<()>> {
    Ok(None)
  }

  async fn optimize_tree(&self, _compilation: &mut Compilation) -> Result<()> {
    Ok(())
  }

  async fn optimize_chunk_modules(&self, _args: OptimizeChunksArgs<'_>) -> Result<()> {
    Ok(())
  }

  async fn before_compile(&self, _params: &CompilationParams) -> Result<()> {
    Ok(())
  }

  async fn after_compile(&self, _compilation: &mut Compilation) -> Result<()> {
    Ok(())
  }

  async fn finish_make(&self, _compilation: &mut Compilation) -> Result<()> {
    Ok(())
  }

  async fn finish_modules(&self, _modules: &mut Compilation) -> Result<()> {
    Ok(())
  }

  /// Webpack resolves loaders in `NormalModuleFactory`,
  /// Rspack resolves it when normalizing configuration.
  /// So this hook is used to resolve inline loader (inline loader requests).
  async fn resolve_loader(
    &self,
    _compiler_options: &CompilerOptions,
    _context: &Path,
    _resolver: &Resolver,
    _loader_request: &str,
    _loader_options: Option<&str>,
  ) -> Result<Option<BoxLoader>> {
    Ok(None)
  }

  async fn before_loaders(&self, _module: &mut NormalModule) -> Result<()> {
    Ok(())
  }

  async fn build_module(&self, _module: &mut dyn Module) -> Result<()> {
    Ok(())
  }

  async fn succeed_module(&self, _module: &dyn Module) -> Result<()> {
    Ok(())
  }

  async fn still_valid_module(&self, _module: &dyn Module) -> Result<()> {
    Ok(())
  }

  async fn runtime_module(
    &self,
    _module: &mut dyn RuntimeModule,
    _source: Arc<dyn Source>,
    _chunk: &Chunk,
  ) -> Result<Option<String>> {
    Ok(None)
  }

  fn module_ids(&self, _modules: &mut Compilation) -> Result<()> {
    Ok(())
  }

  fn chunk_ids(&self, _compilation: &mut Compilation) -> Result<()> {
    Ok(())
  }

  async fn emit(&self, _compilation: &mut Compilation) -> Result<()> {
    Ok(())
  }

  async fn asset_emitted(&self, _args: &AssetEmittedArgs) -> Result<()> {
    Ok(())
  }

  async fn should_emit(&self, _compilation: &mut Compilation) -> PluginShouldEmitHookOutput {
    Ok(None)
  }

  async fn after_emit(&self, _compilation: &mut Compilation) -> Result<()> {
    Ok(())
  }

  fn seal(&self, _compilation: &mut Compilation) -> Result<()> {
    Ok(())
  }

  fn prepare_execute_module(
    &self,
    _id: DependencyId,
    _options: &BuildTimeExecutionOption,
    _import_module_informer: UnboundedSender<Result<String>>,
  ) -> Result<()> {
    Ok(())
  }

  fn execute_module(
    &self,
    _entry: ModuleIdentifier,
    _request: &str,
    _options: &BuildTimeExecutionOption,
    _runtime_modules: Vec<ModuleIdentifier>,
    _codegen_results: &CodeGenerationResults,
    _id: u32,
  ) -> Result<()> {
    Ok(())
  }
}

pub type BoxPlugin = Box<dyn Plugin>;

pub trait PluginExt {
  fn boxed(self) -> BoxPlugin;
}

impl<T: Plugin + 'static> PluginExt for T {
  fn boxed(self) -> BoxPlugin {
    Box::new(self)
  }
}

// #[derive(Debug, Clone, Serialize, Deserialize)]
// #[serde(untagged)]
// pub enum AssetContent {
//   Buffer(Vec<u8>),
//   String(String),
// }

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

// pub trait Parser: Debug + Sync + Send {
//   fn parse(
//     &self,
//     module_type: ModuleType,
//     args: ParseModuleArgs,
//   ) -> Result<TWithDiagnosticArray<BoxModule>>;
// }

// pub type BoxedParser = Box<dyn Parser>;
pub type BoxedParserAndGenerator = Box<dyn ParserAndGenerator>;
pub type BoxedParserAndGeneratorBuilder =
  Box<dyn 'static + Send + Sync + Fn() -> BoxedParserAndGenerator>;

pub struct ApplyContext<'c> {
  pub(crate) registered_parser_and_generator_builder:
    &'c mut FxDashMap<ModuleType, BoxedParserAndGeneratorBuilder>,
  pub compiler_hooks: &'c mut CompilerHooks,
}

impl<'c> ApplyContext<'c> {
  pub fn register_parser_and_generator_builder(
    &mut self,
    module_type: ModuleType,
    parser_and_generator_builder: BoxedParserAndGeneratorBuilder,
  ) {
    self
      .registered_parser_and_generator_builder
      .insert(module_type, parser_and_generator_builder);
  }
}
