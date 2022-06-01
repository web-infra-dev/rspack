#![deny(clippy::all)]

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

  #[inline]
  fn need_build_start(&self) -> bool {
    false
  }

  #[inline]
  fn need_build_end(&self) -> bool {
    false
  }

  #[inline]
  fn need_resolve(&self) -> bool {
    false
  }

  #[inline]
  fn need_transform(&self) -> bool {
    false
  }

  #[inline]
  fn need_transform_ast(&self) -> bool {
    false
  }

  #[inline]
  fn need_tap_generated_chunk(&self) -> bool {
    false
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
