use napi_derive::napi;
use rspack_core::DecoratorOptions;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug, Serialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
#[napi(object)]
pub struct RawDecoratorOptions {
  pub legacy: bool,
  pub emit_metadata: bool,
}

impl From<RawDecoratorOptions> for DecoratorOptions {
  fn from(value: RawDecoratorOptions) -> Self {
    Self {
      legacy: value.legacy,
      emit_metadata: value.emit_metadata,
    }
  }
}
