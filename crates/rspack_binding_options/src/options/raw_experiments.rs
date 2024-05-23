use napi_derive::napi;
use rspack_core::RspackFuture;

#[allow(clippy::empty_structs_with_brackets)]
#[derive(Debug, Default)]
#[napi(object)]
pub struct RawRspackFuture {}

#[derive(Debug, Default)]
#[napi(object)]
pub struct RawExperiments {
  pub top_level_await: bool,
  pub rspack_future: RawRspackFuture,
}

impl From<RawRspackFuture> for RspackFuture {
  fn from(_value: RawRspackFuture) -> Self {
    Self {}
  }
}
