use derive_more::Debug;
use rspack_plugin_rslib::RslibPluginOptions;

#[derive(Debug)]
#[napi(object, object_to_js = false)]
pub struct RawRslibPluginOptions {
  /// Intercept partial parse hooks of APIPlugin, expect some statements not to be parsed as API.
  /// @default `false`
  pub intercept_api_plugin: Option<bool>,
  /// Add shims for javascript/esm modules
  /// @default `false`
  pub force_node_shims: Option<bool>,
  /// Auto downgrade module external type to node-commonjs for CJS require of node builtins
  /// @default `false`
  pub auto_cjs_node_builtin: Option<bool>,
}

impl From<RawRslibPluginOptions> for RslibPluginOptions {
  fn from(value: RawRslibPluginOptions) -> Self {
    Self {
      intercept_api_plugin: value.intercept_api_plugin.unwrap_or_default(),
      force_node_shims: value.force_node_shims.unwrap_or_default(),
      auto_cjs_node_builtin: value.auto_cjs_node_builtin.unwrap_or_default(),
    }
  }
}
