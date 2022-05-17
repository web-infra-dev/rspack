use std::collections::HashMap;

use async_trait::async_trait;
use rspack_core::{BundleContext, LoadedSource, Loader, Plugin, PluginLoadHookOutput};

#[derive(Debug)]
pub struct MockBuitinsPlugin;

pub static PLUGIN_NAME: &'static str = "rspack_mock_buitins_plugin";

fn is_builtin_module(id: &str) -> bool {
  let builtin_modules = vec![
    "http", "https", "url", "zlib", "stream", "assert", "tty", "util",
  ];
  return builtin_modules.contains(&id);
}
impl MockBuitinsPlugin {
  pub fn new() -> MockBuitinsPlugin {
    Self
  }
}
#[async_trait]
impl Plugin for MockBuitinsPlugin {
  fn name(&self) -> &'static str {
    PLUGIN_NAME
  }

  async fn load(&self, _ctx: &BundleContext, id: &str) -> PluginLoadHookOutput {
    if is_builtin_module(id) {
      Some(LoadedSource {
        loader: Some(Loader::Null),
        content: Some(String::new()),
      })
    } else {
      None
    }
  }
}
