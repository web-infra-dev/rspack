use std::fmt::Debug;

use rspack_error::{IntoTWithDiagnosticArray, Result, TWithDiagnosticArray};
use rspack_hash::RspackHashDigest;
use rspack_loader_runner::ResourceData;
use rspack_sources::BoxSource;
use rspack_util::fx_dashmap::FxDashMap;

use crate::{
  AdditionalChunkRuntimeRequirementsArgs, AdditionalModuleRequirementsArgs, AssetInfo, BoxModule,
  ChunkHashArgs, Compilation, CompilationHooks, CompilerHooks, CompilerOptions, ContentHashArgs,
  ContextModuleFactoryHooks, GeneratorOptions, JsChunkHashArgs, ModuleIdentifier, ModuleType,
  NormalModuleFactoryHooks, NormalModuleHooks, OptimizeChunksArgs, ParserAndGenerator,
  ParserOptions, PluginContext, RenderArgs, RenderChunkArgs, RenderManifestArgs,
  RenderModuleContentArgs, RenderStartupArgs, RuntimeRequirementsInTreeArgs, SourceType,
};

#[derive(Debug, Clone)]
pub struct BeforeResolveArgs {
  pub request: String,
  pub context: String,
}

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

  async fn optimize_chunks(
    &self,
    _ctx: PluginContext,
    _args: OptimizeChunksArgs<'_>,
  ) -> PluginOptimizeChunksOutput {
    Ok(())
  }

  async fn optimize_dependencies(&self, _compilation: &mut Compilation) -> Result<Option<()>> {
    Ok(None)
  }

  async fn optimize_code_generation(&self, _compilation: &mut Compilation) -> Result<Option<()>> {
    Ok(None)
  }

  fn module_ids(&self, _modules: &mut Compilation) -> Result<()> {
    Ok(())
  }

  fn chunk_ids(&self, _compilation: &mut Compilation) -> Result<()> {
    Ok(())
  }

  fn seal(&self, _compilation: &mut Compilation) -> Result<()> {
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
pub type BoxedParserAndGeneratorBuilder = Box<
  dyn 'static
    + Send
    + Sync
    + Fn(Option<&ParserOptions>, Option<&GeneratorOptions>) -> BoxedParserAndGenerator,
>;

pub struct ApplyContext<'c> {
  pub(crate) registered_parser_and_generator_builder:
    &'c mut FxDashMap<ModuleType, BoxedParserAndGeneratorBuilder>,
  pub compiler_hooks: &'c mut CompilerHooks,
  pub compilation_hooks: &'c mut CompilationHooks,
  pub normal_module_factory_hooks: &'c mut NormalModuleFactoryHooks,
  pub context_module_factory_hooks: &'c mut ContextModuleFactoryHooks,
  pub normal_module_hooks: &'c mut NormalModuleHooks,
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
