use napi_derive::napi;
use rspack_plugin_remove_duplicate_modules::RemoveDuplicateModulesPluginOptions;

#[derive(Debug, Clone)]
#[napi(object)]
pub struct RawRemoveDuplicateModulesPluginOptions {
  pub min_size: Option<f64>,
}

impl From<RawRemoveDuplicateModulesPluginOptions> for RemoveDuplicateModulesPluginOptions {
  fn from(value: RawRemoveDuplicateModulesPluginOptions) -> Self {
    Self {
      min_size: value.min_size.unwrap_or_default().max(0.0),
    }
  }
}
