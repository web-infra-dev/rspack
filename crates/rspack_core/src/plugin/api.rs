use std::fmt::Debug;

use crate::{
  BoxModule, JobContext, LoadArgs, ParseModuleArgs, PluginContext, RenderManifestArgs, SourceType,
};

use anyhow::Result;
// pub type PluginBuildStartHookOutput = Result<()>;
// pub type PluginBuildEndHookOutput = Result<()>;
pub type PluginLoadHookOutput = Result<Option<String>>;
pub type PluginRenderManifestHookOutput = Result<Vec<Asset>>;
pub type PluginParseModuleHookOutput = Result<BoxModule>;
// pub type PluginTransformAstHookOutput = Result<ast::Module>;
// pub type PluginParseOutput = Result<RspackAst>;
// pub type PluginGenerateOutput = Result<String>;
// pub type PluginTransformHookOutput = Result<TransformResult>;
// pub type PluginTapGeneratedChunkHookOutput = Result<()>;
// pub type PluginRenderChunkHookOutput = Result<OutputChunk>;

pub trait Plugin: Debug + Send + Sync {
  fn register_parse_module(&self, _ctx: PluginContext) -> Option<Vec<SourceType>> {
    None
  }

  fn load(&self, _ctx: PluginContext<&mut JobContext>, _args: LoadArgs) -> PluginLoadHookOutput {
    unreachable!()
  }

  fn parse_module(
    &self,
    _ctx: PluginContext<&mut JobContext>,
    _args: ParseModuleArgs,
  ) -> BoxModule {
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
