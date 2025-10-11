use derive_more::Debug;
use rspack_plugin_rslib::RslibPluginOptions;

#[derive(Debug)]
#[napi(object, object_to_js = false)]
pub struct RawRslibPluginOptions {
  /// Intercept partial parse hooks of APIPlugin, expect some statements not to be parsed as API.
  /// @default `false`
  pub intercept_api_plugin: Option<bool>,
  /// Use the compact runtime for dynamic import from `modern-module`, commonly used in CommonJS output.
  /// This field should not be set to `true` when using `modern-module` with ESM output, as it is already in use.
  /// @default `false`
  pub compact_external_module_dynamic_import: Option<bool>,
  /// Add shims for javascript/esm modules
  /// @default `false`
  pub force_node_shims: Option<bool>,
}

impl From<RawRslibPluginOptions> for RslibPluginOptions {
  fn from(value: RawRslibPluginOptions) -> Self {
    Self {
      intercept_api_plugin: value.intercept_api_plugin.unwrap_or_default(),
      compact_external_module_dynamic_import: value
        .compact_external_module_dynamic_import
        .unwrap_or_default(),
      force_node_shims: value.force_node_shims.unwrap_or_default(),
    }
  }
}
