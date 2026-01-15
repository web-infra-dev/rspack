use derive_more::Debug;
use rspack_plugin_rstest::RstestPluginOptions;

#[derive(Debug)]
#[napi(object, object_to_js = false)]
pub struct RawRstestPluginOptions {
  // Inject __dirname and __filename to each module.
  pub inject_module_path_name: bool,
  // Replace import.meta.dirname and import.meta.filename of each module.
  pub import_meta_path_name: bool,
  // Hoist mock module to the top of the module.
  pub hoist_mock_module: bool,
  // Root of the manual mock directory.
  pub manual_mock_root: String,
  // Preserve `new URL("*.<ext>", import.meta.url)` expressions for specified extensions
  // instead of transforming them to asset imports.
  // This allows rstest to dynamically load modules (e.g., wasm) at runtime.
  // Example: [".wasm"] to preserve wasm URL expressions.
  pub preserve_new_url: Option<Vec<String>>,
}

impl From<RawRstestPluginOptions> for RstestPluginOptions {
  fn from(value: RawRstestPluginOptions) -> Self {
    Self {
      module_path_name: value.inject_module_path_name,
      hoist_mock_module: value.hoist_mock_module,
      import_meta_path_name: value.import_meta_path_name,
      manual_mock_root: value.manual_mock_root,
      preserve_new_url: value.preserve_new_url.unwrap_or_default(),
    }
  }
}
