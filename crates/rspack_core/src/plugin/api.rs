use std::fmt::Debug;

use crate::{
  BoxModule, FactorizeAndBuildArgs, ModuleType, NormalModuleFactoryContext, OptimizeChunksArgs,
  ParseModuleArgs, PluginContext, ProcessAssetsArgs, RenderManifestArgs, RenderRuntimeArgs,
  TransformAst, TransformResult,
};
use rspack_error::{Result, TWithDiagnosticArray};
use rspack_loader_runner::{Content, ResourceData};
use rspack_sources::{BoxSource, RawSource};

// use anyhow::{Context, Result};
use hashbrown::HashMap;
pub type PluginBuildStartHookOutput = Result<()>;
pub type PluginBuildEndHookOutput = Result<()>;
pub type PluginProcessAssetsHookOutput = Result<()>;
pub type PluginReadResourceOutput = Result<Option<Content>>;
pub type PluginLoadHookOutput = Result<Option<Content>>;
pub type PluginTransformOutput = Result<TransformResult>;
pub type PluginFactorizeAndBuildHookOutput = Result<Option<(String, BoxModule)>>;
pub type PluginRenderManifestHookOutput = Result<Vec<RenderManifestEntry>>;
pub type PluginRenderRuntimeHookOutput = Result<Vec<RawSource>>;
pub type PluginParseModuleHookOutput = Result<BoxModule>;
pub type PluginParseOutput = Result<TransformAst>;
pub type PluginGenerateOutput = Result<Content>;
pub type PluginProcessAssetsOutput = Result<()>;
pub type PluginOptimizeChunksOutput = Result<()>;
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

  fn build_start(&mut self) -> PluginBuildStartHookOutput {
    Ok(())
  }

  async fn done(&mut self) -> PluginBuildEndHookOutput {
    Ok(())
  }

  async fn read_resource(&self, _resource_data: &ResourceData) -> PluginReadResourceOutput {
    Ok(None)
  }
  /**
   * factorize_and_build hook will generate BoxModule which will be used to generate ModuleGraphModule.
   * It is used to handle the generation of those modules which are not normal, such as External Module
   * It behaves like a BailHook hook.
   * NOTICE: The factorize_and_build hook is a temporary solution and will be replaced with the real factorize hook later
   */
  async fn factorize_and_build(
    &self,
    _ctx: PluginContext,
    _args: FactorizeAndBuildArgs,
    _job_ctx: &mut NormalModuleFactoryContext,
  ) -> PluginFactorizeAndBuildHookOutput {
    Ok(None)
  }

  fn render_manifest(
    &self,
    _ctx: PluginContext,
    _args: RenderManifestArgs,
  ) -> PluginRenderManifestHookOutput {
    Ok(vec![])
  }
  fn render_runtime(
    &self,
    _ctx: PluginContext,
    args: RenderRuntimeArgs,
  ) -> PluginRenderRuntimeHookOutput {
    Ok(args.sources)
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
}

// #[derive(Debug, Clone, Serialize, Deserialize)]
// #[serde(untagged)]
// pub enum AssetContent {
//   Buffer(Vec<u8>),
//   String(String),
// }

#[derive(Debug)]
pub struct RenderManifestEntry {
  pub(crate) source: BoxSource,
  filename: String,
  // pathOptions÷: PathData;
  // info?: AssetInfo;
  // pub identifier: String,
  // hash?: string;
  // auxiliary?: boolean;
}

impl RenderManifestEntry {
  pub fn new(source: BoxSource, filename: String) -> Self {
    Self { source, filename }
  }

  pub fn source(&self) -> &BoxSource {
    &self.source
  }

  pub fn filename(&self) -> &str {
    &self.filename
  }
}

pub trait Parser: Debug + Sync + Send {
  fn parse(
    &self,
    module_type: ModuleType,
    args: ParseModuleArgs,
  ) -> Result<TWithDiagnosticArray<BoxModule>>;
}

pub type BoxedParser = Box<dyn Parser>;

#[derive(Debug, Default)]
pub struct ApplyContext {
  pub(crate) registered_parser: HashMap<ModuleType, BoxedParser>,
}

impl ApplyContext {
  pub fn register_parser(&mut self, module_type: ModuleType, parser: BoxedParser) {
    self.registered_parser.insert(module_type, parser);
  }
}
