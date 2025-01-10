use napi_derive::napi;
use rspack_plugin_emit_dts::SwcDtsEmitRspackPluginOptions;

#[napi(object, object_to_js = false)]
pub struct RawSwcDtsEmitRspackPluginOptions {
  pub extension: Option<String>,
}

impl From<RawSwcDtsEmitRspackPluginOptions> for SwcDtsEmitRspackPluginOptions {
  fn from(value: RawSwcDtsEmitRspackPluginOptions) -> Self {
    Self {
      extension: value.extension.unwrap_or("d.ts".to_string()),
    }
  }
}
