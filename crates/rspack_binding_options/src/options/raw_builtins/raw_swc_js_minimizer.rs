use napi::{bindgen_prelude::Either3, Either};
use napi_derive::napi;
use rspack_error::{miette::IntoDiagnostic, Result};
use rspack_napi::regexp::{JsRegExp, JsRegExpExt};
use rspack_plugin_swc_js_minimizer::{
  ExtractComments, OptionWrapper, SwcJsMinimizerRspackPluginOptions, SwcJsMinimizerRule,
  SwcJsMinimizerRules,
};
use serde::de::DeserializeOwned;
use swc_core::base::BoolOrDataConfig;

type RawSwcJsMinimizerRule = Either<String, JsRegExp>;
type RawSwcJsMinimizerRules = Either3<String, JsRegExp, Vec<RawSwcJsMinimizerRule>>;
struct RawSwcJsMinimizerRuleWrapper(RawSwcJsMinimizerRule);
struct RawSwcJsMinimizerRulesWrapper(RawSwcJsMinimizerRules);

#[derive(Debug)]
#[napi(object)]
pub struct RawExtractComments {
  pub banner: Option<Either<String, bool>>,
  pub condition: Option<String>,
}

#[derive(Debug)]
#[napi(object, object_to_js = false)]
pub struct RawSwcJsMinimizerRspackPluginOptions {
  pub extract_comments: Option<RawExtractComments>,
  pub compress: serde_json::Value,
  pub mangle: serde_json::Value,
  pub format: serde_json::Value,
  pub module: Option<bool>,
  #[napi(ts_type = "string | RegExp | (string | RegExp)[]")]
  pub test: Option<RawSwcJsMinimizerRules>,
  #[napi(ts_type = "string | RegExp | (string | RegExp)[]")]
  pub include: Option<RawSwcJsMinimizerRules>,
  #[napi(ts_type = "string | RegExp | (string | RegExp)[]")]
  pub exclude: Option<RawSwcJsMinimizerRules>,
}

fn try_deserialize_into<T>(value: serde_json::Value) -> Result<T>
where
  T: DeserializeOwned,
{
  serde_json::from_value(value).into_diagnostic()
}

fn into_condition(c: Option<RawSwcJsMinimizerRules>) -> Option<SwcJsMinimizerRules> {
  c.map(|test| RawSwcJsMinimizerRulesWrapper(test).into())
}

fn into_extract_comments(c: Option<RawExtractComments>) -> Option<ExtractComments> {
  let c = c?;
  let condition = c.condition?;
  let banner = match c.banner {
    Some(banner) => match banner {
      Either::A(s) => OptionWrapper::Custom(s),
      Either::B(b) => {
        if b {
          OptionWrapper::Default
        } else {
          OptionWrapper::Disabled
        }
      }
    },
    None => OptionWrapper::Default,
  };

  Some(ExtractComments { condition, banner })
}

impl TryFrom<RawSwcJsMinimizerRspackPluginOptions> for SwcJsMinimizerRspackPluginOptions {
  type Error = rspack_error::Error;

  fn try_from(value: RawSwcJsMinimizerRspackPluginOptions) -> Result<Self> {
    let compress = try_deserialize_into::<
      BoolOrDataConfig<rspack_plugin_swc_js_minimizer::TerserCompressorOptions>,
    >(value.compress)?
    .or(|| BoolOrDataConfig::from_bool(true));
    let mangle = try_deserialize_into::<
      BoolOrDataConfig<rspack_plugin_swc_js_minimizer::MangleOptions>,
    >(value.mangle)?
    .or(|| BoolOrDataConfig::from_bool(true));
    Ok(Self {
      extract_comments: into_extract_comments(value.extract_comments),
      compress,
      mangle,
      format: try_deserialize_into(value.format)?,
      module: value.module,
      test: into_condition(value.test),
      include: into_condition(value.include),
      exclude: into_condition(value.exclude),
      ..Default::default()
    })
  }
}

impl From<RawSwcJsMinimizerRuleWrapper> for SwcJsMinimizerRule {
  fn from(x: RawSwcJsMinimizerRuleWrapper) -> Self {
    match x.0 {
      Either::A(v) => Self::String(v),
      Either::B(v) => Self::Regexp(v.to_rspack_regex()),
    }
  }
}

impl From<RawSwcJsMinimizerRulesWrapper> for SwcJsMinimizerRules {
  fn from(value: RawSwcJsMinimizerRulesWrapper) -> Self {
    match value.0 {
      Either3::A(v) => Self::String(v),
      Either3::B(v) => Self::Regexp(v.to_rspack_regex()),
      Either3::C(v) => Self::Array(
        v.into_iter()
          .map(|v| RawSwcJsMinimizerRuleWrapper(v).into())
          .collect(),
      ),
    }
  }
}
