use napi_derive::napi;
use rspack_binding_values::{into_asset_conditions, RawAssetConditions};
use rspack_error::Result;
use rspack_plugin_lightning_css_minimizer::{
  Draft, MinimizerOptions, NonStandard, PluginOptions, PseudoClasses,
};

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
  pub targets: Option<Vec<String>>,
  pub include: Option<u32>,
  pub exclude: Option<u32>,
  // TODO: deprecate `draft` in favor of `drafts`
  pub draft: Option<RawDraft>,
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
          .map(|t| {
            rspack_loader_lightningcss::lightningcss::targets::Browsers::from_browserslist(t)
          })
          .transpose()
          .map_err(|e| rspack_error::error!("Failed to parse browserslist: {}", e))?
          .flatten(),
        include: value.minimizer_options.include,
        exclude: value.minimizer_options.exclude,
        // We should use `drafts` if it is present, otherwise use `draft`
        draft: value
          .minimizer_options
          .drafts
          .or(value.minimizer_options.draft)
          .map(|d| Draft {
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
