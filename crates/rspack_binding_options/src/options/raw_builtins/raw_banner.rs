use std::fmt::Debug;

use napi_derive::napi;
use rspack_error::internal_error;
use rspack_plugin_banner::{BannerCondition, BannerConditions, BannerConfig};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
#[napi(object)]
pub struct RawBannerCondition {
  #[napi(ts_type = r#""string" | "regexp""#)]
  pub r#type: String,
  pub string_matcher: Option<String>,
  pub regexp_matcher: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
#[napi(object)]
pub struct RawBannerConditions {
  #[napi(ts_type = r#""string" | "regexp" | "array""#)]
  pub r#type: String,
  pub string_matcher: Option<String>,
  pub regexp_matcher: Option<String>,
  pub array_matcher: Option<Vec<RawBannerCondition>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
#[napi(object)]
pub struct RawBannerConfig {
  pub banner: String,
  pub entry_only: Option<bool>,
  pub footer: Option<bool>,
  pub raw: Option<bool>,
  pub test: Option<RawBannerConditions>,
  pub include: Option<RawBannerConditions>,
  pub exclude: Option<RawBannerConditions>,
}

impl TryFrom<RawBannerCondition> for BannerCondition {
  type Error = rspack_error::Error;

  fn try_from(x: RawBannerCondition) -> rspack_error::Result<Self> {
    let result = match x.r#type.as_str() {
      "string" => Self::String(x.string_matcher.ok_or_else(|| {
        internal_error!("should have a string_matcher when BannerConditions.type is \"string\"")
      })?),
      "regexp" => Self::Regexp(rspack_regex::RspackRegex::new(
        &x.regexp_matcher.ok_or_else(|| {
          internal_error!(
            "should have a regexp_matcher when BannerConditions.type is \"regexp\""
          )
        })?,
      )?),
      _ => panic!(
        "Failed to resolve the condition type {}. Expected type is `string`, `regexp`, `array`, `logical` or `function`.",
        x.r#type
      ),
    };

    Ok(result)
  }
}

impl TryFrom<RawBannerConditions> for BannerConditions {
  type Error = rspack_error::Error;

  fn try_from(x: RawBannerConditions) -> rspack_error::Result<Self> {
    let result: BannerConditions = match x.r#type.as_str() {
      "string" => Self::String(x.string_matcher.ok_or_else(|| {
        internal_error!("should have a string_matcher when BannerConditions.type is \"string\"")
      })?),
      "regexp" => Self::Regexp(rspack_regex::RspackRegex::new(
        &x.regexp_matcher.ok_or_else(|| {
          internal_error!(
            "should have a regexp_matcher when BannerConditions.type is \"regexp\""
          )
        })?,
      )?),
      "array" => Self::Array(
        x.array_matcher
          .ok_or_else(|| {
            internal_error!(
              "should have a array_matcher when BannerConditions.type is \"array\""
            )
          })?
          .into_iter()
          .map(|i| i.try_into())
          .collect::<rspack_error::Result<Vec<_>>>()?,
      ),
      _ => panic!(
        "Failed to resolve the condition type {}. Expected type is `string`, `regexp`, `array`, `logical` or `function`.",
        x.r#type
      ),
    };

    Ok(result)
  }
}

impl TryFrom<RawBannerConfig> for BannerConfig {
  type Error = rspack_error::Error;

  fn try_from(value: RawBannerConfig) -> std::result::Result<Self, Self::Error> {
    fn try_condition(
      raw_condition: Option<RawBannerConditions>,
    ) -> Result<Option<BannerConditions>, rspack_error::Error> {
      let condition: Option<BannerConditions> = if let Some(test) = raw_condition {
        Some(test.try_into()?)
      } else {
        None
      };

      Ok(condition)
    }

    Ok(BannerConfig {
      banner: value.banner,
      entry_only: value.entry_only,
      footer: value.footer,
      raw: value.raw,
      test: try_condition(value.test)?,
      include: try_condition(value.include)?,
      exclude: try_condition(value.exclude)?,
    })
  }
}
