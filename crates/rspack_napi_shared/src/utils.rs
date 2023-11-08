use napi::{
  bindgen_prelude::FromNapiValue, sys, Env, JsFunction, JsObject, JsString, JsStringUtf8,
  JsUnknown, Result,
};

/// `Object.prototype.toString.call`
fn object_prototype_to_string_call(
  raw_env: napi::sys::napi_env,
  obj: &JsObject,
) -> Result<JsStringUtf8> {
  let env = Env::from(raw_env);
  let s: JsString = env
    .get_global()?
    .get_named_property::<JsObject>("Object")?
    .get_named_property::<JsObject>("prototype")?
    .get_named_property::<JsFunction>("toString")?
    .call_without_args(Some(obj))?
    .try_into()?;
  s.into_utf8()
}

pub struct NapiType(JsStringUtf8);

impl NapiType {
  pub fn new(env: sys::napi_env, val: sys::napi_value) -> Result<Self> {
    let o = unsafe { JsObject::from_napi_value(env, val) }?;
    let s = object_prototype_to_string_call(env, &o)?;
    Ok(Self(s))
  }

  pub fn get_type(&self) -> Result<&str> {
    self.0.as_str()
  }

  pub fn is_regex(&self) -> Result<bool> {
    Ok(self.get_type()? == "[object RegExp]")
  }
}

pub fn downcast_into<T: FromNapiValue + 'static>(o: JsUnknown) -> napi::Result<T> {
  <T as FromNapiValue>::from_unknown(o)
}
