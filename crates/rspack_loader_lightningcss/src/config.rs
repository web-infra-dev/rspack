use lightningcss::targets::Browsers;
use rspack_browserslist::browserslist_to_lightningcss_targets;
use rspack_cacheable::{
  cacheable,
  with::{AsOption, AsPreset},
};
use rspack_error::ToStringResultToRspackResultExt;
use serde::Deserialize;

#[cacheable]
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Draft {
  pub custom_media: bool,
}

#[cacheable]
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NonStandard {
  pub deep_selector_combinator: bool,
}

#[cacheable]
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PseudoClasses {
  pub hover: Option<String>,
  pub active: Option<String>,
  pub focus: Option<String>,
  pub focus_visible: Option<String>,
  pub focus_within: Option<String>,
}

#[cacheable]
#[derive(Debug, Default)]
pub struct Config {
  pub minify: Option<bool>,
  pub error_recovery: Option<bool>,
  #[cacheable(with=AsOption<AsPreset>)]
  pub targets: Option<Browsers>,
  pub include: Option<u32>,
  pub exclude: Option<u32>,
  pub drafts: Option<Draft>,
  pub non_standard: Option<NonStandard>,
  pub pseudo_classes: Option<PseudoClasses>,
  pub unused_symbols: Option<Vec<String>>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct RawConfig {
  pub minify: Option<bool>,
  pub error_recovery: Option<bool>,
  pub targets: Option<RawConfigTargets>,
  pub include: Option<u32>,
  pub exclude: Option<u32>,
  pub drafts: Option<Draft>,
  pub non_standard: Option<NonStandard>,
  pub pseudo_classes: Option<PseudoClasses>,
  pub unused_symbols: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum RawConfigTargets {
  Browserslist(Vec<String>),
  Targets(Browsers),
}

impl TryFrom<RawConfig> for Config {
  type Error = rspack_error::Error;
  fn try_from(value: RawConfig) -> Result<Self, Self::Error> {
    Ok(Self {
      minify: value.minify,
      error_recovery: value.error_recovery,
      targets: value
        .targets
        .map(|targets| match targets {
          RawConfigTargets::Browserslist(query) => browserslist_to_lightningcss_targets(query),
          RawConfigTargets::Targets(browsers) => Ok(Some(browsers)),
        })
        .transpose()
        .to_rspack_result_with_message(|e| format!("Failed to parse browserslist: {e}"))?
        .flatten(),
      include: value.include,
      exclude: value.exclude,
      drafts: value.drafts,
      non_standard: value.non_standard,
      pseudo_classes: value.pseudo_classes,
      unused_symbols: value.unused_symbols,
    })
  }
}
