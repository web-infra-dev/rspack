use derive_more::Debug;
use rspack_plugin_rstest::RstestPluginOptions;

#[derive(Debug)]
#[napi(object, object_to_js = false)]
pub struct RawRstestPluginOptions {
  // Inject __dirname and __filename to each module.
  pub inject_module_path_name: bool,
}

impl From<RawRstestPluginOptions> for RstestPluginOptions {
  fn from(value: RawRstestPluginOptions) -> Self {
    Self {
      module_path_name: value.inject_module_path_name,
    }
  }
}
