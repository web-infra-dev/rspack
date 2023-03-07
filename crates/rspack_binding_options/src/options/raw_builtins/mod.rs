use napi_derive::napi;
use rspack_core::{Builtins, Define, Minification, PluginExt, PresetEnv};
use rspack_error::internal_error;
use rspack_plugin_copy::CopyPlugin;
use rspack_plugin_css::{plugin::CssConfig, CssPlugin};
use rspack_plugin_dev_friendly_split_chunks::DevFriendlySplitChunksPlugin;
use rspack_plugin_html::HtmlPlugin;
use rspack_plugin_progress::ProgressPlugin;
use serde::Deserialize;

mod raw_copy;
mod raw_css;
mod raw_decorator;
mod raw_html;
mod raw_plugin_import;
mod raw_postcss;
mod raw_progress;
mod raw_react;
mod raw_relay;

pub use raw_css::*;
pub use raw_decorator::*;
pub use raw_html::*;
pub use raw_postcss::*;
pub use raw_progress::*;
pub use raw_react::*;

use self::{
  raw_copy::RawCopyConfig, raw_plugin_import::RawPluginImportConfig, raw_relay::RawRelayConfig,
};
use crate::RawOptionsApply;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
#[napi(object)]
pub struct RawMinification {
  pub passes: u32,
  pub drop_console: bool,
  pub pure_funcs: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
#[napi(object)]
pub struct RawPresetEnv {
  pub targets: Vec<String>,
  #[napi(ts_type = "'usage' | 'entry'")]
  pub mode: Option<String>,
  pub core_js: Option<String>,
}

impl From<RawMinification> for Minification {
  fn from(value: RawMinification) -> Self {
    Self {
      passes: value.passes as usize,
      drop_console: value.drop_console,
      pure_funcs: value.pure_funcs,
    }
  }
}

impl From<RawPresetEnv> for PresetEnv {
  fn from(raw_preset_env: RawPresetEnv) -> Self {
    Self {
      targets: raw_preset_env.targets,
      mode: raw_preset_env.mode.and_then(|mode| match mode.as_str() {
        "usage" => Some(swc_core::ecma::preset_env::Mode::Usage),
        "entry" => Some(swc_core::ecma::preset_env::Mode::Entry),
        _ => None,
      }),
      core_js: raw_preset_env.core_js,
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
  pub minify_options: Option<RawMinification>,
  pub preset_env: Option<RawPresetEnv>,
  #[napi(ts_type = "Record<string, string>")]
  pub define: Define,
  pub tree_shaking: bool,
  pub progress: Option<RawProgressPluginConfig>,
  pub react: RawReactOptions,
  pub decorator: Option<RawDecoratorOptions>,
  pub no_emit_assets: bool,
  pub emotion: Option<String>,
  pub dev_friendly_split_chunks: bool,
  pub copy: Option<RawCopyConfig>,
  pub plugin_import: Option<Vec<RawPluginImportConfig>>,
  pub relay: Option<RawRelayConfig>,
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
        targets: self
          .preset_env
          .as_ref()
          .map(|preset_env| preset_env.targets.clone())
          .unwrap_or_default(),
        postcss: self.postcss.unwrap_or_default().into(),
        modules: css.modules.try_into()?,
      };
      plugins.push(CssPlugin::new(options).boxed());
    }
    if let Some(progress) = self.progress {
      plugins.push(ProgressPlugin::new(progress.into()).boxed());
    }
    if self.dev_friendly_split_chunks {
      plugins.push(DevFriendlySplitChunksPlugin::new().boxed());
    }
    if let Some(copy) = self.copy {
      plugins.push(CopyPlugin::new(copy.patterns.into_iter().map(Into::into).collect()).boxed());
    }

    Ok(Builtins {
      minify_options: self.minify_options.map(Into::into),
      preset_env: self.preset_env.map(Into::into),
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
      plugin_import: self
        .plugin_import
        .map(|plugin_imports| plugin_imports.into_iter().map(Into::into).collect()),
      relay: self.relay.map(Into::into),
    })
  }
}
