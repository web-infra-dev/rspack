mod data_uri;
mod json;
use std::{collections::HashMap, path::Path};

use async_trait::async_trait;
use data_uri::guess_mime_types_ext;
use rspack_core::{BundleContext, Loader, Plugin, PluginLoadHookOutput};

#[derive(Debug)]
pub struct LoaderPlugin {
  pub options: HashMap<String, Loader>,
}

pub static PLUGIN_NAME: &'static str = "rspack_loader_plugin";

fn is_builtin_module(id: &str) -> bool {
  let builtin_modules = vec![
    "http", "https", "url", "zlib", "stream", "assert", "tty", "util",
  ];
  return builtin_modules.contains(&id);
}
#[async_trait]
impl Plugin for LoaderPlugin {
  fn name(&self) -> &'static str {
    PLUGIN_NAME
  }

  async fn load(&self, _ctx: &BundleContext, id: &str) -> PluginLoadHookOutput {
    let (loader, ext) = if is_builtin_module(id) {
      (Some(Loader::Empty), None)
    } else {
      let ext = Path::new(id).extension().and_then(|ext| ext.to_str())?;
      let loader = self.options.get(ext)?;
      (Some(*loader), Some(ext))
    };
    let loader = loader?;
    match loader {
      Loader::DataURI => {
        let mime_type = guess_mime_types_ext(ext.unwrap());
        let format = "base64";
        let data = std::fs::read(id).ok()?;
        let data_uri = format!("data:{};{},{}", mime_type, format, base64::encode(&data));
        Some(
          format!(
            "
          var img = \"{}\";
          export default img;
          ",
            data_uri
          )
          .trim()
          .to_string(),
        )
      }
      Loader::Json => {
        let data = std::fs::read_to_string(id).ok()?;
        Some(format!(
          "
          export default {}
          ",
          data
        ))
      }
      Loader::Text => {
        let data = std::fs::read_to_string(id).ok()?;
        let data = serde_json::to_string(&data).ok()?;
        Some(format!(
          "
          export default {}
          ",
          data
        ))
      }
      Loader::Empty => Some(
        r#"
        export default {}
        "#
        .to_string(),
      ),
    }
  }
}
