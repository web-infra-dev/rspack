use std::fmt::Debug;

use hashbrown::HashMap;

use rspack_error::Result;
use rspack_loader_runner::{Content, ResourceData};
use rspack_sources::BoxSource;

use crate::{
  AdditionalChunkRuntimeRequirementsArgs, BoxModule, ChunkUkey, Compilation, CompilationArgs,
  DoneArgs, FactorizeArgs, Module, ModuleArgs, ModuleType, NormalModuleFactoryContext,
  OptimizeChunksArgs, ParserAndGenerator, PluginContext, ProcessAssetsArgs, RenderManifestArgs,
  ThisCompilationArgs, TransformAst, TransformResult,
};

// use anyhow::{Context, Result};
pub type PluginCompilationHookOutput = Result<()>;
pub type PluginThisCompilationHookOutput = Result<()>;
pub type PluginMakeHookOutput = Result<()>;
pub type PluginBuildEndHookOutput = Result<()>;
pub type PluginProcessAssetsHookOutput = Result<()>;
pub type PluginReadResourceOutput = Result<Option<Content>>;
pub type PluginLoadHookOutput = Result<Option<Content>>;
pub type PluginTransformOutput = Result<TransformResult>;
// FIXME: factorize should only return `BoxModule`, the first string currently is used to generate `id`(moduleIds)
pub type PluginFactorizeHookOutput = Result<Option<(String, BoxModule)>>;
pub type PluginModuleHookOutput = Result<Option<BoxModule>>;
pub type PluginRenderManifestHookOutput = Result<Vec<RenderManifestEntry>>;
pub type PluginParseModuleHookOutput = Result<BoxModule>;
pub type PluginParseOutput = Result<TransformAst>;
pub type PluginGenerateOutput = Result<Content>;
pub type PluginProcessAssetsOutput = Result<()>;
pub type PluginOptimizeChunksOutput = Result<()>;
pub type PluginAdditionalChunkRuntimeRequirementsOutput = Result<()>;
// pub type PluginTransformAstHookOutput = Result<ast::Module>;

// pub type PluginTransformHookOutput = Result<TransformResult>;
// pub type PluginTapGeneratedChunkHookOutput = Result<()>;
// pub type PluginRenderChunkHookOutput = Result<OutputChunk>;

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

  fn make(&self, _ctx: PluginContext, _compilation: &Compilation) -> PluginMakeHookOutput {
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

  async fn module(&self, _ctx: PluginContext, _args: &ModuleArgs) -> PluginModuleHookOutput {
    Ok(None)
  }

  fn render_manifest(
    &self,
    _ctx: PluginContext,
    _args: RenderManifestArgs,
  ) -> PluginRenderManifestHookOutput {
    Ok(vec![])
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

  async fn process_assets(
    &mut self,
    _ctx: PluginContext,
    _args: ProcessAssetsArgs<'_>,
  ) -> PluginProcessAssetsOutput {
    Ok(())
  }

  fn optimize_chunks(
    &mut self,
    _ctx: PluginContext,
    _args: OptimizeChunksArgs,
  ) -> PluginOptimizeChunksOutput {
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
}

// #[derive(Debug, Clone, Serialize, Deserialize)]
// #[serde(untagged)]
// pub enum AssetContent {
//   Buffer(Vec<u8>),
//   String(String),
// }

#[derive(Debug)]
pub struct PathData {
  pub chunk_ukey: ChunkUkey,
}

#[derive(Debug)]
pub struct RenderManifestEntry {
  pub(crate) source: BoxSource,
  filename: String,
  pub(crate) path_options: PathData,
  // info?: AssetInfo;
  // pub identifier: String,
  // hash?: string;
  // auxiliary?: boolean;
}

impl RenderManifestEntry {
  pub fn new(source: BoxSource, filename: String, path_options: PathData) -> Self {
    Self {
      source,
      filename,
      path_options,
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
