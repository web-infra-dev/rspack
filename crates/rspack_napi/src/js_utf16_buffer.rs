use std::ptr;

use napi::bindgen_prelude::*;

// This struct is designed for the process of transferring strings from Rust to JavaScript.
//
// The struct stores UTF-16 encoded strings directly for direct use on the JavaScript side.
// The conversion from UTF-8 to UTF-16 can be performed in another thread to avoid blocking the JavaScript thread.
pub struct JsUtf16Buffer(Vec<u16>);

impl JsUtf16Buffer {
  pub fn new(buffer: Vec<u16>) -> Self {
    Self(buffer)
  }

  pub fn to_string(&self) -> String {
    String::from_utf16(&self.0).unwrap()
  }
}

impl From<&str> for JsUtf16Buffer {
  fn from(value: &str) -> Self {
    Self(value.encode_utf16().collect::<Vec<_>>())
  }
}

impl From<&String> for JsUtf16Buffer {
  fn from(value: &String) -> Self {
    Self(value.encode_utf16().collect::<Vec<_>>())
  }
}

impl ToNapiValue for JsUtf16Buffer {
  unsafe fn to_napi_value(env: sys::napi_env, val: Self) -> Result<sys::napi_value> {
    let mut ptr = ptr::null_mut();

    check_status!(
      unsafe {
        sys::napi_create_string_utf16(
          env,
          val.0.as_ptr() as *const _,
          val.0.len() as isize,
          &mut ptr,
        )
      },
      "Failed to convert napi `string` into rust type `JsUtf16Buffer`"
    )?;

    Ok(ptr)
  }
}

impl FromNapiValue for JsUtf16Buffer {
  unsafe fn from_napi_value(env: sys::napi_env, napi_val: sys::napi_value) -> Result<Self> {
    let mut len = 0;

    check_status!(
      unsafe { sys::napi_get_value_string_utf16(env, napi_val, ptr::null_mut(), 0, &mut len) },
      "Failed to convert napi `utf16 buffer` into rust type `String`",
    )?;

    // end char len in C
    len += 1;
    let mut ret = vec![0; len];
    let mut written_char_count = 0;

    check_status!(
      unsafe {
        sys::napi_get_value_string_utf16(
          env,
          napi_val,
          ret.as_mut_ptr(),
          len,
          &mut written_char_count,
        )
      },
      "Failed to convert napi `utf16 string` into rust type `String`",
    )?;

    ret.pop();

    Ok(JsUtf16Buffer(ret))
  }
}

impl TypeName for JsUtf16Buffer {
  fn type_name() -> &'static str {
    "String(buffer)"
  }

  fn value_type() -> ValueType {
    ValueType::String
  }
}

impl ValidateNapiValue for JsUtf16Buffer {}
