use napi_derive::napi;
use rspack_plugin_css::{plugin::PostcssConfig, pxtorem::options::PxToRemOptions};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug, Serialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
#[napi(object)]
pub struct RawPostCssConfig {
  pub pxtorem: Option<RawPxToRemConfig>,
}

impl From<RawPostCssConfig> for PostcssConfig {
  fn from(value: RawPostCssConfig) -> Self {
    Self {
      pxtorem: value.pxtorem.map(|item| item.into()),
    }
  }
}

// postcss-px-to-rem
#[derive(Deserialize, Debug, Serialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
#[napi(object)]
pub struct RawPxToRemConfig {
  pub root_value: Option<u32>,
  pub unit_precision: Option<u32>,
  pub selector_black_list: Option<Vec<String>>,
  pub prop_list: Option<Vec<String>>,
  pub replace: Option<bool>,
  pub media_query: Option<bool>,
  pub min_pixel_value: Option<f64>,
}

impl From<RawPxToRemConfig> for PxToRemOptions {
  fn from(value: RawPxToRemConfig) -> Self {
    Self {
      root_value: value.root_value,
      unit_precision: value.unit_precision,
      selector_black_list: value.selector_black_list,
      prop_list: value.prop_list,
      replace: value.replace,
      media_query: value.media_query,
      min_pixel_value: value.min_pixel_value,
    }
  }
}
