use std::marker::PhantomData;

use napi::{
  bindgen_prelude::FromNapiValue, sys, Env, JsFunction, JsObject, JsString, JsStringUtf8,
  JsUnknown, NapiRaw, NapiValue, Result,
};

/// `Object.prototype.toString.call`
/// Safety: [napi::JsStringUtf8]'s lifetime is bound to `&T`
unsafe fn object_prototype_to_string_call<T: NapiRaw>(
  raw_env: sys::napi_env,
  obj: &T,
) -> Result<JsStringUtf8> {
  let env = Env::from(raw_env);
  let s: JsString = env
    .get_global()?
    // `Object` is a function, but we want to use it as an JSObject.
    .get_named_property_unchecked::<JsObject>("Object")?
    .get_named_property::<JsObject>("prototype")?
    .get_named_property::<JsFunction>("toString")?
    .call_without_args(Some(
      // Safety: `JsObject::call_without_args` only leverages the `JsObject::raw` method.
      // It's not necessarily have to be exactly an `JsObject` instance.
      unsafe { &JsObject::from_raw_unchecked(raw_env, obj.raw()) },
    ))?
    .try_into()?;
  s.into_utf8()
}

pub struct NapiTypeRef<'r>(JsStringUtf8, PhantomData<&'r *mut ()>);

impl<'r> NapiTypeRef<'r> {
  // Safety: This call would be successful when `val` is a valid `impl NapiRaw` and `env` is a valid `napi_env`.
  pub unsafe fn new<T: NapiRaw>(env: sys::napi_env, val: &'r T) -> Result<NapiTypeRef<'r>> {
    let s = unsafe { object_prototype_to_string_call(env, val) }?;
    Ok(Self(s, PhantomData))
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
