use derive_more::Debug;
use rspack_plugin_rslib::{RslibPluginOptions, SwcEmitDtsOptions};

#[derive(Debug)]
#[napi(object, object_to_js = false)]
pub struct RawSwcEmitDtsOptions {
  pub root_dir: String,
  pub declaration_dir: String,
}

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
  /// Emit isolated declaration files for modules transformed by `builtin:swc-loader`
  pub emit_dts: Option<RawSwcEmitDtsOptions>,
}

impl From<RawRslibPluginOptions> for RslibPluginOptions {
  fn from(value: RawRslibPluginOptions) -> Self {
    Self {
      intercept_api_plugin: value.intercept_api_plugin.unwrap_or_default(),
      force_node_shims: value.force_node_shims.unwrap_or_default(),
      auto_cjs_node_builtin: value.auto_cjs_node_builtin.unwrap_or_default(),
      emit_dts: value.emit_dts.map(Into::into),
    }
  }
}

impl From<RawSwcEmitDtsOptions> for SwcEmitDtsOptions {
  fn from(value: RawSwcEmitDtsOptions) -> Self {
    Self {
      root_dir: value.root_dir,
      declaration_dir: value.declaration_dir,
    }
  }
}
