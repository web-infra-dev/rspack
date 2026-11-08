// Copied from https://github.com/napi-rs/napi-rs/blob/main/crates/napi/src/js_values/value_ref.rs
// 1. A new implementation has been added for creating a reference from raw napi_env and napi_value.
// 2. Implementation for &Ref and &mut Ref has been added to trait ToNapiValue.

#![allow(clippy::not_unsafe_ptr_arg_deref)]

use std::ptr;

use napi::{Result, bindgen_prelude::ToNapiValue, check_status, sys};

pub struct Ref {
  pub(crate) raw_ref: sys::napi_ref,
  pub(crate) count: u32,
}

#[allow(clippy::non_send_fields_in_send_ty)]
unsafe impl Send for Ref {}
unsafe impl Sync for Ref {}

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
