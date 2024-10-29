use std::fmt::Debug;

use napi::{
  bindgen_prelude::{FromNapiValue, Function, TypeName, ValidateNapiValue},
  sys, Env, Error, JsObject, JsUnknown, NapiRaw, NapiValue, Result, Status,
};

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
    let regexp_constructor = global.get_named_property::<Function<JsUnknown, ()>>("RegExp")?;

    if js_object.instanceof(regexp_constructor)? {
      Ok(Self(js_object))
    } else {
      Err(Error::new(
        Status::ObjectExpected,
        format!(
          "Expect value to be '[object RegExp]', but received {}",
          js_object.coerce_to_string()?.into_utf8()?.as_str()?
        ),
      ))
    }
  }
}
