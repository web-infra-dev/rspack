use std::fmt::Debug;

use napi::{JsObject, NapiRaw, NapiValue};

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
}

impl NapiRaw for JsRegExp {
  unsafe fn raw(&self) -> napi::sys::napi_value {
    self.0.raw()
  }
}

impl NapiValue for JsRegExp {
  unsafe fn from_raw(env: napi::sys::napi_env, value: napi::sys::napi_value) -> napi::Result<Self> {
    let js_object = JsObject::from_raw(env, value)?;
    // TODO(hyf0): we should do dome validation here. Such as make sure the type of the Object is `[object RegExp]`
    Ok(Self(js_object))
  }

  unsafe fn from_raw_unchecked(env: napi::sys::napi_env, value: napi::sys::napi_value) -> Self {
    Self::from_raw(env, value).expect("Should not failed")
  }
}
