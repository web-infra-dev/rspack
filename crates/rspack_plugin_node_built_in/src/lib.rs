#![deny(clippy::all)]

use async_trait::async_trait;
use nodejs_resolver::Resolver;
use rspack_core::{LoadArgs, LoadedSource, Loader, Plugin, PluginContext, PluginLoadHookOutput};

#[derive(Debug)]
pub struct NodeBuiltInPlugin;

pub static PLUGIN_NAME: &str = "rspack_node_built_in_plugin";

#[async_trait]
impl Plugin for NodeBuiltInPlugin {
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

  async fn load(&self, ctx: &PluginContext, args: &LoadArgs) -> PluginLoadHookOutput {
    let id = &args.id;
    let result = if ctx.options.platform.eq(&rspack_core::Platform::Node)
      && Resolver::is_build_in_module(id)
    {
      let content = format!(
        r#"var {id} = eval("require('{id}')");

Object.keys({id}).forEach(function(key) {{
  if (key === "default" || key === "__esModule") return;
  if (key in exports && exports[key] === {id}[key]) return;
  Object.defineProperty(exports, key, {{
      enumerable: true,
      get: function() {{
          return {id}[key];
      }}
  }});
}});

export default {id};
      "#
      );

      Some(LoadedSource {
        content: Some(content),
        loader: Some(Loader::Js),
      })
    } else {
      None
    };

    Ok(result)
  }
}
