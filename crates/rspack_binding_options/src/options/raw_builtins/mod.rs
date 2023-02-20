use napi_derive::napi;
use rspack_core::{Builtins, Define, Minification, PluginExt};
use rspack_error::internal_error;
use rspack_plugin_css::{plugin::CssConfig, CssPlugin};
use rspack_plugin_html::HtmlPlugin;
use rspack_plugin_progress::ProgressPlugin;
use serde::Deserialize;

mod raw_css;
mod raw_decorator;
mod raw_html;
mod raw_postcss;
mod raw_progress;
mod raw_react;

pub use raw_css::*;
pub use raw_decorator::*;
pub use raw_html::*;
pub use raw_postcss::*;
pub use raw_progress::*;
pub use raw_react::*;

use crate::RawOptionsApply;

#[derive(Debug, Deserialize)]
#[napi(object)]
pub struct RawMinification {
  pub passes: u32,
  pub enable: bool,
  pub drop_console: bool,
  pub pure_funcs: Vec<String>,
}

impl From<RawMinification> for Minification {
  fn from(value: RawMinification) -> Self {
    Self {
      enable: value.enable,
      passes: value.passes as usize,
      drop_console: value.drop_console,
      pure_funcs: value.pure_funcs,
    }
  }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
#[napi(object)]
pub struct RawBuiltins {
  pub html: Option<Vec<RawHtmlPluginConfig>>,
  pub css: Option<RawCssPluginConfig>,
  pub postcss: Option<RawPostCssConfig>,
  pub minify: RawMinification,
  pub polyfill: bool,
  pub preset_env: Vec<String>,
  #[napi(ts_type = "Record<string, string>")]
  pub define: Define,
  pub tree_shaking: bool,
  pub progress: Option<RawProgressPluginConfig>,
  pub react: RawReactOptions,
  pub decorator: Option<RawDecoratorOptions>,
  pub no_emit_assets: bool,
  pub emotion: Option<String>,
  pub dev_friendly_split_chunks: bool,
}

impl RawOptionsApply for RawBuiltins {
  type Options = Builtins;

  fn apply(
    self,
    plugins: &mut Vec<rspack_core::BoxPlugin>,
  ) -> Result<Self::Options, rspack_error::Error> {
    if let Some(htmls) = self.html {
      for html in htmls {
        plugins.push(HtmlPlugin::new(html.into()).boxed());
      }
    }
    if let Some(css) = self.css {
      let options = CssConfig {
        preset_env: self.preset_env.clone(),
        postcss: self.postcss.unwrap_or_default().into(),
        modules: css.modules.try_into()?,
      };
      plugins.push(CssPlugin::new(options).boxed());
    }
    if let Some(progress) = self.progress {
      plugins.push(ProgressPlugin::new(progress.into()).boxed());
    }
    Ok(Builtins {
      minify: self.minify.into(),
      polyfill: self.polyfill,
      preset_env: self.preset_env,
      define: self.define,
      tree_shaking: self.tree_shaking,
      react: self.react.into(),
      decorator: self.decorator.map(|i| i.into()),
      no_emit_assets: self.no_emit_assets,
      dev_friendly_split_chunks: self.dev_friendly_split_chunks,
      emotion: self
        .emotion
        .map(|i| serde_json::from_str(&i))
        .transpose()
        .map_err(|e| internal_error!(e.to_string()))?,
    })
  }
}
