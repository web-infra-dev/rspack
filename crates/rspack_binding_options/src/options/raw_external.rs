use std::collections::HashMap;
use std::fmt::Debug;

use napi::JsFunction;
use napi_derive::napi;
use rspack_core::ExternalItemFnCtx;
use rspack_core::{ExternalItem, ExternalItemFnResult, ExternalItemObject, ExternalItemValue};
use rspack_regex::RspackRegex;
use serde::Deserialize;
use {
  napi::Env,
  rspack_error::internal_error,
  rspack_napi_shared::threadsafe_function::{ThreadsafeFunction, ThreadsafeFunctionCallMode},
  rspack_napi_shared::NapiResultExt,
  rspack_napi_shared::NAPI_ENV,
  std::sync::Arc,
};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
#[napi(object)]
pub struct RawExternalItem {
  #[napi(ts_type = r#""string" | "regexp" | "object" | "function""#)]
  pub r#type: String,
  pub string_payload: Option<String>,
  pub regexp_payload: Option<String>,
  pub object_payload: Option<HashMap<String, RawExternalItemValue>>,
  #[serde(skip_deserializing)]
  #[napi(ts_type = r#"(value: any) => any"#)]
  pub fn_payload: Option<JsFunction>,
}

impl Debug for RawExternalItem {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("RawExternalItem")
      .field("r#type", &self.r#type)
      .field("string_payload", &self.string_payload)
      .field("regexp_payload", &self.regexp_payload)
      .field("object_payload", &self.object_payload)
      .field("fn_payload", &"Function")
      .finish()
  }
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
#[napi(object)]
pub struct RawExternalItemValue {
  #[napi(ts_type = r#""string" | "bool" | "array""#)]
  pub r#type: String,
  pub string_payload: Option<String>,
  pub bool_payload: Option<bool>,
  pub array_payload: Option<Vec<String>>,
}

impl From<RawExternalItemValue> for ExternalItemValue {
  fn from(value: RawExternalItemValue) -> Self {
    match value.r#type.as_str() {
      "string" => Self::String(
        value
          .string_payload
          .expect("should have a string_payload when RawExternalItemValue.type is \"string\""),
      ),
      "bool" => Self::Bool(
        value
          .bool_payload
          .expect("should have a bool_payload when RawExternalItemValue.type is \"bool\""),
      ),
      "array" => Self::Array(
        value
          .array_payload
          .expect("should have a array_payload when RawExternalItemValue.type is \"array\""),
      ),
      _ => unreachable!(),
    }
  }
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
#[napi(object)]
pub struct RawExternalItemFnResult {
  pub external_type: Option<String>,
  pub result: Option<RawExternalItemValue>,
}

impl From<RawExternalItemFnResult> for ExternalItemFnResult {
  fn from(value: RawExternalItemFnResult) -> Self {
    Self {
      external_type: value.external_type,
      result: value.result.map(Into::into),
    }
  }
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
#[napi(object)]
pub struct RawExternalItemFnCtx {
  pub request: String,
  pub context: String,
  pub dependency_type: String,
}

impl From<ExternalItemFnCtx> for RawExternalItemFnCtx {
  fn from(value: ExternalItemFnCtx) -> Self {
    Self {
      request: value.request,
      dependency_type: value.dependency_type,
      context: value.context,
    }
  }
}

impl TryFrom<RawExternalItem> for ExternalItem {
  type Error = anyhow::Error;

  #[allow(clippy::unwrap_in_result)]
  fn try_from(value: RawExternalItem) -> anyhow::Result<Self> {
    match value.r#type.as_str() {
      "string" => Ok(Self::from(value.string_payload.expect(
        "should have a string_payload when RawExternalItem.type is \"string\"",
      ))),
      "regexp" => {
        let payload = value
          .regexp_payload
          .expect("should have a regexp_payload when RawExternalItem.type is \"regexp\"");
        let reg =
          RspackRegex::new(&payload).expect("regex_payload is not a legal regex in rust side");
        Ok(Self::from(reg))
      }
      "object" => {
        let payload: ExternalItemObject = value
          .object_payload
          .expect("should have a object_payload when RawExternalItem.type is \"object\"")
          .into_iter()
          .map(|(k, v)| (k, v.into()))
          .collect();
        Ok(payload.into())
      }
      "function" => {
        let fn_payload = value
          .fn_payload
          .expect("should have a fn_payload for external");
        let fn_payload: ThreadsafeFunction<RawExternalItemFnCtx, RawExternalItemFnResult> =
          NAPI_ENV.with(|env| -> anyhow::Result<_> {
            let env = env.borrow().expect("Failed to get env with external");
            let fn_payload =
              rspack_binding_macros::js_fn_into_theadsafe_fn!(fn_payload, &Env::from(env));
            Ok(fn_payload)
          })?;
        let fn_payload = Arc::new(fn_payload);
        Ok(Self::Fn(Box::new(move |ctx: ExternalItemFnCtx| {
          let fn_payload = fn_payload.clone();
          Box::pin(async move {
            fn_payload
              .call(ctx.into(), ThreadsafeFunctionCallMode::NonBlocking)
              .into_rspack_result()?
              .await
              .map_err(|err| internal_error!("Failed to call external function: {err}"))?
              .map(|r| r.into())
          })
        })))
      }
      _ => unreachable!(),
    }
  }
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
#[napi(object)]
pub struct RawExternalsPresets {
  pub node: bool,
  pub web: bool,
}
