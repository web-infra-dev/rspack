#[cfg(feature = "node-api")]
use napi_derive::napi;
use rspack_core::{CompilerOptionsBuilder, Plugin};
use rspack_plugin_css::plugin::CssConfig;

mod raw_css;
mod raw_html;

pub use raw_css::*;
pub use raw_html::*;

use crate::RawOption;

use serde::Deserialize;

#[derive(Debug, Deserialize, Default)]
#[cfg(feature = "node-api")]
#[napi(object)]
pub struct RawBuiltins {
  pub html: Option<Vec<RawHtmlPluginConfig>>,
  pub css: Option<RawCssPluginConfig>,
}

#[derive(Debug, Deserialize, Default)]
#[cfg(not(feature = "node-api"))]
pub struct RawBuiltins {
  pub html: Option<Vec<RawHtmlPluginConfig>>,
  pub css: Option<RawCssPluginConfig>,
}

pub(super) fn normalize_builtin(
  builtins: RawBuiltins,
  plugins: &mut Vec<Box<dyn Plugin>>,
  options: &CompilerOptionsBuilder,
) -> Result<(), anyhow::Error> {
  if let Some(configs) = builtins.html {
    for config in configs {
      plugins.push(Box::new(rspack_plugin_html::HtmlPlugin::new(
        config.to_compiler_option(options)?,
      )));
    }
  }

  let css_config = builtins.css.clone().unwrap_or_default();
  plugins.push(Box::new(rspack_plugin_css::CssPlugin::new(CssConfig {
    preset_env: css_config.preset_env,
  })));
  Ok(())
}
