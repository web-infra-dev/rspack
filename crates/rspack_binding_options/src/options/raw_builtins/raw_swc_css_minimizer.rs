use napi::{bindgen_prelude::Either3, Either};
use napi_derive::napi;
use rspack_error::Result;
use rspack_napi::regexp::{JsRegExp, JsRegExpExt};
use rspack_plugin_swc_css_minimizer::{
  SwcCssMinimizerRspackPluginOptions, SwcCssMinimizerRule, SwcCssMinimizerRules,
};

type RawSwcCssMinimizerRule = Either<String, JsRegExp>;
type RawSwcCssMinimizerRules = Either3<String, JsRegExp, Vec<RawSwcCssMinimizerRule>>;
struct RawSwcCssMinimizerRuleWrapper(RawSwcCssMinimizerRule);
struct RawSwcCssMinimizerRulesWrapper(RawSwcCssMinimizerRules);

#[derive(Debug)]
#[napi(object, object_to_js = false)]
pub struct RawSwcCssMinimizerRspackPluginOptions {
  #[napi(ts_type = "string | RegExp | (string | RegExp)[]")]
  pub test: Option<RawSwcCssMinimizerRules>,
  #[napi(ts_type = "string | RegExp | (string | RegExp)[]")]
  pub include: Option<RawSwcCssMinimizerRules>,
  #[napi(ts_type = "string | RegExp | (string | RegExp)[]")]
  pub exclude: Option<RawSwcCssMinimizerRules>,
}

fn into_condition(c: Option<RawSwcCssMinimizerRules>) -> Option<SwcCssMinimizerRules> {
  c.map(|test| RawSwcCssMinimizerRulesWrapper(test).into())
}

impl TryFrom<RawSwcCssMinimizerRspackPluginOptions> for SwcCssMinimizerRspackPluginOptions {
  type Error = rspack_error::Error;

  fn try_from(value: RawSwcCssMinimizerRspackPluginOptions) -> Result<Self> {
    Ok(Self {
      test: into_condition(value.test),
      include: into_condition(value.include),
      exclude: into_condition(value.exclude),
    })
  }
}

impl From<RawSwcCssMinimizerRuleWrapper> for SwcCssMinimizerRule {
  fn from(x: RawSwcCssMinimizerRuleWrapper) -> Self {
    match x.0 {
      Either::A(v) => Self::String(v),
      Either::B(v) => Self::Regexp(v.to_rspack_regex()),
    }
  }
}

impl From<RawSwcCssMinimizerRulesWrapper> for SwcCssMinimizerRules {
  fn from(value: RawSwcCssMinimizerRulesWrapper) -> Self {
    match value.0 {
      Either3::A(v) => Self::String(v),
      Either3::B(v) => Self::Regexp(v.to_rspack_regex()),
      Either3::C(v) => Self::Array(
        v.into_iter()
          .map(|v| RawSwcCssMinimizerRuleWrapper(v).into())
          .collect(),
      ),
    }
  }
}
