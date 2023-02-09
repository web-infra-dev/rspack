use napi_derive::napi;
use rspack_core::{SnapshotOptions, SnapshotStrategy};
use serde::Deserialize;

#[derive(Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
#[napi(object)]
pub struct RawSnapshotStrategy {
  pub hash: bool,
  pub timestamp: bool,
}

impl From<RawSnapshotStrategy> for SnapshotStrategy {
  fn from(value: RawSnapshotStrategy) -> Self {
    Self {
      hash: value.hash,
      timestamp: value.timestamp,
    }
  }
}

#[derive(Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
#[napi(object)]
pub struct RawSnapshotOptions {
  pub resolve: RawSnapshotStrategy,
  pub module: RawSnapshotStrategy,
}

impl From<RawSnapshotOptions> for SnapshotOptions {
  fn from(value: RawSnapshotOptions) -> Self {
    let RawSnapshotOptions { resolve, module } = value;

    SnapshotOptions {
      resolve: resolve.into(),
      module: module.into(),
    }
  }
}
