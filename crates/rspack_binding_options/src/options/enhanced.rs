use std::collections::HashMap;

use napi_derive::napi;
use serde::Deserialize;

use crate::RawReactOptions;

#[derive(Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
#[cfg(feature = "node-api")]
#[napi(object)]
pub struct RawEnhancedOptions {
  pub svgr: Option<bool>,
  pub progress: Option<bool>,
  pub lazy_compilation: Option<bool>,
  pub react: Option<RawReactOptions>,
  pub inline_style: Option<bool>,
  pub globals: Option<HashMap<String, String>>,
  pub define: Option<HashMap<String, String>>,
}

#[derive(Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
#[cfg(not(feature = "node-api"))]
pub struct RawEnhancedOptions {
  pub svgr: Option<bool>,
  pub progress: Option<bool>,
  pub lazy_compilation: Option<bool>,
  pub react: Option<RawReactOptions>,
  pub inline_style: Option<bool>,
  pub globals: Option<HashMap<String, String>>,
  pub define: Option<HashMap<String, String>>,
}
