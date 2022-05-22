use std::collections::HashMap;

use async_trait::async_trait;
use rspack_core::{BundleContext, LoadArgs, LoadedSource, Loader, Plugin, PluginLoadHookOutput};

#[derive(Debug)]
pub struct NodeEmulationPlugin;

pub static PLUGIN_NAME: &'static str = "rspack_node_emulation_plugin";

fn is_builtin_module(id: &str) -> bool {
  let builtin_modules = vec![
    "http", "https", "url", "zlib", "stream", "assert", "tty", "util",
  ];
  return builtin_modules.contains(&id);
}
impl NodeEmulationPlugin {
  pub fn new() -> Self {
    Self
  }
}
#[async_trait]
impl Plugin for NodeEmulationPlugin {
  fn name(&self) -> &'static str {
    PLUGIN_NAME
  }

  async fn load(&self, _ctx: &BundleContext, args: &LoadArgs) -> PluginLoadHookOutput {
    if is_builtin_module(&args.id) {
      Some(LoadedSource {
        loader: Some(Loader::Null),
        content: Some(String::new()),
      })
    } else {
      None
    }
  }
}
