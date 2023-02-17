use std::str::FromStr;

use napi_derive::napi;
use rspack_core::{Builtins, CompilerOptionsBuilder, Define, Mode, Plugin};
use rspack_plugin_css::plugin::{CssConfig, LocalIdentName, LocalsConvention, PostcssConfig};
use rspack_plugin_progress::{ProgressPlugin, ProgressPluginConfig};

mod raw_copy;
mod raw_css;
mod raw_decorator;
mod raw_emotion;
mod raw_html;
mod raw_postcss;
mod raw_progress;
mod raw_react;

pub use raw_css::*;
pub use raw_decorator::*;
pub use raw_emotion::*;
pub use raw_html::*;
pub use raw_postcss::*;
pub use raw_progress::*;
pub use raw_react::*;
use serde::Deserialize;

use self::raw_copy::RawCopyConfig;
use crate::RawOption;

#[derive(Debug, Deserialize)]
#[napi(object)]
pub struct Minification {
  pub passes: Option<u32>,
  pub enable: Option<bool>,
  pub drop_console: Option<bool>,
  pub pure_funcs: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
#[napi(object)]
pub struct RawBuiltins {
  pub html: Option<Vec<RawHtmlPluginConfig>>,
  pub css: Option<RawCssPluginConfig>,
  pub postcss: Option<RawPostCssConfig>,
  pub minify: Option<Minification>,
  pub polyfill: Option<bool>,
  pub browserslist: Option<Vec<String>>,
  #[napi(ts_type = "Record<string, string>")]
  pub define: Option<Define>,
  pub tree_shaking: Option<bool>,
  pub progress: Option<RawProgressPluginConfig>,
  pub react: Option<RawReactOptions>,
  pub decorator: Option<RawDecoratorOptions>,
  pub no_emit_assets: Option<bool>,
  pub emotion: Option<String>,
  pub dev_friendly_split_chunks: Option<bool>,
  pub copy: Option<RawCopyConfig>,
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
    modules: if let Some(m) = css_config.modules {
      rspack_plugin_css::plugin::ModulesConfig {
        locals_convention: m
          .locals_convention
          .map(|s| LocalsConvention::from_str(&s))
          .transpose()?
          .unwrap_or_default(),
        local_ident_name: m
          .local_ident_name
          .map(|n| LocalIdentName::from_str(&n))
          .transpose()?
          .unwrap_or_else(|| LocalIdentName::with_mode(options.mode)),
        exports_only: m.exports_only.unwrap_or_default(),
      }
    } else {
      rspack_plugin_css::plugin::ModulesConfig {
        local_ident_name: LocalIdentName::with_mode(options.mode),
        locals_convention: LocalsConvention::from_str("asIs").expect("invalid LocalsConvention"),
        exports_only: false,
      }
    },
  })));
  if let Some(progress_config) = builtins.progress.clone() {
    plugins.push(Box::new(ProgressPlugin::new(ProgressPluginConfig {
      prefix: progress_config.prefix,
    })));
  }

  let ret = Builtins {
    browserslist: builtins.browserslist.unwrap_or_default(),
    minify: builtins
      .minify
      .map(Into::into)
      .unwrap_or(rspack_core::Minification {
        enable: matches!(options.mode, Some(Mode::Production)),
        passes: 1,
        drop_console: false,
        pure_funcs: vec![],
      }),
    polyfill: builtins.polyfill.unwrap_or(true),
    define: builtins.define.unwrap_or_default(),
    tree_shaking: builtins.tree_shaking.unwrap_or_default(),
    // side_effects: builtins.side_effects.unwrap_or_default(),
    react: RawOption::raw_to_compiler_option(builtins.react, options)?,
    decorator: transform_to_decorator_options(builtins.decorator),
    no_emit_assets: builtins.no_emit_assets.unwrap_or(false),
    emotion: transform_emotion(builtins.emotion)?,
    dev_friendly_split_chunks: builtins.dev_friendly_split_chunks.unwrap_or(false),
    copy: builtins.copy.map(Into::into),
  };
  Ok(ret)
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

#[allow(clippy::from_over_into)]
/// Reason is the same as above
impl Into<rspack_core::Minification> for Minification {
  fn into(self) -> rspack_core::Minification {
    rspack_core::Minification {
      // this is used for production mode, turn off the minification
      enable: self.enable.unwrap_or(true),
      passes: self.passes.unwrap_or(1) as usize,
      drop_console: self.drop_console.unwrap_or(false),
      pure_funcs: self.pure_funcs.unwrap_or(vec![]),
    }
  }
}
