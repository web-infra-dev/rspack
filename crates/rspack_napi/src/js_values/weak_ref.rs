#![allow(clippy::not_unsafe_ptr_arg_deref)]

use std::{cell::Cell, ptr, rc::Rc};

use napi::{
  bindgen_prelude::{check_status, JsObjectValue, Object, ToNapiValue},
  sys::{self, napi_env},
  Env, JsValue, Result,
};

pub struct WeakRef {
  raw_ref: sys::napi_ref,
  deleted: Rc<Cell<bool>>,
}

impl WeakRef {
  pub fn new(env: napi_env, object: &mut Object) -> Result<Self> {
    let mut raw_ref = ptr::null_mut();
    check_status!(unsafe { sys::napi_create_reference(env, object.raw(), 0, &mut raw_ref) })?;

    let deleted = Rc::new(Cell::new(false));
    let deleted_clone = deleted.clone();
    object.add_finalizer((), (), move |_ctx| {
      deleted_clone.set(true);
    })?;

    Ok(Self { raw_ref, deleted })
  }

  pub fn as_object(&self, env: &Env) -> Result<Object<'static>> {
    let napi_val = unsafe { ToNapiValue::to_napi_value(env.raw(), self)? };
    Ok(Object::from_raw(env.raw(), napi_val))
  }
}

impl ToNapiValue for &WeakRef {
  unsafe fn to_napi_value(env: sys::napi_env, val: Self) -> Result<sys::napi_value> {
    if val.deleted.get() {
      return Err(napi::Error::new(
        napi::Status::InvalidArg,
        "WeakRef has been deleted",
      ));
    }
    let mut result = ptr::null_mut();
    check_status!(
      unsafe { sys::napi_get_reference_value(env, val.raw_ref, &mut result) },
      "Failed to get reference value"
    )?;
    Ok(result)
  }
}

impl ToNapiValue for &mut WeakRef {
  unsafe fn to_napi_value(env: sys::napi_env, val: Self) -> Result<sys::napi_value> {
    if val.deleted.get() {
      return Err(napi::Error::new(
        napi::Status::InvalidArg,
        "WeakRef has been deleted",
      ));
    }
    let mut result = ptr::null_mut();
    check_status!(
      unsafe { sys::napi_get_reference_value(env, val.raw_ref, &mut result) },
      "Failed to get reference value"
    )?;
    Ok(result)
  }
}
