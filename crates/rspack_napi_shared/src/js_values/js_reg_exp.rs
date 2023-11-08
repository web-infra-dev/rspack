use std::fmt::Debug;

use napi::{
  bindgen_prelude::{FromNapiValue, TypeName, ValidateNapiValue},
  sys, Error, JsObject, NapiRaw, Result, Status,
};

use crate::utils::NapiType;

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
  unsafe fn raw(&self) -> sys::napi_value {
    unsafe { self.0.raw() }
  }
}

impl FromNapiValue for JsRegExp {
  unsafe fn from_napi_value(env: sys::napi_env, napi_val: sys::napi_value) -> Result<Self> {
    let ty = NapiType::new(env, napi_val)?;
    if !ty.is_regex()? {
      return Err(Error::new(
        Status::InvalidArg,
        format!(
          "Expect value to be '[object RegExp]', but received {}",
          ty.get_type()?
        ),
      ));
    }
    Ok(Self(unsafe { JsObject::from_napi_value(env, napi_val) }?))
  }
}

impl ValidateNapiValue for JsRegExp {}

impl TypeName for JsRegExp {
  fn type_name() -> &'static str {
    "RegExp"
  }

  fn value_type() -> napi::ValueType {
    napi::ValueType::Object
  }
}
