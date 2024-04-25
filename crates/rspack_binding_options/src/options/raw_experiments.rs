use napi_derive::napi;
use rspack_core::RspackFuture;

#[derive(Debug, Default)]
#[napi(object)]
pub struct RawRspackFuture {
  pub new_treeshaking: bool,
}

#[derive(Debug, Default)]
#[napi(object)]
pub struct RawExperiments {
  pub new_split_chunks: bool,
  pub top_level_await: bool,
  pub rspack_future: RawRspackFuture,
}

impl From<RawRspackFuture> for RspackFuture {
  fn from(value: RawRspackFuture) -> Self {
    Self {
      new_treeshaking: value.new_treeshaking,
    }
  }
}
