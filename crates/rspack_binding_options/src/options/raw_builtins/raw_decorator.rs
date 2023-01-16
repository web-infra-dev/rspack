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

pub fn transform_to_decorator_options(
  raw: Option<RawDecoratorOptions>,
) -> Option<DecoratorOptions> {
  raw.map(|inner| {
    let RawDecoratorOptions {
      legacy,
      emit_metadata,
    } = inner;
    DecoratorOptions {
      legacy,
      emit_metadata,
    }
  })
}
