#[cfg(feature = "node-api")]
use napi_derive::napi;

use rspack_plugin_css::pxtorem::option::PxToRemOption;
// use rspack_plugin_css::pxtorem::option;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug, Serialize, Default, Clone)]
#[cfg(feature = "node-api")]
#[napi(object)]
#[serde(rename_all = "camelCase")]
pub struct RawPostCssConfig {
  pub pxtorem: Option<RawPxToRemConfig>,
}

#[derive(Deserialize, Debug, Serialize, Default, Clone)]
#[cfg(not(feature = "node-api"))]
#[serde(rename_all = "camelCase")]
pub struct RawPostCssConfig {
  pub pxtorem: Option<RawPxToRemConfig>,
}

// postcss-px-to-rem
#[derive(Deserialize, Debug, Serialize, Default, Clone)]
#[cfg(feature = "node-api")]
#[napi(object)]
#[serde(rename_all = "camelCase")]
pub struct RawPxToRemConfig {
  pub root_value: Option<u32>,
  pub unit_precision: Option<u32>,
  pub selector_black_list: Option<Vec<String>>,
  pub prop_list: Option<Vec<String>>,
  pub replace: Option<bool>,
  pub media_query: Option<bool>,
  pub min_pixel_value: Option<f64>,
}

#[derive(Deserialize, Debug, Serialize, Default, Clone)]
#[cfg(not(feature = "node-api"))]
#[serde(rename_all = "camelCase")]
pub struct RawPxToRemConfig {
  pub root_value: Option<u32>,
  pub unit_precision: Option<u32>,
  pub selector_black_list: Option<Vec<String>>,
  pub prop_list: Option<Vec<String>>,
  pub replace: Option<bool>,
  pub media_query: Option<bool>,
  pub min_pixel_value: Option<f64>,
}
#[allow(clippy::from_over_into)]
/// I need to use `Into` instead of `From` because
/// using `From` means I need to import [RawPostCssConfig]
/// in `rspack_plugin_css` which lead a cycle reference
/// `rspack_plugin_css <- rspack_binding_options` <- `rspack_plugin_css`
impl Into<PxToRemOption> for RawPxToRemConfig {
  fn into(self) -> PxToRemOption {
    PxToRemOption {
      root_value: self.root_value,
      unit_precision: self.unit_precision,
      selector_black_list: self.selector_black_list,
      prop_list: self.prop_list,
      replace: self.replace,
      media_query: self.media_query,
      min_pixel_value: self.min_pixel_value,
    }
  }
}
