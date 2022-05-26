#![deny(clippy::all)]

use async_trait::async_trait;
use rspack_core::{BundleContext, LoadArgs, LoadedSource, Loader, Plugin, PluginLoadHookOutput};

#[derive(Debug)]
pub struct MockBuitinsPlugin;

pub static PLUGIN_NAME: &str = "rspack_mock_buitins_plugin";

fn is_builtin_module(id: &str) -> bool {
  let builtin_modules = vec![
    "http", "https", "url", "zlib", "stream", "assert", "tty", "util", "os",
  ];
  builtin_modules.contains(&id)
}
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

  async fn load(&self, _ctx: &BundleContext, args: &LoadArgs) -> PluginLoadHookOutput {
    if is_builtin_module(&args.id) {
      Some(LoadedSource {
        loader: Some(Loader::Js),
        content: Some(
          r#"var p=new Proxy(function(){},{get(){return p},set(){return!0},apply(){return p},constructor(){return p},defineProperty(){return!0},deleteProperty(){return!0},getOwnPropertyDescriptor(){return{value:p,writable:!0,enumerable:!0,configurable:!0,get(){return p},set(){return!0},}},getPrototypeOf(){return p},setPrototypeOf(){return!0},has(){return!0},isExtensible(){return!0},ownKeys(){return[]},preventExtensions(){return!0},});
export default p;"#.to_owned(),
        ),
      })
    } else {
      None
    }
  }
}
