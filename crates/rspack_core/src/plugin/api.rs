use std::fmt::Debug;

use crate::{
  BoxModule, LoadArgs, ModuleType, NormalModuleFactoryContext, ParseModuleArgs, PluginContext,
  RenderManifestArgs, RenderRuntimeArgs, ResolveArgs, RuntimeSourceNode, TransformAst,
  TransformResult,
};
use crate::{Content, TransformArgs};

use anyhow::Context;
use anyhow::Result;
use hashbrown::HashMap;
pub type PluginBuildStartHookOutput = Result<()>;
pub type PluginBuildEndHookOutput = Result<()>;
pub type PluginLoadHookOutput = Result<Option<Content>>;
pub type PluginTransformOutput = Result<TransformResult>;
pub type PluginRenderManifestHookOutput = Result<Vec<Asset>>;
pub type PluginRenderRuntimeHookOutput = Result<Vec<RuntimeSourceNode>>;
pub type PluginParseModuleHookOutput = Result<BoxModule>;
pub type PluginResolveHookOutput = Result<Option<String>>;
pub type PluginParseOutput = Result<TransformAst>;
pub type PluginGenerateOutput = Result<Content>;
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
  fn generate(&self, ast: &Option<TransformAst>) -> PluginGenerateOutput {
    let ast = ast.as_ref().context("call generate when ast is empty")?;
    match ast {
      TransformAst::JavaScript(_ast) => Err(anyhow::anyhow!("js ast codegen not supported yet")),
      TransformAst::Css(_ast) => Err(anyhow::anyhow!("css ast codegen not supported yet ")),
    }
  }
  fn parse(&self, _uri: &str, _content: &Content) -> PluginParseOutput {
    unreachable!()
  }
  fn transform(
    &self,
    _ctx: PluginContext<&mut NormalModuleFactoryContext>,
    args: TransformArgs,
  ) -> PluginTransformOutput {
    let result = TransformResult {
      content: args.content,
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
  fn render_runtime(
    &self,
    _ctx: PluginContext,
    _args: RenderRuntimeArgs,
  ) -> PluginRenderRuntimeHookOutput {
    Ok(vec![])
  }
}

pub trait Filename: Debug + Sync + Send {
  // TODO more params, e.g. hash, name, etc.
  fn filename(&self, filename: String, ext: String) -> String;
}

#[derive(Debug)]
pub struct OutputFilename {
  template: String,
}

impl OutputFilename {
  pub fn new(template: String) -> Self {
    Self { template }
  }
}

impl Filename for OutputFilename {
  fn filename(&self, filename: String, ext: String) -> String {
    // TODO add more
    self
      .template
      .replace("[name]", &filename)
      .replace("[ext]", &ext)
  }
}

#[derive(Debug)]
pub struct OutputAssetModuleFilename {
  template: String,
}

impl Default for OutputAssetModuleFilename {
  fn default() -> Self {
    Self::new("assets/[name][ext]".to_owned())
  }
}

impl OutputAssetModuleFilename {
  pub fn new(template: String) -> Self {
    Self { template }
  }
}

impl Filename for OutputAssetModuleFilename {
  // TODO add more
  fn filename(&self, filename: String, ext: String) -> String {
    self
      .template
      .replace("[name]", &filename)
      .replace("[ext]", &ext)
  }
}

#[derive(Debug)]
pub enum AssetContent {
  Buffer(Vec<u8>),
  String(String),
}

#[derive(Debug)]
pub struct Asset {
  content: AssetContent,
  filename: String,
  // pathOptions÷: PathData;
  // info?: AssetInfo;
  // pub identifier: String,
  // hash?: string;
  // auxiliary?: boolean;
}

impl Asset {
  pub fn new(content: AssetContent, filename: String) -> Self {
    Self { content, filename }
  }

  pub fn content(&self) -> &AssetContent {
    &self.content
  }

  pub fn filename(&self) -> &String {
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
