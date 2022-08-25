use std::fmt::Debug;

use rspack_loader_runner::{Content, ResourceData};

use crate::{
  BoxModule, FactorizeAndBuildArgs, ModuleType, NormalModuleFactoryContext, ParseModuleArgs,
  PluginContext, ProcessAssetsArgs, RenderManifestArgs, RenderRuntimeArgs, RuntimeSourceNode,
  TransformAst, TransformResult,
};
use rspack_error::Result;

// use anyhow::{Context, Result};
use hashbrown::HashMap;
pub type PluginBuildStartHookOutput = Result<()>;
pub type PluginBuildEndHookOutput = Result<()>;
pub type PluginReadResourceOutput = Result<Option<Content>>;
pub type PluginLoadHookOutput = Result<Option<Content>>;
pub type PluginTransformOutput = Result<TransformResult>;
pub type PluginFactorizeAndBuildHookOutput = Result<Option<(String, BoxModule)>>;
pub type PluginRenderManifestHookOutput = Result<Vec<RenderManifestEntry>>;
pub type PluginRenderRuntimeHookOutput = Result<Vec<RuntimeSourceNode>>;
pub type PluginParseModuleHookOutput = Result<BoxModule>;
pub type PluginParseOutput = Result<TransformAst>;
pub type PluginGenerateOutput = Result<Content>;
pub type PluginProcessAssetsOutput = Result<()>;
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

  fn build_start(&self) -> PluginBuildStartHookOutput {
    Ok(())
  }

  async fn build_end(&self) -> PluginBuildEndHookOutput {
    Ok(())
  }

  // async fn resolve(
  //   &self,
  //   _ctx: PluginContext<&mut NormalModuleFactoryContext>,
  //   _agrs: ResolveArgs<'_>,
  // ) -> PluginResolveHookOutput {
  //   Ok(None)
  // }

  // async fn load(
  //   &self,
  //   _ctx: PluginContext<&mut NormalModuleFactoryContext>,
  //   _args: LoadArgs<'_>,
  // ) -> PluginLoadHookOutput {
  //   Ok(None)
  // }
  // fn reuse_ast(&self) -> bool {
  //   false
  // }
  // fn generate(&self, ast: &Option<TransformAst>) -> PluginGenerateOutput {
  //   let ast = ast.as_ref().context("call generate when ast is empty")?;
  //   match ast {
  //     TransformAst::JavaScript(_ast) => Err(anyhow::anyhow!("js ast codegen not supported yet")),
  //     TransformAst::Css(_ast) => Err(anyhow::anyhow!("css ast codegen not supported yet ")),
  //   }
  // }
  // fn parse(&self, _uri: &str, _content: &Content) -> PluginParseOutput {
  //   unreachable!()
  // }
  // fn transform(
  //   &self,
  //   _ctx: PluginContext<&mut NormalModuleFactoryContext>,
  //   args: TransformArgs,
  // ) -> PluginTransformOutput {
  //   let result = TransformResult {
  //     content: args.content,
  //     ast: args.ast,
  //   };
  //   Ok(result)
  // }
  // fn transform_include(&self, _uri: &str) -> bool {
  //   false
  // }

  async fn read_resource(&self, _resource_data: &ResourceData) -> PluginReadResourceOutput {
    Ok(None)
  }
  /**
   * factorize_and_build hook will generate BoxModule which will be used to generate ModuleGraphModule.
   * It is used to handle the generation of those modules which are not normal, such as External Module
   * It behaves like a BailHook hook.
   * NOTICE: The factorize_and_build hook is a temporary solution and will be replaced with the real factorize hook later
   */
  fn factorize_and_build(
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
    Ok(args.sources.to_vec())
  }
  fn process_assets(
    &self,
    _ctx: PluginContext,
    _args: ProcessAssetsArgs,
  ) -> PluginProcessAssetsOutput {
    Ok(())
  }
}

#[derive(Debug)]
pub enum AssetContent {
  Buffer(Vec<u8>),
  String(String),
}

#[derive(Debug)]
pub struct RenderManifestEntry {
  pub(crate) content: AssetContent,
  filename: String,
  // pathOptions÷: PathData;
  // info?: AssetInfo;
  // pub identifier: String,
  // hash?: string;
  // auxiliary?: boolean;
}

impl RenderManifestEntry {
  pub fn new(content: AssetContent, filename: String) -> Self {
    Self { content, filename }
  }

  pub fn content(&self) -> &AssetContent {
    &self.content
  }

  pub fn filename(&self) -> &str {
    &self.filename
  }
}

pub trait Parser: Debug + Sync + Send {
  fn parse(&self, module_type: ModuleType, args: ParseModuleArgs) -> Result<BoxModule>;
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
