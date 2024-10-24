#![allow(clippy::not_unsafe_ptr_arg_deref)]

use std::cell::RefCell;
use std::ptr;
use std::rc::Rc;

use napi::bindgen_prelude::{check_status, ToNapiValue};
use napi::sys::{self, napi_env, napi_value};
use napi::{Env, Result};

// A RAII (Resource Acquisition Is Initialization) style wrapper around `Ref` that ensures the
// reference is unreferenced when it goes out of scope. This struct maintains a single reference
// count and automatically cleans up when it is dropped.
pub struct OneShotRef {
  env: napi_env,
  raw_ref: sys::napi_ref,
  cleanup_flag: Rc<RefCell<bool>>,
}

impl OneShotRef {
  pub fn new(env: napi_env, value: napi_value) -> Result<Self> {
    let mut raw_ref = ptr::null_mut();
    check_status!(unsafe { sys::napi_create_reference(env, value, 1, &mut raw_ref) })?;

    // cleanup references to be executed when the JS thread exits normally
    let cleanup_flag = Rc::new(RefCell::new(false));
    let mut env_wrapper = unsafe { Env::from_raw(env) };
    let _ = env_wrapper.add_env_cleanup_hook(cleanup_flag.clone(), move |cleanup_flag| {
      if !*cleanup_flag.borrow() {
        *cleanup_flag.borrow_mut() = true;
        unsafe { sys::napi_delete_reference(env, raw_ref) };
      }
    });

    Ok(Self {
      env,
      raw_ref,
      cleanup_flag,
    })
  }
}

impl Drop for OneShotRef {
  fn drop(&mut self) {
    if !*self.cleanup_flag.borrow() {
      *self.cleanup_flag.borrow_mut() = true;
      unsafe { sys::napi_delete_reference(self.env, self.raw_ref) };
    }
  }
}

impl ToNapiValue for &OneShotRef {
  unsafe fn to_napi_value(env: sys::napi_env, val: Self) -> Result<sys::napi_value> {
    let mut result = ptr::null_mut();
    check_status!(
      unsafe { sys::napi_get_reference_value(env, val.raw_ref, &mut result) },
      "Failed to get reference value"
    )?;
    Ok(result)
  }
}

impl ToNapiValue for &mut OneShotRef {
  unsafe fn to_napi_value(env: sys::napi_env, val: Self) -> Result<sys::napi_value> {
    let mut result = ptr::null_mut();
    check_status!(
      unsafe { sys::napi_get_reference_value(env, val.raw_ref, &mut result) },
      "Failed to get reference value"
    )?;
    Ok(result)
  }
}
