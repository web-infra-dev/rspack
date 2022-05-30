#![deny(clippy::all)]

use async_trait::async_trait;
use rspack_core::{
  BundleContext, ImportKind, LoadArgs, LoadedSource, Loader, OnResolveResult, Plugin,
  PluginLoadHookOutput, PluginResolveHookOutput, ResolveArgs,
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

  async fn resolve(&self, _ctx: &BundleContext, args: &ResolveArgs) -> PluginResolveHookOutput {
    if args.kind == ImportKind::DynamicImport {
      return Some(OnResolveResult {
        source: Some(LoadedSource {
          content: "".to_string(),
          loader: Some(Loader::Js),
        }),
        uri: args.id.clone(),
        ..Default::default()
      });
    }
    None
  }
}
