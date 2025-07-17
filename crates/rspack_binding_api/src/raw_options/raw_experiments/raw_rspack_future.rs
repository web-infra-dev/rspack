use napi_derive::napi;
use rspack_core::RspackFuture;

#[allow(clippy::empty_structs_with_brackets)]
#[derive(Debug, Default)]
#[napi(object, object_to_js = false)]
pub struct RawRspackFuture {}

impl From<RawRspackFuture> for RspackFuture {
  fn from(_value: RawRspackFuture) -> Self {
    Self {}
  }
}
