use std::collections::HashMap;

use napi_derive::napi;
use rspack_core::ExternalItem;
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
#[napi(object)]
pub struct RawExternalItem {
  #[napi(ts_type = r#""string" | "regexp" | "object""#)]
  pub r#type: String,
  pub string_payload: Option<String>,
  pub regexp_payload: Option<String>,
  pub object_payload: Option<HashMap<String, String>>,
}

impl From<RawExternalItem> for ExternalItem {
  fn from(value: RawExternalItem) -> Self {
    match value.r#type.as_str() {
      "string" => Self::from(
        value
          .string_payload
          .expect("should have a string_payload when RawExternalItem.type is \"string\""),
      ),
      "regexp" => Self::from(
        value
          .regexp_payload
          .expect("should have a regexp_payload when RawExternalItem.type is \"regexp\""),
      ),
      "object" => Self::from(
        value
          .object_payload
          .expect("should have a object_payload when RawExternalItem.type is \"object\""),
      ),
      _ => unreachable!(),
    }
  }
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
#[napi(object)]
pub struct RawExternalsPresets {
  pub node: bool,
}
