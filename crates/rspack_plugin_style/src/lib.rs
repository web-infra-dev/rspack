use async_trait::async_trait;
use rspack_core::{BundleContext, Loader, Plugin, PluginTransformHookOutput};

pub static PLUGIN_NAME: &str = "rspack_loader_plugin";

#[derive(Debug)]
pub struct StyleLoaderPlugin {}

#[async_trait]
impl Plugin for StyleLoaderPlugin {
  fn name(&self) -> &'static str {
    PLUGIN_NAME
  }

  fn transform(
    &self,
    _ctx: &BundleContext,
    _uri: &str,
    loader: &mut Option<Loader>,
    raw: String,
  ) -> PluginTransformHookOutput {
    if let Some(Loader::Css) = loader {
      *loader = Some(Loader::Js);
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
