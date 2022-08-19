use std::collections::HashMap;

use napi_derive::napi;
use serde::Deserialize;

use crate::{define_napi_object, RawReactOptions};

define_napi_object!(
  #[derive(Deserialize, Debug, Default)]
  #[serde(rename_all = "camelCase")]
  pub RawEnhancedOptions {
    pub svgr: Option<bool>,
    pub progress: Option<bool>,
    pub lazy_compilation: Option<bool>,
    pub react: Option<RawReactOptions>,
    pub inline_style: Option<bool>,
    pub globals: Option<HashMap<String, String>>,
    pub define: Option<HashMap<String, String>>
  }
);
