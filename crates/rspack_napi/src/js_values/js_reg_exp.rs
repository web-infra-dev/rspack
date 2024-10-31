use std::fmt::Debug;

use napi::{
  bindgen_prelude::{FromNapiValue, TypeName, ValidateNapiValue},
  sys, Env, Error, JsFunction, JsObject, NapiRaw, NapiValue, Result, Status,
};
use rspack_regex::RspackRegex;

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

impl ValidateNapiValue for JsRegExp {}

impl TypeName for JsRegExp {
  fn type_name() -> &'static str {
    "RegExp"
  }

  fn value_type() -> napi::ValueType {
    napi::ValueType::Object
  }
}

impl NapiRaw for JsRegExp {
  unsafe fn raw(&self) -> sys::napi_value {
    unsafe { self.0.raw() }
  }
}

impl FromNapiValue for JsRegExp {
  unsafe fn from_napi_value(env: sys::napi_env, napi_val: sys::napi_value) -> Result<Self> {
    let js_object = unsafe { JsObject::from_raw_unchecked(env, napi_val) };

    let env = Env::from(env);
    let global = env.get_global()?;
    let object_prototype_to_string = global
      .get_named_property_unchecked::<JsObject>("Object")?
      .get_named_property::<JsObject>("prototype")?
      .get_named_property::<JsFunction>("toString")?;

    let js_string = object_prototype_to_string
      .call_without_args(Some(&js_object))?
      .coerce_to_string()?
      .into_utf8()?;
    let js_object_type = js_string.as_str()?;

    if js_object_type == "[object RegExp]" {
      Ok(Self(js_object))
    } else {
      Err(Error::new(
        Status::ObjectExpected,
        format!(
          "Expect value to be '[object RegExp]', but received {}",
          js_object_type
        ),
      ))
    }
  }
}

impl From<JsRegExp> for RspackRegex {
  fn from(value: JsRegExp) -> Self {
    let pat = value.source();
    let flags = value.flags();

    Self::with_flags(&pat, &flags).unwrap_or_else(|_| {
      panic!(
        "Try convert {:?} to RspackRegex with flags: {:?} failed",
        pat, flags
      )
    })
  }
}
