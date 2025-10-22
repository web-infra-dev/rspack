use rspack_core::MangleExportsOptions;
use rspack_plugin_javascript::RenameExportsPluginOptions;

use crate::raw_options::RawMangleExportsOptions;

#[derive(Debug)]
#[napi(object, object_to_js = false)]
pub struct RawRenameExportsPluginOptions {
  #[napi(ts_type = r#"boolean | "size" | "deterministic""#)]
  pub mangle_exports: RawMangleExportsOptions,
  pub inline_exports: bool,
}

impl From<RawRenameExportsPluginOptions> for RenameExportsPluginOptions {
  fn from(value: RawRenameExportsPluginOptions) -> Self {
    Self {
      mangle_exports: value.mangle_exports.into(),
      inline_exports: value.inline_exports,
    }
  }
}
