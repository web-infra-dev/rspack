use std::{fmt::Debug, sync::Arc};

use derivative::Derivative;
use napi::{Env, JsFunction};
use napi_derive::napi;
use rspack_error::internal_error;
use rspack_napi_shared::{
  threadsafe_function::{ThreadsafeFunction, ThreadsafeFunctionCallMode},
  NapiResultExt, NAPI_ENV,
};
use rspack_plugin_banner::{
  BannerContent, BannerContentFnCtx, BannerPluginOptions, BannerRule, BannerRules,
};
use serde::Deserialize;

use crate::chunk::JsChunk;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
#[napi(object)]
pub struct RawBannerRule {
  #[napi(ts_type = r#""string" | "regexp""#)]
  pub r#type: String,
  pub string_matcher: Option<String>,
  pub regexp_matcher: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
#[napi(object)]
pub struct RawBannerRules {
  #[napi(ts_type = r#""string" | "regexp" | "array""#)]
  pub r#type: String,
  pub string_matcher: Option<String>,
  pub regexp_matcher: Option<String>,
  pub array_matcher: Option<Vec<RawBannerRule>>,
}

#[napi(object)]
pub struct RawBannerContentFnCtx {
  pub hash: String,
  pub chunk: JsChunk,
  pub filename: String,
}

impl<'a> From<BannerContentFnCtx<'a>> for RawBannerContentFnCtx {
  fn from(value: BannerContentFnCtx) -> Self {
    Self {
      hash: value.hash.to_string(),
      chunk: JsChunk::from(value.chunk),
      filename: value.filename.to_string(),
    }
  }
}

#[derive(Derivative, Deserialize)]
#[derivative(Debug)]
#[serde(rename_all = "camelCase")]
#[napi(object)]
pub struct RawBannerContent {
  #[napi(ts_type = r#""string" | "function""#)]
  pub r#type: String,
  pub string_payload: Option<String>,
  #[derivative(Debug = "ignore")]
  #[serde(skip_deserializing)]
  pub fn_payload: Option<JsFunction>,
}

impl TryFrom<RawBannerContent> for BannerContent {
  type Error = rspack_error::Error;

  fn try_from(value: RawBannerContent) -> Result<Self, Self::Error> {
    match value.r#type.as_str() {
      "string" => {
        let s = value.string_payload.ok_or_else(|| {
          internal_error!("should have a string_payload when RawBannerContent.type is \"string\"")
        })?;
        Ok(BannerContent::String(s))
      }
      "function" => {
        let func = value.fn_payload.ok_or_else(|| {
          internal_error!("should have a fn_payload when RawBannerContent.type is \"function\"")
        })?;
        let func: ThreadsafeFunction<RawBannerContentFnCtx, String> =
          NAPI_ENV.with(|env| -> anyhow::Result<_> {
            let env = env.borrow().expect("Failed to get env with external");
            let func_use = rspack_binding_macros::js_fn_into_threadsafe_fn!(func, &Env::from(env));
            Ok(func_use)
          })?;
        let func = Arc::new(func);
        Ok(BannerContent::Fn(Box::new(
          move |ctx: BannerContentFnCtx| {
            let func = func.clone();
            Box::pin(async move {
              func
                .call(ctx.into(), ThreadsafeFunctionCallMode::NonBlocking)
                .into_rspack_result()?
                .await
                .map_err(|err| internal_error!("Failed to call rule.use function: {err}"))?
            })
          },
        )))
      }
      _ => unreachable!(),
    }
  }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
#[napi(object)]
pub struct RawBannerPluginOptions {
  pub banner: RawBannerContent,
  pub entry_only: Option<bool>,
  pub footer: Option<bool>,
  pub raw: Option<bool>,
  pub test: Option<RawBannerRules>,
  pub include: Option<RawBannerRules>,
  pub exclude: Option<RawBannerRules>,
}

impl TryFrom<RawBannerRule> for BannerRule {
  type Error = rspack_error::Error;

  fn try_from(x: RawBannerRule) -> rspack_error::Result<Self> {
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

impl TryFrom<RawBannerRules> for BannerRules {
  type Error = rspack_error::Error;

  fn try_from(x: RawBannerRules) -> rspack_error::Result<Self> {
    let result: BannerRules = match x.r#type.as_str() {
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

impl TryFrom<RawBannerPluginOptions> for BannerPluginOptions {
  type Error = rspack_error::Error;

  fn try_from(value: RawBannerPluginOptions) -> std::result::Result<Self, Self::Error> {
    fn try_condition(
      raw_condition: Option<RawBannerRules>,
    ) -> Result<Option<BannerRules>, rspack_error::Error> {
      let condition: Option<BannerRules> = if let Some(test) = raw_condition {
        Some(test.try_into()?)
      } else {
        None
      };

      Ok(condition)
    }

    Ok(BannerPluginOptions {
      banner: value.banner.try_into()?,
      entry_only: value.entry_only,
      footer: value.footer,
      raw: value.raw,
      test: try_condition(value.test)?,
      include: try_condition(value.include)?,
      exclude: try_condition(value.exclude)?,
    })
  }
}
