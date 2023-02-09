use napi_derive::napi;
use rspack_core::NodeOption;
use serde::Deserialize;

#[derive(Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
#[napi(object)]
pub struct RawNodeOption {
  pub dirname: String,
}

impl From<RawNodeOption> for NodeOption {
  fn from(value: RawNodeOption) -> Self {
    Self {
      dirname: value.dirname,
    }
  }
}
