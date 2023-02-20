use napi_derive::napi;
use rspack_core::EntryItem;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
#[napi(object)]
pub struct RawEntryItem {
  pub import: Vec<String>,
  pub runtime: Option<String>,
}

impl From<RawEntryItem> for EntryItem {
  fn from(value: RawEntryItem) -> Self {
    Self {
      import: value.import,
      runtime: value.runtime,
    }
  }
}
