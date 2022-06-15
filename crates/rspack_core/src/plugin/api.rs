use std::fmt::Debug;

use crate::{
  BoxModule, JobContext, LoadArgs, ParseModuleArgs, PluginContext, RenderManifestArgs, ResolveArgs,
  SourceType,
};

use anyhow::Result;
pub type PluginBuildStartHookOutput = Result<()>;
pub type PluginBuildEndHookOutput = Result<()>;
pub type PluginLoadHookOutput = Result<Option<String>>;
pub type PluginRenderManifestHookOutput = Result<Vec<Asset>>;
pub type PluginParseModuleHookOutput = Result<BoxModule>;
pub type PluginResolveHookOutput = Result<Option<String>>;
// pub type PluginTransformAstHookOutput = Result<ast::Module>;
// pub type PluginParseOutput = Result<RspackAst>;
// pub type PluginGenerateOutput = Result<String>;
// pub type PluginTransformHookOutput = Result<TransformResult>;
// pub type PluginTapGeneratedChunkHookOutput = Result<()>;
// pub type PluginRenderChunkHookOutput = Result<OutputChunk>;

#[async_trait::async_trait]
pub trait Plugin: Debug + Send + Sync {
  fn build_start(&self) -> PluginBuildStartHookOutput {
    Ok(())
  }

  fn build_end(&self) -> PluginBuildEndHookOutput {
    Ok(())
  }

  fn register_parse_module(&self, _ctx: PluginContext) -> Option<Vec<SourceType>> {
    None
  }

  async fn resolve(
    &self,
    _ctx: PluginContext<&mut JobContext>,
    _agrs: ResolveArgs<'_>,
  ) -> PluginResolveHookOutput {
    Ok(None)
  }

  async fn load(
    &self,
    _ctx: PluginContext<&mut JobContext>,
    _args: LoadArgs<'_>,
  ) -> PluginLoadHookOutput {
    Ok(None)
  }

  fn parse_module(
    &self,
    _ctx: PluginContext<&mut JobContext>,
    _args: ParseModuleArgs,
  ) -> PluginParseModuleHookOutput {
    unreachable!()
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
  pub rendered: String,
  pub filename: AssetFilename,
  // pathOptions÷: PathData;
  // info?: AssetInfo;
  // pub identifier: String,
  // hash?: string;
  // auxiliary?: boolean;
}

impl Asset {
  pub fn final_filename(&self) -> String {
    match &self.filename {
      AssetFilename::Static(name) => name.clone(),
      AssetFilename::Templace(_) => todo!("Templace"),
    }
  }
}
