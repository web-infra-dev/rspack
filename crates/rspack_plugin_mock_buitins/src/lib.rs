#![deny(clippy::all)]

use async_trait::async_trait;
use rspack_core::Plugin;

#[derive(Debug)]
pub struct MockBuitinsPlugin;

pub static PLUGIN_NAME: &str = "rspack_mock_buitins_plugin";

// fn is_builtin_module(id: &str) -> bool {
//   let builtin_modules = vec![
//     "http", "https", "url", "zlib", "stream", "assert", "tty", "util", "os",
//   ];
//   builtin_modules.contains(&id)
// }
impl MockBuitinsPlugin {
  pub fn new() -> MockBuitinsPlugin {
    Self
  }
}

impl Default for MockBuitinsPlugin {
  fn default() -> Self {
    Self::new()
  }
}

#[async_trait]
impl Plugin for MockBuitinsPlugin {
  fn name(&self) -> &'static str {
    PLUGIN_NAME
  }

  // async fn load(&self, _ctx: &BundleContext, args: &LoadArgs) -> PluginLoadHookOutput {
  //   if is_builtin_module(&args.id) {
  //     Some(LoadedSource {
  //       loader: Some(Loader::Null),
  //       content: Some(String::new()),
  //     })
  //   } else {
  //     None
  //   }
  // }
}
