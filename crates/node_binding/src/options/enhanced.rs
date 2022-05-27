use std::collections::HashMap;

use napi_derive::napi;
use rspack_core::BundleMode;
use serde::Deserialize;

use crate::RawReactOptions;

#[derive(Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
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

impl From<BundleMode> for RawEnhancedOptions {
  fn from(mode: BundleMode) -> Self {
    Self {
      svgr: Some(false),
      progress: Some(true),
      lazy_compilation: Some(false),
      react: Some(mode.into()),
      inline_style: Some(false),
      globals: Some(Default::default()),
      define: Some(Default::default()),
    }
  }
}
