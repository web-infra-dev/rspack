use std::fmt::Debug;

use napi::{
  bindgen_prelude::{FromNapiValue, TypeName, ValidateNapiValue},
  JsObject, NapiRaw, NapiValue,
};

use crate::object_prototype_to_string_call;

pub struct JsRegExp(JsObject);

impl Debug for JsRegExp {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_tuple("JsRegExp").field(&self.source()).finish()
  }
}

impl JsRegExp {
  pub fn source(&self) -> String {
    self
      .0
      .get_named_property("source")
      .expect("RegExp should have `source` property")
  }

  pub fn flags(&self) -> String {
    self
      .0
      .get_named_property("flags")
      .expect("RegExp should have `flags` property")
  }
}

impl NapiRaw for JsRegExp {
  unsafe fn raw(&self) -> napi::sys::napi_value {
    self.0.raw()
  }
}

impl FromNapiValue for JsRegExp {
  unsafe fn from_napi_value(
    env: napi::sys::napi_env,
    napi_val: napi::sys::napi_value,
  ) -> napi::Result<Self> {
    let js_object = JsObject::from_raw(env, napi_val)?;
    let ty_string = object_prototype_to_string_call(env, &js_object)?;
    if ty_string.as_str() != "[object RegExp]" {
      return Err(napi::Error::from_reason(format!(
        "Expect [object RegExp] got {ty_string}"
      )));
    }
    Ok(Self(js_object))
  }
}

impl TypeName for JsRegExp {
  fn type_name() -> &'static str {
    "RegExp"
  }

  fn value_type() -> napi::ValueType {
    napi::ValueType::Object
  }
}

impl ValidateNapiValue for JsRegExp {}
