use napi_derive::napi;
use rspack_error::internal_error;
use rspack_plugin_swc_js_minimizer::{
  SwcJsMinimizerRspackPluginOptions, SwcJsMinimizerRule, SwcJsMinimizerRules,
};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
#[napi(object)]
pub struct RawSwcJsMinimizerRule {
  #[napi(ts_type = r#""string" | "regexp""#)]
  pub r#type: String,
  pub string_matcher: Option<String>,
  pub regexp_matcher: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
#[napi(object)]
pub struct RawSwcJsMinimizerRules {
  #[napi(ts_type = r#""string" | "regexp" | "array""#)]
  pub r#type: String,
  pub string_matcher: Option<String>,
  pub regexp_matcher: Option<String>,
  pub array_matcher: Option<Vec<RawSwcJsMinimizerRule>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
#[napi(object)]
pub struct RawSwcJsMinimizerRspackPluginOptions {
  pub passes: u32,
  pub drop_console: bool,
  pub keep_class_names: bool,
  pub keep_fn_names: bool,
  #[napi(ts_type = r#""all" | "some" | "false""#)]
  pub comments: String,
  pub ascii_only: bool,
  pub pure_funcs: Vec<String>,
  pub extract_comments: Option<String>,
  pub test: Option<RawSwcJsMinimizerRules>,
  pub include: Option<RawSwcJsMinimizerRules>,
  pub exclude: Option<RawSwcJsMinimizerRules>,
}

impl TryFrom<RawSwcJsMinimizerRspackPluginOptions> for SwcJsMinimizerRspackPluginOptions {
  type Error = rspack_error::Error;

  fn try_from(value: RawSwcJsMinimizerRspackPluginOptions) -> rspack_error::Result<Self> {
    fn try_condition(
      raw_condition: Option<RawSwcJsMinimizerRules>,
    ) -> Result<Option<SwcJsMinimizerRules>, rspack_error::Error> {
      let condition: Option<SwcJsMinimizerRules> = if let Some(test) = raw_condition {
        Some(test.try_into()?)
      } else {
        None
      };

      Ok(condition)
    }

    Ok(Self {
      passes: value.passes as usize,
      drop_console: value.drop_console,
      keep_class_names: value.keep_class_names,
      keep_fn_names: value.keep_fn_names,
      pure_funcs: value.pure_funcs,
      ascii_only: value.ascii_only,
      comments: value.comments,
      extract_comments: value.extract_comments,
      test: try_condition(value.test)?,
      include: try_condition(value.include)?,
      exclude: try_condition(value.exclude)?,
    })
  }
}

impl TryFrom<RawSwcJsMinimizerRule> for SwcJsMinimizerRule {
  type Error = rspack_error::Error;

  fn try_from(x: RawSwcJsMinimizerRule) -> rspack_error::Result<Self> {
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

  fn try_from(value: RawSwcJsMinimizerRules) -> rspack_error::Result<Self> {
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
