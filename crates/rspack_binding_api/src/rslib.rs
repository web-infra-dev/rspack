use derive_more::Debug;
use rspack_plugin_rslib::RslibPluginOptions;

#[derive(Debug)]
#[napi(object, object_to_js = false)]
pub struct RawRslibPluginOptions {
  pub intercept_api_plugin: bool,
}

impl From<RawRslibPluginOptions> for RslibPluginOptions {
  fn from(value: RawRslibPluginOptions) -> Self {
    Self {
      intercept_api_plugin: value.intercept_api_plugin,
    }
  }
}
