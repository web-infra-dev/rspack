use std::{fmt::Debug, path::Path};

use rspack_error::Result;
use rspack_loader_runner::{Content, ResourceData};
use rspack_sources::BoxSource;
use rustc_hash::FxHashMap as HashMap;

use crate::{
  AdditionalChunkRuntimeRequirementsArgs, AssetInfo, BoxLoader, BoxModule, ChunkAssetArgs,
  ChunkHashArgs, Compilation, CompilationArgs, CompilerOptions, ContentHashArgs, DoneArgs,
  FactorizeArgs, JsChunkHashArgs, Module, ModuleArgs, ModuleFactoryResult, ModuleType,
  NormalModule, NormalModuleBeforeResolveArgs, NormalModuleFactoryContext,
  NormalModuleFactoryResolveForSchemeArgs, OptimizeChunksArgs, ParserAndGenerator, PluginContext,
  ProcessAssetsArgs, RenderArgs, RenderChunkArgs, RenderManifestArgs, RenderModuleContentArgs,
  RenderStartupArgs, Resolver, SourceType, ThisCompilationArgs,
};

// use anyhow::{Context, Result};
pub type PluginCompilationHookOutput = Result<()>;
pub type PluginThisCompilationHookOutput = Result<()>;
pub type PluginMakeHookOutput = Result<()>;
pub type PluginBuildEndHookOutput = Result<()>;
pub type PluginProcessAssetsHookOutput = Result<()>;
pub type PluginReadResourceOutput = Result<Option<Content>>;
pub type PluginFactorizeHookOutput = Result<Option<ModuleFactoryResult>>;
pub type PluginModuleHookOutput = Result<Option<BoxModule>>;
pub type PluginNormalModuleFactoryResolveForSchemeOutput = Result<Option<ResourceData>>;
pub type PluginNormalModuleFactoryBeforeResolveOutput = Result<Option<bool>>;
pub type PluginContentHashHookOutput = Result<Option<(SourceType, String)>>;
pub type PluginChunkHashHookOutput = Result<Option<u64>>;
pub type PluginRenderManifestHookOutput = Result<Vec<RenderManifestEntry>>;
pub type PluginRenderChunkHookOutput = Result<Option<BoxSource>>;
pub type PluginProcessAssetsOutput = Result<()>;
pub type PluginOptimizeChunksOutput = Result<()>;
pub type PluginAdditionalChunkRuntimeRequirementsOutput = Result<()>;
pub type PluginRenderModuleContentOutput = Result<Option<BoxSource>>;
pub type PluginRenderStartupHookOutput = Result<Option<BoxSource>>;
pub type PluginRenderHookOutput = Result<Option<BoxSource>>;
pub type PluginJsChunkHashHookOutput = Result<()>;

#[async_trait::async_trait]
pub trait Plugin: Debug + Send + Sync {
  fn name(&self) -> &'static str {
    "unknown"
  }

  fn apply(&mut self, _ctx: PluginContext<&mut ApplyContext>) -> Result<()> {
    Ok(())
  }

  async fn compilation(&mut self, _args: CompilationArgs<'_>) -> PluginCompilationHookOutput {
    Ok(())
  }

  async fn this_compilation(
    &mut self,
    _args: ThisCompilationArgs<'_>,
  ) -> PluginThisCompilationHookOutput {
    Ok(())
  }

  async fn make(&self, _ctx: PluginContext, _compilation: &Compilation) -> PluginMakeHookOutput {
    Ok(())
  }

  async fn done<'s, 'c>(
    &mut self,
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
    _args: FactorizeArgs<'_>,
    _job_ctx: &mut NormalModuleFactoryContext,
  ) -> PluginFactorizeHookOutput {
    Ok(None)
  }

  async fn before_resolve(
    &self,
    _ctx: PluginContext,
    _args: &NormalModuleBeforeResolveArgs,
  ) -> PluginNormalModuleFactoryBeforeResolveOutput {
    Ok(None)
  }

  async fn context_module_before_resolve(
    &self,
    _ctx: PluginContext,
    _args: &NormalModuleBeforeResolveArgs,
  ) -> PluginNormalModuleFactoryBeforeResolveOutput {
    Ok(None)
  }

  async fn module(&self, _ctx: PluginContext, _args: &ModuleArgs) -> PluginModuleHookOutput {
    Ok(None)
  }

  async fn normal_module_factory_resolve_for_scheme(
    &self,
    _ctx: PluginContext,
    _args: &NormalModuleFactoryResolveForSchemeArgs,
  ) -> PluginNormalModuleFactoryResolveForSchemeOutput {
    Ok(None)
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
    _args: &ChunkHashArgs<'_>,
  ) -> PluginChunkHashHookOutput {
    Ok(None)
  }

  async fn render_manifest(
    &self,
    _ctx: PluginContext,
    _args: RenderManifestArgs<'_>,
  ) -> PluginRenderManifestHookOutput {
    Ok(vec![])
  }

  // JavascriptModulesPlugin hook
  async fn render_chunk(
    &self,
    _ctx: PluginContext,
    _args: &RenderChunkArgs,
  ) -> PluginRenderChunkHookOutput {
    Ok(None)
  }

  /// webpack `compilation.hooks.chunkAsset`
  async fn chunk_asset(&mut self, _args: &ChunkAssetArgs) -> Result<()> {
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
  fn render_module_content(
    &self,
    _ctx: PluginContext,
    _args: &RenderModuleContentArgs,
  ) -> PluginRenderModuleContentOutput {
    Ok(None)
  }

  // JavascriptModulesPlugin hook
  fn js_chunk_hash(
    &self,
    _ctx: PluginContext,
    _args: &mut JsChunkHashArgs,
  ) -> PluginJsChunkHashHookOutput {
    Ok(())
  }

  fn additional_chunk_runtime_requirements(
    &self,
    _ctx: PluginContext,
    _args: &mut AdditionalChunkRuntimeRequirementsArgs,
  ) -> PluginAdditionalChunkRuntimeRequirementsOutput {
    Ok(())
  }

  fn additional_tree_runtime_requirements(
    &self,
    _ctx: PluginContext,
    _args: &mut AdditionalChunkRuntimeRequirementsArgs,
  ) -> PluginAdditionalChunkRuntimeRequirementsOutput {
    Ok(())
  }

  fn runtime_requirements_in_tree(
    &self,
    _ctx: PluginContext,
    _args: &mut AdditionalChunkRuntimeRequirementsArgs,
  ) -> PluginAdditionalChunkRuntimeRequirementsOutput {
    Ok(())
  }

  async fn process_assets_stage_additional(
    &mut self,
    _ctx: PluginContext,
    _args: ProcessAssetsArgs<'_>,
  ) -> PluginProcessAssetsOutput {
    Ok(())
  }

  async fn process_assets_stage_additions(
    &mut self,
    _ctx: PluginContext,
    _args: ProcessAssetsArgs<'_>,
  ) -> PluginProcessAssetsOutput {
    Ok(())
  }

  async fn process_assets_stage_pre_process(
    &mut self,
    _ctx: PluginContext,
    _args: ProcessAssetsArgs<'_>,
  ) -> PluginProcessAssetsOutput {
    Ok(())
  }

  async fn process_assets_stage_none(
    &mut self,
    _ctx: PluginContext,
    _args: ProcessAssetsArgs<'_>,
  ) -> PluginProcessAssetsOutput {
    Ok(())
  }

  async fn process_assets_stage_optimize_size(
    &mut self,
    _ctx: PluginContext,
    _args: ProcessAssetsArgs<'_>,
  ) -> PluginProcessAssetsOutput {
    Ok(())
  }

  async fn process_assets_stage_dev_tooling(
    &mut self,
    _ctx: PluginContext,
    _args: ProcessAssetsArgs<'_>,
  ) -> PluginProcessAssetsOutput {
    Ok(())
  }

  async fn process_assets_stage_optimize_inline(
    &mut self,
    _ctx: PluginContext,
    _args: ProcessAssetsArgs<'_>,
  ) -> PluginProcessAssetsOutput {
    Ok(())
  }

  async fn process_assets_stage_summarize(
    &mut self,
    _ctx: PluginContext,
    _args: ProcessAssetsArgs<'_>,
  ) -> PluginProcessAssetsOutput {
    Ok(())
  }

  async fn process_assets_stage_optimize_hash(
    &mut self,
    _ctx: PluginContext,
    _args: ProcessAssetsArgs<'_>,
  ) -> PluginProcessAssetsOutput {
    Ok(())
  }

  async fn process_assets_stage_report(
    &mut self,
    _ctx: PluginContext,
    _args: ProcessAssetsArgs<'_>,
  ) -> PluginProcessAssetsOutput {
    Ok(())
  }

  async fn optimize_chunks(
    &mut self,
    _ctx: PluginContext,
    _args: OptimizeChunksArgs<'_>,
  ) -> PluginOptimizeChunksOutput {
    Ok(())
  }

  async fn optimize_modules(&mut self, _compilation: &mut Compilation) -> Result<()> {
    Ok(())
  }

  async fn optimize_chunk_modules(&mut self, _args: OptimizeChunksArgs<'_>) -> Result<()> {
    Ok(())
  }

  async fn before_compile(&mut self) -> Result<()> {
    Ok(())
  }

  async fn finish_modules(&mut self, _modules: &mut Compilation) -> Result<()> {
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

  fn module_ids(&mut self, _modules: &mut Compilation) -> Result<()> {
    Ok(())
  }

  fn chunk_ids(&mut self, _compilation: &mut Compilation) -> Result<()> {
    Ok(())
  }

  async fn emit(&mut self, _compilation: &mut Compilation) -> Result<()> {
    Ok(())
  }

  async fn after_emit(&mut self, _compilation: &mut Compilation) -> Result<()> {
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
  pub(crate) source: BoxSource,
  filename: String,
  pub(crate) info: AssetInfo,
  // pub identifier: String,
  // hash?: string;
  // auxiliary?: boolean;
}

impl RenderManifestEntry {
  pub fn new(source: BoxSource, filename: String, info: AssetInfo) -> Self {
    Self {
      source,
      filename,
      info,
    }
  }

  pub fn source(&self) -> &BoxSource {
    &self.source
  }

  pub fn filename(&self) -> &str {
    &self.filename
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

#[derive(Default)]
pub struct ApplyContext {
  // pub(crate) registered_parser: HashMap<ModuleType, BoxedParser>,
  pub(crate) registered_parser_and_generator_builder:
    HashMap<ModuleType, BoxedParserAndGeneratorBuilder>,
}

impl ApplyContext {
  // pub fn register_parser(&mut self, module_type: ModuleType, parser: BoxedParser) {
  //   self.registered_parser.insert(module_type, parser);
  // }

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
