#![deny(clippy::all)]

use async_trait::async_trait;
use rspack_core::{
  LoadArgs, LoadedSource, Loader, OnResolveResult, Plugin, PluginContext, PluginLoadHookOutput,
  PluginResolveHookOutput, ResolveArgs,
};

#[derive(Debug)]
pub struct GlobalsPlugin;

pub static PLUGIN_NAME: &str = "rspack_loader_globals";

#[async_trait]
impl Plugin for GlobalsPlugin {
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
  fn need_tap_generated_chunk(&self) -> bool {
    false
  }
  async fn resolve(&self, ctx: &PluginContext, args: &ResolveArgs) -> PluginResolveHookOutput {
    if ctx.options.globals.get(&args.id).is_some() {
      return Ok(Some(OnResolveResult {
        uri: format!("globals:{}", args.id),
        external: false,
      }));
    }

    Ok(None)
  }

  async fn load(&self, ctx: &PluginContext, args: &LoadArgs) -> PluginLoadHookOutput {
    if &args.id[0..8] == "globals:" {
      if let Some(expr_string) = ctx.options.globals.get(&args.id[8..]) {
        let content = format!(
          r#"var global = typeof globalThis !== 'undefined' ? globalThis : global || self;
var imported = global.{};
export default imported;
          "#,
          expr_string
        );

        return Ok(Some(LoadedSource {
          content: Some(content),
          loader: Some(Loader::Js),
        }));
      } else {
        return Ok(None);
      }
    }

    Ok(None)
  }
}
