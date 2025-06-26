use rspack_collections::Identifier;
use rspack_napi::napi::bindgen_prelude::{Result, ToNapiValue};

pub struct JsIdentifier(Identifier);

impl JsIdentifier {
  pub fn raw(&self) -> Identifier {
    self.0
  }
}

impl From<Identifier> for JsIdentifier {
  fn from(value: Identifier) -> Self {
    JsIdentifier(value)
  }
}

impl ToNapiValue for JsIdentifier {
  unsafe fn to_napi_value(env: napi::sys::napi_env, val: Self) -> Result<napi::sys::napi_value> {
    ToNapiValue::to_napi_value(env, val.0.as_str())
  }
}
