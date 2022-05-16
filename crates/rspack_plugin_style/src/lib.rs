use std::path::Path;

use async_trait::async_trait;
use rspack_core::{
  BundleContext, LoadedSource, Loader, Plugin, PluginLoadHookOutput, PluginTransformRawHookOutput,
};

pub static PLUGIN_NAME: &'static str = "rspack_loader_plugin";

#[derive(Debug)]
pub struct StyleLoaderPlugin {}

#[async_trait]
impl Plugin for StyleLoaderPlugin {
  fn name(&self) -> &'static str {
    PLUGIN_NAME
  }

  fn transform_raw(
    &self,
    _ctx: &BundleContext,
    _uri: &str,
    loader: &mut Loader,
    raw: String,
  ) -> PluginTransformRawHookOutput {
    if let Loader::Css = loader {
      format!(
        "
        if (typeof document !== 'undefined') {{
          var style = document.createElement('style');
          var node = document.createTextNode(`{}`);
          style.appendChild(node);
          document.head.appendChild(style);
        }}
      ",
        raw
      )
    } else {
      raw
    }
  }
}
