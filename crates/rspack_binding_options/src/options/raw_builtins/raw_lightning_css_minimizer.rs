use napi::{bindgen_prelude::Either3, Either};
use napi_derive::napi;
use rspack_error::Result;
use rspack_napi::regexp::{JsRegExp, JsRegExpExt};
use rspack_plugin_lightning_css_minimizer::{
  LightningCssMinimizerOptions, LightningCssMinimizerRule, LightningCssMinimizerRules,
};

type RawLightningCssMinimizerRule = Either<String, JsRegExp>;
type RawLightningCssMinimizerRules = Either3<String, JsRegExp, Vec<RawLightningCssMinimizerRule>>;
struct RawLightningCssMinimizerRuleWrapper(RawLightningCssMinimizerRule);
struct RawLightningCssMinimizerRulesWrapper(RawLightningCssMinimizerRules);

#[derive(Debug)]
#[napi(object)]
pub struct RawLightningCssMinimizerRspackPluginOptions {
  pub error_recovery: bool,
  pub unused_symbols: Vec<String>,
  pub remove_unused_local_idents: bool,
  pub browserslist: Vec<String>,
  #[napi(ts_type = "string | RegExp | (string | RegExp)[]")]
  pub test: Option<RawLightningCssMinimizerRules>,
  #[napi(ts_type = "string | RegExp | (string | RegExp)[]")]
  pub include: Option<RawLightningCssMinimizerRules>,
  #[napi(ts_type = "string | RegExp | (string | RegExp)[]")]
  pub exclude: Option<RawLightningCssMinimizerRules>,
}

fn into_condition(c: Option<RawLightningCssMinimizerRules>) -> Option<LightningCssMinimizerRules> {
  c.map(|test| RawLightningCssMinimizerRulesWrapper(test).into())
}

impl TryFrom<RawLightningCssMinimizerRspackPluginOptions> for LightningCssMinimizerOptions {
  type Error = rspack_error::Error;

  fn try_from(value: RawLightningCssMinimizerRspackPluginOptions) -> Result<Self> {
    Ok(Self {
      error_recovery: value.error_recovery,
      unused_symbols: value.unused_symbols,
      remove_unused_local_idents: value.remove_unused_local_idents,
      browserlist: value.browserslist,
      test: into_condition(value.test),
      include: into_condition(value.include),
      exclude: into_condition(value.exclude),
    })
  }
}

impl From<RawLightningCssMinimizerRuleWrapper> for LightningCssMinimizerRule {
  fn from(x: RawLightningCssMinimizerRuleWrapper) -> Self {
    match x.0 {
      Either::A(v) => Self::String(v),
      Either::B(v) => Self::Regexp(v.to_rspack_regex()),
    }
  }
}

impl From<RawLightningCssMinimizerRulesWrapper> for LightningCssMinimizerRules {
  fn from(value: RawLightningCssMinimizerRulesWrapper) -> Self {
    match value.0 {
      Either3::A(v) => Self::String(v),
      Either3::B(v) => Self::Regexp(v.to_rspack_regex()),
      Either3::C(v) => Self::Array(
        v.into_iter()
          .map(|v| RawLightningCssMinimizerRuleWrapper(v).into())
          .collect(),
      ),
    }
  }
}
