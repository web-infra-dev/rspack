use rspack_loader_swc::{SwcDtsEmitOptions};
use napi_derive::napi;

#[napi(object, object_to_js = false)]
pub struct RawSwcDtsEmitRspackPluginOptions {
  pub root_dir: Option<String>,
}

impl From<RawSwcDtsEmitRspackPluginOptions> for SwcDtsEmitOptions {
  fn from(value: RawSwcDtsEmitRspackPluginOptions) -> Self {
    Self {
      root_dir: value.root_dir.unwrap(),
    }
  }
}
