use napi::{bindgen_prelude::Either3, Either};
use napi_derive::napi;
use rspack_error::{miette::IntoDiagnostic, Result};
use rspack_napi_shared::{JsRegExp, JsRegExpExt};
use rspack_plugin_swc_js_minimizer::{
  ExtractComments, OptionWrapper, SwcJsMinimizerRspackPluginOptions, SwcJsMinimizerRule,
  SwcJsMinimizerRules,
};
use serde::Deserialize;
use swc_config::config_types::BoolOrDataConfig;

type RawSwcJsMinimizerRule = Either<String, JsRegExp>;
type RawSwcJsMinimizerRules = Either3<String, JsRegExp, Vec<RawSwcJsMinimizerRule>>;
struct RawSwcJsMinimizerRuleWrapper(RawSwcJsMinimizerRule);
struct RawSwcJsMinimizerRulesWrapper(RawSwcJsMinimizerRules);

#[derive(Debug)]
#[napi(object)]
pub struct RawExtractComments {
  pub banner: Option<Either<String, bool>>,
  pub condition: Option<String>,
  pub filename: Option<String>,
}

#[derive(Debug)]
#[napi(object)]
pub struct RawSwcJsMinimizerRspackPluginOptions {
  pub extract_comments: Option<RawExtractComments>,
  pub compress: Either<bool, String>,
  pub mangle: Either<bool, String>,
  pub format: String,
  pub module: Option<bool>,
  #[napi(ts_type = "string | RegExp | (string | RegExp)[]")]
  pub test: Option<RawSwcJsMinimizerRules>,
  #[napi(ts_type = "string | RegExp | (string | RegExp)[]")]
  pub include: Option<RawSwcJsMinimizerRules>,
  #[napi(ts_type = "string | RegExp | (string | RegExp)[]")]
  pub exclude: Option<RawSwcJsMinimizerRules>,
}

fn try_deserialize_into<'de, T: 'de + Deserialize<'de>>(
  value: &'de Either<bool, String>,
) -> Result<BoolOrDataConfig<T>> {
  Ok(match value {
    Either::A(b) => BoolOrDataConfig::from_bool(*b),
    Either::B(s) => BoolOrDataConfig::from_obj(serde_json::from_str(s).into_diagnostic()?),
  })
}

fn into_condition(c: Option<RawSwcJsMinimizerRules>) -> Option<SwcJsMinimizerRules> {
  c.map(|test| RawSwcJsMinimizerRulesWrapper(test).into())
}

fn into_extract_comments(c: Option<RawExtractComments>) -> Option<ExtractComments> {
  let c = c?;
  let condition = c.condition?;
  let filename = c.filename;
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

  Some(ExtractComments {
    condition,
    banner,
    filename,
  })
}

impl TryFrom<RawSwcJsMinimizerRspackPluginOptions> for SwcJsMinimizerRspackPluginOptions {
  type Error = rspack_error::Error;

  fn try_from(value: RawSwcJsMinimizerRspackPluginOptions) -> Result<Self> {
    Ok(Self {
      extract_comments: into_extract_comments(value.extract_comments),
      compress: try_deserialize_into(&value.compress)?,
      mangle: try_deserialize_into(&value.mangle)?,
      format: serde_json::from_str(&value.format).into_diagnostic()?,
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
