use napi::Either;
use napi_derive::napi;
use rspack_browserslist::browserslist_to_lightningcss_targets;
use rspack_error::{Result, ToStringResultToRspackResultExt};
use rspack_plugin_lightning_css_minimizer::{
  Browsers, Draft, MinimizerOptions, NonStandard, PluginOptions, PseudoClasses,
};

use crate::asset_condition::{RawAssetConditions, into_asset_conditions};

#[derive(Debug)]
#[napi(object)]
pub struct RawLightningCssMinimizerRspackPluginOptions {
  #[napi(ts_type = "string | RegExp | (string | RegExp)[]")]
  pub test: Option<RawAssetConditions>,
  #[napi(ts_type = "string | RegExp | (string | RegExp)[]")]
  pub include: Option<RawAssetConditions>,
  #[napi(ts_type = "string | RegExp | (string | RegExp)[]")]
  pub exclude: Option<RawAssetConditions>,
  pub remove_unused_local_idents: bool,
  pub minimizer_options: RawLightningCssMinimizerOptions,
}

#[derive(Debug)]
#[napi(object)]
pub struct RawLightningCssMinimizerOptions {
  pub error_recovery: bool,
  #[napi(ts_type = "string[] | RawLightningCssBrowsers")]
  pub targets: Option<Either<Vec<String>, RawLightningCssBrowsers>>,
  pub include: Option<u32>,
  pub exclude: Option<u32>,
  pub drafts: Option<RawDraft>,
  pub non_standard: Option<RawNonStandard>,
  pub pseudo_classes: Option<RawLightningCssPseudoClasses>,
  pub unused_symbols: Vec<String>,
}

#[derive(Debug)]
#[napi(object)]
pub struct RawLightningCssBrowsers {
  pub android: Option<u32>,
  pub chrome: Option<u32>,
  pub edge: Option<u32>,
  pub firefox: Option<u32>,
  pub ie: Option<u32>,
  #[napi(js_name = "ios_saf")]
  pub ios_saf: Option<u32>,
  pub opera: Option<u32>,
  pub safari: Option<u32>,
  pub samsung: Option<u32>,
}

#[derive(Debug)]
#[napi(object)]
pub struct RawDraft {
  pub custom_media: bool,
}

#[derive(Debug)]
#[napi(object)]
pub struct RawNonStandard {
  pub deep_selector_combinator: bool,
}

#[derive(Debug)]
#[napi(object)]
pub struct RawLightningCssPseudoClasses {
  pub hover: Option<String>,
  pub active: Option<String>,
  pub focus: Option<String>,
  pub focus_visible: Option<String>,
  pub focus_within: Option<String>,
}

impl TryFrom<RawLightningCssMinimizerRspackPluginOptions> for PluginOptions {
  type Error = rspack_error::Error;

  fn try_from(value: RawLightningCssMinimizerRspackPluginOptions) -> Result<Self> {
    Ok(Self {
      test: value.test.map(into_asset_conditions),
      include: value.include.map(into_asset_conditions),
      exclude: value.exclude.map(into_asset_conditions),
      remove_unused_local_idents: value.remove_unused_local_idents,
      minimizer_options: MinimizerOptions {
        error_recovery: value.minimizer_options.error_recovery,
        targets: value
          .minimizer_options
          .targets
          .map(|targets| match targets {
            Either::A(query) => browserslist_to_lightningcss_targets(query),
            Either::B(browsers) => Ok(Some(Browsers {
              android: browsers.android,
              chrome: browsers.chrome,
              edge: browsers.edge,
              firefox: browsers.firefox,
              ie: browsers.ie,
              ios_saf: browsers.ios_saf,
              opera: browsers.opera,
              safari: browsers.safari,
              samsung: browsers.samsung,
            })),
          })
          .transpose()
          .to_rspack_result_with_message(|e| format!("Failed to parse browserslist: {e}"))?
          .flatten(),
        include: value.minimizer_options.include,
        exclude: value.minimizer_options.exclude,
        drafts: value.minimizer_options.drafts.map(|d| Draft {
          custom_media: d.custom_media,
        }),
        non_standard: value.minimizer_options.non_standard.map(|n| NonStandard {
          deep_selector_combinator: n.deep_selector_combinator,
        }),
        pseudo_classes: value
          .minimizer_options
          .pseudo_classes
          .map(|p| PseudoClasses {
            hover: p.hover,
            active: p.active,
            focus: p.focus,
            focus_visible: p.focus_visible,
            focus_within: p.focus_within,
          }),
        unused_symbols: value.minimizer_options.unused_symbols,
      },
    })
  }
}
