#[cfg(feature = "node-api")]
use napi_derive::napi;
use rspack_core::DecoratorOptions;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug, Serialize, Default, Clone)]
#[cfg(feature = "node-api")]
#[napi(object)]
#[serde(rename_all = "camelCase")]
pub struct RawDecoratorOptions {
  pub legacy: bool,
  pub emit_metadata: bool,
  pub use_define_for_class_fields: bool,
}

#[derive(Deserialize, Debug, Serialize, Default, Clone)]
#[cfg(not(feature = "node-api"))]
#[serde(rename_all = "camelCase")]
pub struct RawDecoratorOptions {
  pub legacy: bool,
  pub emit_metadata: bool,
  pub use_define_for_class_fields: bool,
}

pub fn transform_to_decorator_options(
  raw: Option<RawDecoratorOptions>,
) -> Option<DecoratorOptions> {
  raw.map(|inner| {
    let RawDecoratorOptions {
      legacy,
      emit_metadata,
      use_define_for_class_fields,
    } = inner;
    DecoratorOptions {
      legacy,
      emit_metadata,
      use_define_for_class_fields,
    }
  })
}
