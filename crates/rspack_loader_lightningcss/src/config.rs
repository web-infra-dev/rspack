use lightningcss::targets::Browsers;
use rspack_cacheable::{
  cacheable,
  with::{AsOption, AsPreset},
};
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
  pub draft: Option<Draft>,
  pub non_standard: Option<NonStandard>,
  pub pseudo_classes: Option<PseudoClasses>,
  pub unused_symbols: Option<Vec<String>>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct RawConfig {
  pub minify: Option<bool>,
  pub error_recovery: Option<bool>,
  pub targets: Option<Vec<String>>,
  pub include: Option<u32>,
  pub exclude: Option<u32>,
  // TODO: deprecate `draft` in favor of `drafts`
  pub draft: Option<Draft>,
  pub drafts: Option<Draft>,
  pub non_standard: Option<NonStandard>,
  pub pseudo_classes: Option<PseudoClasses>,
  pub unused_symbols: Option<Vec<String>>,
}

impl TryFrom<RawConfig> for Config {
  type Error = rspack_error::miette::Report;
  fn try_from(value: RawConfig) -> Result<Self, Self::Error> {
    Ok(Self {
      minify: value.minify,
      error_recovery: value.error_recovery,
      targets: value
        .targets
        .map(lightningcss::targets::Browsers::from_browserslist)
        .transpose()
        .map_err(|err| rspack_error::error!("Failed to parse browserslist: {}", err))?
        .flatten(),
      include: value.include,
      exclude: value.exclude,
      // We should use `drafts` if it is present, otherwise use `draft`
      draft: value.drafts.or(value.draft),
      non_standard: value.non_standard,
      pseudo_classes: value.pseudo_classes,
      unused_symbols: value.unused_symbols,
    })
  }
}
