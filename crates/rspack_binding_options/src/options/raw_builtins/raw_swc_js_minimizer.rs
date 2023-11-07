use napi::Either;
use napi_derive::napi;
use rspack_error::{internal_error, Result};
use rspack_plugin_swc_js_minimizer::{
  SwcJsMinimizerRspackPluginOptions, SwcJsMinimizerRule, SwcJsMinimizerRules,
};
use serde::Deserialize;
use swc_config::config_types::BoolOrDataConfig;

#[derive(Debug)]
#[napi(object)]
pub struct RawSwcJsMinimizerRule {
  #[napi(ts_type = r#""string" | "regexp""#)]
  pub r#type: String,
  pub string_matcher: Option<String>,
  pub regexp_matcher: Option<String>,
}

#[derive(Debug)]
#[napi(object)]
pub struct RawSwcJsMinimizerRules {
  #[napi(ts_type = r#""string" | "regexp" | "array""#)]
  pub r#type: String,
  pub string_matcher: Option<String>,
  pub regexp_matcher: Option<String>,
  pub array_matcher: Option<Vec<RawSwcJsMinimizerRule>>,
}

#[derive(Debug)]
#[napi(object)]
pub struct RawSwcJsMinimizerRspackPluginOptions {
  pub extract_comments: Option<String>,
  pub compress: Either<bool, String>,
  pub mangle: Either<bool, String>,
  pub format: String,
  pub test: Option<RawSwcJsMinimizerRules>,
  pub include: Option<RawSwcJsMinimizerRules>,
  pub exclude: Option<RawSwcJsMinimizerRules>,
}

fn try_deserialize_into<'de, T: 'de + Deserialize<'de>>(
  value: &'de Either<bool, String>,
) -> Result<BoolOrDataConfig<T>> {
  Ok(match value {
    Either::A(b) => BoolOrDataConfig::from_bool(*b),
    Either::B(s) => BoolOrDataConfig::from_obj(serde_json::from_str(s)?),
  })
}

impl TryFrom<RawSwcJsMinimizerRspackPluginOptions> for SwcJsMinimizerRspackPluginOptions {
  type Error = rspack_error::Error;

  fn try_from(value: RawSwcJsMinimizerRspackPluginOptions) -> Result<Self> {
    fn try_condition(
      raw_condition: Option<RawSwcJsMinimizerRules>,
    ) -> Result<Option<SwcJsMinimizerRules>> {
      let condition: Option<SwcJsMinimizerRules> = if let Some(test) = raw_condition {
        Some(test.try_into()?)
      } else {
        None
      };

      Ok(condition)
    }

    Ok(Self {
      extract_comments: value.extract_comments,
      compress: try_deserialize_into(&value.compress)?,
      mangle: try_deserialize_into(&value.mangle)?,
      format: serde_json::from_str(&value.format)?,
      test: try_condition(value.test)?,
      include: try_condition(value.include)?,
      exclude: try_condition(value.exclude)?,
      ..Default::default()
    })
  }
}

impl TryFrom<RawSwcJsMinimizerRule> for SwcJsMinimizerRule {
  type Error = rspack_error::Error;

  fn try_from(x: RawSwcJsMinimizerRule) -> Result<Self> {
    let result = match x.r#type.as_str() {
      "string" => Self::String(x.string_matcher.ok_or_else(|| {
        internal_error!(
          "should have a string_matcher when MinificationConditions.type is \"string\""
        )
      })?),
      "regexp" => Self::Regexp(rspack_regex::RspackRegex::new(
        &x.regexp_matcher.ok_or_else(|| {
          internal_error!(
            "should have a regexp_matcher when MinificationConditions.type is \"regexp\""
          )
        })?,
      )?),
      _ => panic!(
        "Failed to resolve the condition type {}. Expected type is `string`, `regexp` or `array`.",
        x.r#type
      ),
    };

    Ok(result)
  }
}

impl TryFrom<RawSwcJsMinimizerRules> for SwcJsMinimizerRules {
  type Error = rspack_error::Error;

  fn try_from(value: RawSwcJsMinimizerRules) -> Result<Self> {
    let result = match value.r#type.as_str() {
      "string" => Self::String(value.string_matcher.ok_or_else(|| {
        internal_error!("should have a string_matcher when MinificationConditions.type is \"string\"")
      })?),
      "regexp" => Self::Regexp(rspack_regex::RspackRegex::new(
        &value.regexp_matcher.ok_or_else(|| {
          internal_error!(
            "should have a regexp_matcher when MinificationConditions.type is \"regexp\""
          )
        })?,
      )?),
      "array" => Self::Array(
        value.array_matcher
          .ok_or_else(|| {
            internal_error!(
              "should have a array_matcher when MinificationConditions.type is \"array\""
            )
          })?
          .into_iter()
          .map(|i| i.try_into())
          .collect::<rspack_error::Result<Vec<_>>>()?,
      ),
      _ => panic!(
        "Failed to resolve the MinificationContions type {}. Expected type is `string`, `regexp`, `array`.",
        value.r#type
      ),
    };

    Ok(result)
  }
}
