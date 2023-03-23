use napi_derive::napi;
use rspack_core::NodeOption;
use serde::Deserialize;

#[derive(Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
#[napi(object)]
pub struct RawNodeOption {
  pub dirname: String,
  pub filename: String,
  pub global: String,
}

impl From<RawNodeOption> for NodeOption {
  fn from(value: RawNodeOption) -> Self {
    Self {
      dirname: value.dirname,
      filename: value.filename,
      global: value.global,
    }
  }
}
