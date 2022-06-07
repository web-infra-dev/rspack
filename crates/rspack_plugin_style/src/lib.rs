#![deny(clippy::all)]

use async_trait::async_trait;
use rspack_core::{Loader, Plugin, PluginContext, PluginTransformHookOutput, TransformArgs};

pub static PLUGIN_NAME: &str = "rspack_loader_plugin";

#[derive(Debug)]
pub struct StyleLoaderPlugin {}

#[async_trait]
impl Plugin for StyleLoaderPlugin {
  fn name(&self) -> &'static str {
    PLUGIN_NAME
  }

  #[inline]
  fn need_build_start(&self) -> bool {
    false
  }

  #[inline]
  fn need_build_end(&self) -> bool {
    false
  }

  #[inline]
  fn need_resolve(&self) -> bool {
    false
  }

  #[inline]
  fn need_load(&self) -> bool {
    false
  }

  #[inline]
  fn need_transform_ast(&self) -> bool {
    false
  }

  #[inline]
  fn need_tap_generated_chunk(&self) -> bool {
    false
  }
  fn transform(&self, _ctx: &PluginContext, args: TransformArgs) -> PluginTransformHookOutput {
    let TransformArgs {
      code: raw, loader, ..
    } = args;
    if let Some(Loader::Css) = loader {
      *loader = Some(Loader::Js);
      Ok(
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
        .into(),
      )
    } else {
      Ok(raw.into())
    }
  }
}
