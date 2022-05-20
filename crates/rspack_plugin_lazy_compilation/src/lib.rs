use async_trait::async_trait;
use rspack_core::{
  BundleContext, ImportKind, LoadArgs, LoadedSource, Loader, Plugin, PluginLoadHookOutput,
};

#[derive(Debug)]
pub struct LazyCompilationPlugin {}

impl LazyCompilationPlugin {
  pub fn new() -> LazyCompilationPlugin {
    Self {}
  }
}

pub static PLUGIN_NAME: &'static str = "rspack_lazy_compilation_plugin";

#[async_trait]
impl Plugin for LazyCompilationPlugin {
  fn name(&self) -> &'static str {
    PLUGIN_NAME
  }

  async fn load(&self, _ctx: &BundleContext, args: &LoadArgs) -> PluginLoadHookOutput {
    if args.kind == ImportKind::DynamicImport {
      return Some(LoadedSource {
        content: Some("".to_string()),
        loader: Some(Loader::Js),
      });
    }
    None
  }
}
