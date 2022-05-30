#![deny(clippy::all)]

use anyhow::Result;
use async_trait::async_trait;
use rspack_core::{
  BundleContext, ImportKind, LoadArgs, LoadedSource, Loader, Plugin, PluginLoadHookOutput,
};

#[derive(Debug)]
pub struct LazyCompilationPlugin {}

impl LazyCompilationPlugin {
  pub fn new() -> Self {
    Self {}
  }
}

impl Default for LazyCompilationPlugin {
  fn default() -> Self {
    Self::new()
  }
}

pub static PLUGIN_NAME: &str = "rspack_lazy_compilation_plugin";

#[async_trait]
impl Plugin for LazyCompilationPlugin {
  fn name(&self) -> &'static str {
    PLUGIN_NAME
  }

  async fn load(&self, _ctx: &BundleContext, args: &LoadArgs) -> PluginLoadHookOutput {
    if args.kind == ImportKind::DynamicImport {
      return Ok(Some(LoadedSource {
        content: Some("".to_string()),
        loader: Some(Loader::Js),
      }));
    }
    Ok(None)
  }
}
