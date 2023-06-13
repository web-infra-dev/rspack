use std::ptr;

use napi::{bindgen_prelude::FromNapiValue, JsFunction, JsObject, JsString};

fn get_js_global_object(env: napi::sys::napi_env) -> napi::Result<JsObject> {
  let mut global = ptr::null_mut();
  let status = unsafe { napi::sys::napi_get_global(env, &mut global) };
  if status != 0 {
    panic!("napi: Get `global` failed")
  }
  unsafe { JsObject::from_napi_value(env, global) }
}

/// `Object.prototype.toString.call`
pub fn object_prototype_to_string_call(
  env: napi::sys::napi_env,
  obj: &JsObject,
) -> napi::Result<String> {
  let type_description: napi::Result<String> = try {
    let global: JsObject = get_js_global_object(env)?;
    // `Object`
    let object: JsObject = global.get_named_property("Object")?;
    let prototype: JsObject = object.get_named_property("prototype")?;
    let to_string: JsFunction = prototype.get_named_property("toString")?;
    let to_string_ret: JsString = to_string.call_without_args(Some(obj))?.try_into()?;
    to_string_ret.into_utf8()?.into_owned()?
  };
  type_description
}
