use std::path::Path;

use async_trait::async_trait;
use rspack_core::{BundleContext, Plugin, PluginLoadHookOutput};

#[derive(Debug)]
pub struct StyleLoaderPlugin {}

pub static PLUGIN_NAME: &'static str = "rspack_style_plugin";

#[async_trait]
impl Plugin for StyleLoaderPlugin {
  fn name(&self) -> &'static str {
    PLUGIN_NAME
  }

  async fn load(&self, _ctx: &BundleContext, id: &str) -> PluginLoadHookOutput {
    let path = Path::new(id);
    let ext = path.extension().and_then(|ext| ext.to_str())?;
    match ext {
      "css" => {
        let content = std::fs::read_to_string(path).ok()?;
        Some(format!(
          "
          if (typeof document !== 'undefined') {{
            var style = document.createElement('style');
            var node = document.createTextNode(`{}`);
            style.appendChild(node);
            document.head.appendChild(style);
          }}
        ",
          content
        ))
      }
      _ => None,
    }
  }
}
