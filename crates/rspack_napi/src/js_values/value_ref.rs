// Copied from https://github.com/napi-rs/napi-rs/blob/main/crates/napi/src/js_values/value_ref.rs
// 1. A new implementation has been added for creating a reference from raw napi_env and napi_value.
// 2. Implementation for &Ref and &mut Ref has been added to trait ToNapiValue.

#![allow(clippy::not_unsafe_ptr_arg_deref)]

use std::ptr;

use napi::{
  bindgen_prelude::ToNapiValue,
  check_status, sys,
  sys::{napi_env, napi_value},
  Result,
};

pub struct Ref {
  pub(crate) raw_ref: sys::napi_ref,
  pub(crate) count: u32,
}

#[allow(clippy::non_send_fields_in_send_ty)]
unsafe impl Send for Ref {}
unsafe impl Sync for Ref {}

impl Ref {
  pub fn new(env: napi_env, value: napi_value, ref_count: u32) -> Result<Ref> {
    let mut raw_ref = ptr::null_mut();
    assert_ne!(ref_count, 0, "Initial `ref_count` must be > 0");
    check_status!(unsafe { sys::napi_create_reference(env, value, ref_count, &mut raw_ref) })?;
    Ok(Ref {
      raw_ref,
      count: ref_count,
    })
  }

  pub fn reference(&mut self, env: napi_env) -> Result<u32> {
    check_status!(unsafe { sys::napi_reference_ref(env, self.raw_ref, &mut self.count) })?;
    Ok(self.count)
  }

  pub fn unref(&mut self, env: napi_env) -> Result<u32> {
    check_status!(unsafe { sys::napi_reference_unref(env, self.raw_ref, &mut self.count) })?;

    if self.count == 0 {
      check_status!(unsafe { sys::napi_delete_reference(env, self.raw_ref) })?;
    }
    Ok(self.count)
  }
}

#[cfg(debug_assertions)]
impl Drop for Ref {
  fn drop(&mut self) {
    debug_assert_eq!(
      self.count, 0,
      "Ref count is not equal to 0 while dropping Ref, potential memory leak"
    );
  }
}

impl ToNapiValue for &Ref {
  unsafe fn to_napi_value(env: sys::napi_env, val: Self) -> Result<sys::napi_value> {
    let mut result = ptr::null_mut();
    check_status!(
      unsafe { sys::napi_get_reference_value(env, val.raw_ref, &mut result) },
      "Failed to get reference value"
    )?;
    Ok(result)
  }
}

impl ToNapiValue for &mut Ref {
  unsafe fn to_napi_value(env: sys::napi_env, val: Self) -> Result<sys::napi_value> {
    let mut result = ptr::null_mut();
    check_status!(
      unsafe { sys::napi_get_reference_value(env, val.raw_ref, &mut result) },
      "Failed to get reference value"
    )?;
    Ok(result)
  }
}
