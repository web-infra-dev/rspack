#[deny(clippy::all)]
use anyhow::Result;
use async_trait::async_trait;
use rspack_core::{
  BundleContext, LoadArgs, LoadedSource, Loader, OnResolveResult, Plugin, PluginLoadHookOutput,
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

  async fn resolve(&self, ctx: &BundleContext, args: &ResolveArgs) -> PluginResolveHookOutput {
    if ctx.options.globals.get(&args.id).is_some() {
      return Ok(Some(OnResolveResult {
        uri: format!("globals:{}", args.id),
        external: false,
      }));
    }

    Ok(None)
  }

  async fn load(&self, ctx: &BundleContext, args: &LoadArgs) -> PluginLoadHookOutput {
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
