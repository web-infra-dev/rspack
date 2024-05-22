use napi_derive::napi;
use rspack_core::SnapshotOptions;

#[derive(Debug, Default)]
#[napi(object)]
pub struct RawSnapshotOptions;

impl From<RawSnapshotOptions> for SnapshotOptions {
  fn from(_value: RawSnapshotOptions) -> Self {
    SnapshotOptions
  }
}
