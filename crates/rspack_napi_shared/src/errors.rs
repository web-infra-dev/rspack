use std::{ffi::CString, ptr};

use napi::{bindgen_prelude::*, sys::napi_value, Env, Error, Result};
use rspack_error::{
  miette::{self, Diagnostic},
  thiserror::{self, Error},
};

#[derive(Debug, Error, Diagnostic)]
#[diagnostic()]
#[error("{0}\n{1}")]
struct NodeError(String, String);

pub trait NapiErrorExt {
  fn into_rspack_error(self) -> rspack_error::Error;
  fn into_rspack_error_with_detail(self, env: &Env) -> rspack_error::Error;
}

pub trait NapiResultExt<T> {
  fn into_rspack_result(self) -> rspack_error::Result<T>;
  fn into_rspack_result_with_detail(self, env: &Env) -> rspack_error::Result<T>;
}

impl NapiErrorExt for Error {
  fn into_rspack_error(self) -> rspack_error::Error {
    NodeError(self.reason, "".to_string()).into()
  }
  fn into_rspack_error_with_detail(self, env: &Env) -> rspack_error::Error {
    let (reason, backtrace) = extract_stack_or_message_from_napi_error(env, self);
    NodeError(reason, backtrace.unwrap_or_default()).into()
  }
}

impl<T: 'static> NapiResultExt<T> for Result<T> {
  fn into_rspack_result(self) -> rspack_error::Result<T> {
    self.map_err(|e| e.into_rspack_error())
  }
  fn into_rspack_result_with_detail(self, env: &Env) -> rspack_error::Result<T> {
    self.map_err(|e| e.into_rspack_error_with_detail(env))
  }
}

const fn get_backtrace() -> Option<String> {
  None
}

/// Extract stack or message from a native Node error object,
/// otherwise we try to format the error from the given `Error` object that indicates which was created on the Rust side.
#[inline(always)]
fn extract_stack_or_message_from_napi_error(env: &Env, err: Error) -> (String, Option<String>) {
  let maybe_reason = err.reason.clone();
  match unsafe { ToNapiValue::to_napi_value(env.raw(), err) } {
    Ok(napi_error) => match try_extract_string_value_from_property(env, napi_error, "stack") {
      Err(_) => match try_extract_string_value_from_property(env, napi_error, "message") {
        Err(e) => (format!("Unknown NAPI error {e}"), get_backtrace()),
        Ok(message) => (message, get_backtrace()),
      },
      Ok(message) => (message, get_backtrace()),
    },
    Err(e) if maybe_reason.is_empty() => (
      format!("Failed to extract NAPI error stack or message: {e}"),
      get_backtrace(),
    ),
    Err(_) => (maybe_reason, None),
  }
}

fn try_extract_string_value_from_property<S: AsRef<str>>(
  env: &Env,
  napi_object: napi_value,
  property: S,
) -> napi::Result<String> {
  let property = CString::new(property.as_ref())?;

  let mut value_ptr = ptr::null_mut();

  check_status!(
    unsafe {
      sys::napi_get_named_property(env.raw(), napi_object, property.as_ptr(), &mut value_ptr)
    },
    "Failed to get the named property from object (property: {property:?})"
  )?;

  let mut str_len = 0;
  check_status!(
    unsafe {
      sys::napi_get_value_string_utf8(env.raw(), value_ptr, ptr::null_mut(), 0, &mut str_len)
    },
    "Failed to get the length of the underlying property (property: {property:?})"
  )?;

  str_len += 1;
  let mut buf = Vec::with_capacity(str_len);
  let mut copied_len = 0;

  check_status!(
    unsafe {
      sys::napi_get_value_string_utf8(
        env.raw(),
        value_ptr,
        buf.as_mut_ptr(),
        str_len,
        &mut copied_len,
      )
    },
    "Failed to get value of the property (property: {property:?})"
  )?;

  let mut buf = std::mem::ManuallyDrop::new(buf);

  let buf = unsafe { Vec::from_raw_parts(buf.as_mut_ptr() as *mut u8, copied_len, copied_len) };

  Ok(String::from_utf8_lossy(&buf).into_owned())
}
