use std::fmt::Debug;

use crate::{
  BoxModule, JobContext, LoadArgs, ParseModuleArgs, PluginContext, RenderManifestArgs, SourceType,
};

pub trait Plugin: Debug + Send + Sync {
  fn register_parse_module(&self, _ctx: PluginContext) -> Option<Vec<SourceType>> {
    None
  }

  fn load(&self, _ctx: PluginContext<&mut JobContext>, _args: LoadArgs) -> Option<String> {
    unreachable!()
  }

  fn parse_module(
    &self,
    _ctx: PluginContext<&mut JobContext>,
    _args: ParseModuleArgs,
  ) -> BoxModule {
    unreachable!()
  }

  fn render_manifest(&self, _ctx: PluginContext, _args: RenderManifestArgs) -> Vec<Asset> {
    vec![]
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
