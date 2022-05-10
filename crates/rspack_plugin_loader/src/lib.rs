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

#[async_trait]
impl Plugin for LoaderPlugin {
  fn name(&self) -> &'static str {
    "rspack_loader_plugin"
  }

  async fn load(&self, _ctx: &BundleContext, id: &str) -> PluginLoadHookOutput {
    let loader_and_ext = Path::new(id).extension().and_then(|ext| {
      ext
        .to_str()
        .and_then(|ext| (self.options.get(ext).map(|loader| (loader, ext))))
    });
    if let Some((loader, ext)) = loader_and_ext {
      match loader {
        Loader::DataURI => {
          let mime_type = guess_mime_types_ext(ext);
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
      }
    } else {
      None
    }
  }
}
