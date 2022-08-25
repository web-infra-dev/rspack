use std::ffi::CStr;
use std::fmt::Write as FmtWrite;
use std::io::Write as IoWrite;
use std::ptr;

use napi::{check_status, Env, Error, Result};
use napi_derive::napi;
use napi_sys::{napi_env, napi_value};
use once_cell::sync::OnceCell;

static CUSTOM_TRACE_SUBSCRIBER: OnceCell<bool> = OnceCell::new();

pub fn get_named_property_value_string(
  env_ptr: napi_env,
  object_ptr: napi_value,
  property_name: &str,
) -> Result<String> {
  let mut bytes_with_nul: Vec<u8> = Vec::with_capacity(property_name.len() + 1);

  std::write!(&mut bytes_with_nul, "{}", property_name)?;
  std::write!(&mut bytes_with_nul, "{}", '\0')?;

  let mut value_ptr = ptr::null_mut();

  check_status!(
    unsafe {
      napi_sys::napi_get_named_property(
        env_ptr,
        object_ptr,
        CStr::from_bytes_with_nul_unchecked(&bytes_with_nul).as_ptr(),
        &mut value_ptr,
      )
    },
    "failed to get the value"
  )?;

  let mut str_len = 0;
  check_status!(
    unsafe {
      napi_sys::napi_get_value_string_utf8(env_ptr, value_ptr, ptr::null_mut(), 0, &mut str_len)
    },
    "failed to get the value"
  )?;

  str_len += 1;
  let mut buf = Vec::with_capacity(str_len);
  let mut copied_len = 0;

  check_status!(
    unsafe {
      napi_sys::napi_get_value_string_utf8(
        env_ptr,
        value_ptr,
        buf.as_mut_ptr(),
        str_len,
        &mut copied_len,
      )
    },
    "failed to get the value"
  )?;

  // Vec<i8> -> Vec<u8> See: https://stackoverflow.com/questions/59707349/cast-vector-of-i8-to-vector-of-u8-in-rust
  let mut buf = std::mem::ManuallyDrop::new(buf);

  let buf = unsafe { Vec::from_raw_parts(buf.as_mut_ptr() as *mut u8, copied_len, copied_len) };

  String::from_utf8(buf).map_err(|_| Error::from_reason("failed to get property"))
}

#[napi]
pub fn init_custom_trace_subscriber(
  mut env: Env,
  // trace_out_file_path: Option<String>,
) -> Result<()> {
  CUSTOM_TRACE_SUBSCRIBER.get_or_init(|| {
    let guard = rspack_core::log::enable_tracing_by_env_with_chrome_layer();
    if let Some(guard) = guard {
      env
        .add_env_cleanup_hook(guard, |flush_guard| {
          flush_guard.flush();
          drop(flush_guard);
        })
        .expect("Should able to initialize cleanup for custom trace subscriber");
    }
    true
  });

  Ok(())
}
