#[cfg(feature = "node-api")]
use napi_derive::napi;
use rspack_core::{Builtins, CompilerOptionsBuilder, Define, Mode, Plugin};
use rspack_plugin_css::plugin::{CssConfig, PostcssConfig};
use rspack_plugin_progress::{ProgressPlugin, ProgressPluginConfig};

mod raw_css;
mod raw_decorator;
mod raw_html;
mod raw_postcss;
mod raw_progress;
mod raw_react;

use crate::RawOption;
pub use raw_css::*;
pub use raw_decorator::*;
pub use raw_html::*;
pub use raw_postcss::*;
pub use raw_progress::*;
pub use raw_react::*;

use serde::Deserialize;

#[derive(Debug, Deserialize, Default)]
#[cfg(feature = "node-api")]
#[napi(object)]
#[serde(rename_all = "camelCase")]
pub struct RawBuiltins {
  pub html: Option<Vec<RawHtmlPluginConfig>>,
  pub css: Option<RawCssPluginConfig>,
  pub postcss: Option<RawPostCssConfig>,
  pub minify: Option<bool>,
  pub polyfill: Option<bool>,
  pub browserslist: Option<Vec<String>>,
  #[napi(ts_type = "Record<string, string>")]
  pub define: Option<Define>,
  pub tree_shaking: Option<bool>,
  pub side_effects: Option<bool>,
  pub progress: Option<RawProgressPluginConfig>,
  pub react: Option<RawReactOptions>,
  pub decorator: Option<RawDecoratorOptions>,
}

#[derive(Debug, Deserialize, Default)]
#[cfg(not(feature = "node-api"))]
#[serde(rename_all = "camelCase")]
pub struct RawBuiltins {
  pub html: Option<Vec<RawHtmlPluginConfig>>,
  pub css: Option<RawCssPluginConfig>,
  pub postcss: Option<RawPostCssConfig>,
  pub minify: Option<bool>,
  pub polyfill: Option<bool>,
  pub browserslist: Option<Vec<String>>,
  pub define: Option<Define>,
  pub tree_shaking: Option<bool>,
  pub side_effects: Option<bool>,
  pub progress: Option<RawProgressPluginConfig>,
  pub react: Option<RawReactOptions>,
  pub decorator: Option<RawDecoratorOptions>,
}

pub(super) fn normalize_builtin(
  builtins: RawBuiltins,
  plugins: &mut Vec<Box<dyn Plugin>>,
  options: &CompilerOptionsBuilder,
) -> Result<Builtins, anyhow::Error> {
  if let Some(configs) = builtins.html {
    for config in configs {
      plugins.push(Box::new(rspack_plugin_html::HtmlPlugin::new(
        config.to_compiler_option(options)?,
      )));
    }
  }

  let css_config = builtins.css.clone().unwrap_or_default();
  let postcss_config = builtins.postcss.clone().unwrap_or_default();
  plugins.push(Box::new(rspack_plugin_css::CssPlugin::new(CssConfig {
    preset_env: css_config.preset_env.unwrap_or_default(),
    postcss: postcss_config.into(),
    modules: css_config.modules.unwrap_or_default(),
  })));
  if let Some(progress_config) = builtins.progress.clone() {
    plugins.push(Box::new(ProgressPlugin::new(ProgressPluginConfig {
      prefix: progress_config.prefix,
    })));
  }

  Ok(Builtins {
    browserslist: builtins.browserslist.unwrap_or_default(),
    minify: builtins
      .minify
      .unwrap_or(matches!(options.mode, Some(Mode::Production))),
    polyfill: builtins.polyfill.unwrap_or(true),
    define: builtins.define.unwrap_or_default(),
    tree_shaking: builtins.tree_shaking.unwrap_or_default(),
    side_effects: builtins.side_effects.unwrap_or_default(),
    react: RawOption::raw_to_compiler_option(builtins.react, options)?,
    decorator: transform_to_decorator_options(builtins.decorator),
  })
}

#[allow(clippy::from_over_into)]
/// I need to use `Into` instead of `From` because
/// using `From` means I need to import [RawPostCssConfig]
/// in `rspack_plugin_css` which lead a cycle reference
/// `rspack_plugin_css <- rspack_binding_options` <- `rspack_plugin_css`
impl Into<PostcssConfig> for RawPostCssConfig {
  fn into(self) -> PostcssConfig {
    PostcssConfig {
      pxtorem: self.pxtorem.map(|item| item.into()),
    }
  }
}
