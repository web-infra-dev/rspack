use async_trait::async_trait;
use rspack_core::{BundleContext, LoadedFile, Loader, Plugin, PluginLoadHookOutput};

#[derive(Debug)]
pub struct MockBuiltinsPlugin {}

pub static PLUGIN_NAME: &'static str = "rspack_loader_plugin";

fn is_builtin_module(id: &str) -> bool {
  let builtin_modules = vec![
    "http", "https", "url", "zlib", "stream", "assert", "tty", "util",
  ];
  return builtin_modules.contains(&id);
}

#[async_trait]
impl Plugin for MockBuiltinsPlugin {
  fn name(&self) -> &'static str {
    PLUGIN_NAME
  }

  async fn load(&self, _ctx: &BundleContext, id: &str) -> PluginLoadHookOutput {
    if is_builtin_module(id) {
      Some(LoadedFile::with_loader(String::new(), Loader::Null))
    } else {
      None
    }
  }
}
