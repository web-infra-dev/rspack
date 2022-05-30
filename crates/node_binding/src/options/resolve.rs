use std::collections::HashMap;

use napi_derive::napi;
use rspack_core::BundleMode;
use serde::Deserialize;

#[derive(Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
#[napi(object)]
pub struct RawResolveOptions {
  pub alias: Option<HashMap<String, String>>,
  pub condition_names: Option<Vec<String>>,
  pub alias_field: Option<String>,
}
