use std::fmt::Debug;

use crate::TransformArgs;
use crate::{
  BoxModule, LoadArgs, ModuleType, NormalModuleFactoryContext, ParseModuleArgs, PluginContext,
  RenderManifestArgs, ResolveArgs, RspackAst, TransformResult,
};

use anyhow::Context;
use anyhow::Result;
use hashbrown::HashMap;
pub type PluginBuildStartHookOutput = Result<()>;
pub type PluginBuildEndHookOutput = Result<()>;
pub type PluginLoadHookOutput = Result<Option<String>>;
pub type PluginTransformOutput = Result<TransformResult>;
pub type PluginRenderManifestHookOutput = Result<Vec<Asset>>;
pub type PluginParseModuleHookOutput = Result<BoxModule>;
pub type PluginResolveHookOutput = Result<Option<String>>;
pub type PluginParseOutput = Result<RspackAst>;
pub type PluginGenerateOutput = Result<String>;
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

  fn build_end(&self) -> PluginBuildEndHookOutput {
    Ok(())
  }

  async fn resolve(
    &self,
    _ctx: PluginContext<&mut NormalModuleFactoryContext>,
    _agrs: ResolveArgs<'_>,
  ) -> PluginResolveHookOutput {
    Ok(None)
  }

  async fn load(
    &self,
    _ctx: PluginContext<&mut NormalModuleFactoryContext>,
    _args: LoadArgs<'_>,
  ) -> PluginLoadHookOutput {
    Ok(None)
  }
  fn reuse_ast(&self) -> bool {
    false
  }
  fn generate(&self, ast: &Option<RspackAst>) -> PluginGenerateOutput {
    let ast = ast.as_ref().context("call generate when ast is empty")?;
    match ast {
      RspackAst::JavaScript(_ast) => Err(anyhow::anyhow!("js ast codegen not supported yet")),
      RspackAst::Css(_ast) => Err(anyhow::anyhow!("css ast codegen not supported yet ")),
    }
  }
  fn parse(&self, _uri: &str, _code: &str) -> PluginParseOutput {
    anyhow::bail!("not implemented")
  }
  fn transform(
    &self,
    _ctx: PluginContext<&mut NormalModuleFactoryContext>,
    args: TransformArgs,
  ) -> PluginTransformOutput {
    let result = TransformResult {
      code: args.code,
      ast: args.ast,
    };
    Ok(result)
  }
  fn transform_include(&self, _uri: &str) -> bool {
    false
  }
  fn render_manifest(
    &self,
    _ctx: PluginContext,
    _args: RenderManifestArgs,
  ) -> PluginRenderManifestHookOutput {
    Ok(vec![])
  }
}

#[derive(Debug)]
pub enum AssetFilename {
  Static(String),
  Templace(String),
}

#[derive(Debug)]
pub struct Asset {
  rendered: String,
  filename: AssetFilename,
  // pathOptions÷: PathData;
  // info?: AssetInfo;
  // pub identifier: String,
  // hash?: string;
  // auxiliary?: boolean;
}

impl Asset {
  pub fn new(rendered: String, filename: AssetFilename) -> Self {
    Self { rendered, filename }
  }

  pub fn source(&self) -> &str {
    self.rendered.as_str()
  }
}

impl Asset {
  pub fn final_filename(&self) -> String {
    match &self.filename {
      AssetFilename::Static(name) => name.clone(),
      AssetFilename::Templace(_) => todo!("Templace"),
    }
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
