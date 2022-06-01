#![deny(clippy::all)]

mod data_uri;
mod json;
use std::path::Path;

use async_trait::async_trait;
use data_uri::guess_mime_types_ext;
use rspack_core::{BundleContext, Loader, Plugin, PluginTransformHookOutput};

#[derive(Debug)]
pub struct LoaderInterpreterPlugin;

pub static PLUGIN_NAME: &str = "rspack_loader_plugin";

#[async_trait]
impl Plugin for LoaderInterpreterPlugin {
  fn name(&self) -> &'static str {
    PLUGIN_NAME
  }

  fn transform(
    &self,
    _ctx: &BundleContext,
    uri: &str,
    loader: &mut Option<Loader>,
    raw: String,
  ) -> PluginTransformHookOutput {
    if let Some(loader) = loader {
      let result = match loader {
        Loader::DataURI => {
          *loader = Loader::Js;
          let mime_type = match Path::new(uri).extension().and_then(|ext| ext.to_str()) {
            Some(ext) => guess_mime_types_ext(ext),
            None => return Err(anyhow::anyhow!("invalid extension")),
          };
          let format = "base64";
          let data_uri = format!("data:{};{},{}", mime_type, format, base64::encode(&raw));
          format!(
            "var img = \"{data_uri}\";
export default img;",
          )
          .trim()
          .to_string()
        }
        Loader::Json => {
          *loader = Loader::Js;
          format!("export default {raw};")
        }
        Loader::Text => {
          *loader = Loader::Js;
          let data = serde_json::to_string(&raw)?;
          format!("export default {data};")
        }
        Loader::Null => {
          *loader = Loader::Js;
          r#"export default {};"#.to_string()
        }
        _ => raw,
      };
      Ok(result)
    } else {
      Ok(raw)
    }
  }
}
