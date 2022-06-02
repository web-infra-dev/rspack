use std::collections::HashMap;

#[cfg(not(feature = "test"))]
use napi_derive::napi;

use serde::Deserialize;

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum ResolveAliasValue {
  // `bool` just for derserialize
  False(bool),
  Target(String),
}

impl napi::bindgen_prelude::ToNapiValue for ResolveAliasValue {
  unsafe fn to_napi_value(
    env: napi::sys::napi_env,
    val: Self,
  ) -> napi::Result<napi::sys::napi_value> {
    match val {
      ResolveAliasValue::False(b) => bool::to_napi_value(env, b),
      ResolveAliasValue::Target(s) => String::to_napi_value(env, s),
    }
  }
}

impl napi::bindgen_prelude::FromNapiValue for ResolveAliasValue {
  unsafe fn from_napi_value(
    env: napi::sys::napi_env,
    napi_val: napi::sys::napi_value,
  ) -> napi::Result<Self> {
    let mut val_type = 0;

    napi::check_status!(
      napi::sys::napi_typeof(env, napi_val, &mut val_type),
      "Failed to convert napi value into rust type `ResolveAliasValue`",
    )?;

    match val_type {
      napi::sys::ValueType::napi_boolean => Ok(ResolveAliasValue::False(bool::from_napi_value(
        env, napi_val,
      )?)),
      _ => Ok(ResolveAliasValue::Target(String::from_napi_value(
        env, napi_val,
      )?)),
    }
  }
}

impl Default for ResolveAliasValue {
  fn default() -> Self {
    ResolveAliasValue::False(false)
  }
}

#[derive(Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
#[napi(object)]
#[cfg(not(feature = "test"))]
pub struct RawResolveOptions {
  pub extensions: Option<Vec<String>>,
  #[napi(ts_type = "{ [key: string]: false | string }")]
  pub alias: Option<HashMap<String, ResolveAliasValue>>,
  pub condition_names: Option<Vec<String>>,
  pub alias_field: Option<String>,
}

#[derive(Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
#[cfg(feature = "test")]
pub struct RawResolveOptions {
  pub extensions: Option<Vec<String>>,
  pub alias: Option<HashMap<String, ResolveAliasValue>>,
  pub condition_names: Option<Vec<String>>,
  pub alias_field: Option<String>,
}
