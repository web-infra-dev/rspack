use std::{ffi::CString, fmt, ptr};

use napi::{bindgen_prelude::*, sys::napi_value, Env};

/// Either a NAPI status code or a panic code.
#[derive(Copy, Clone)]
pub enum ErrorKind {
  Napi(Status),
  Panic,
}

impl AsRef<str> for ErrorKind {
  fn as_ref(&self) -> &str {
    match self {
      ErrorKind::Napi(e) => e.as_ref(),
      ErrorKind::Panic => "Panic",
    }
  }
}

impl fmt::Display for ErrorKind {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}", self.as_ref())
  }
}
impl fmt::Debug for ErrorKind {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{self}")
  }
}

/// A wrapper around `napi::Error` that uses `ErrorKind` to represent both NAPI status code and panic code.
pub type Error = napi::Error<ErrorKind>;
pub type Result<T> = std::result::Result<T, Error>;

pub trait RspackErrorExt {
  fn into_napi_error(self) -> Error;
}

pub trait RspackResultExt<T> {
  fn into_napi_result(self) -> Result<T>;
}

impl RspackErrorExt for rspack_error::Error {
  fn into_napi_error(self) -> Error {
    match &self {
      rspack_error::Error::Panic { .. } => Error::new(ErrorKind::Panic, self.to_string()),
      _ => Error::new(ErrorKind::Napi(Status::GenericFailure), self.to_string()),
    }
  }
}

impl<T> RspackResultExt<T> for rspack_error::Result<T> {
  fn into_napi_result(self) -> Result<T> {
    self.map_err(<rspack_error::Error as RspackErrorExt>::into_napi_error)
  }
}

impl RspackErrorExt for napi::Error<Status> {
  fn into_napi_error(self) -> Error {
    Error::new(
      ErrorKind::Napi(Status::GenericFailure),
      self.reason.to_string(),
    )
  }
}

impl<T> RspackResultExt<T> for napi::Result<T> {
  fn into_napi_result(self) -> Result<T> {
    self.map_err(<napi::Error<Status> as RspackErrorExt>::into_napi_error)
  }
}

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
    match self.status {
      ErrorKind::Napi(status) => rspack_error::Error::Napi {
        status: status.to_string(),
        reason: self.reason.to_string(),
        backtrace: "".to_owned(),
      },
      ErrorKind::Panic => rspack_error::Error::Panic {
        message: self.reason.to_string(),
        backtrace: "".to_owned(),
      },
    }
  }
  fn into_rspack_error_with_detail(self, env: &Env) -> rspack_error::Error {
    match self.status {
      ErrorKind::Napi(status) => {
        let (reason, backtrace) = extract_stack_or_message_from_napi_error(env, self);
        rspack_error::Error::Napi {
          status: status.to_string(),
          reason,
          backtrace: backtrace.unwrap_or_default(),
        }
      }
      ErrorKind::Panic => rspack_error::Error::Panic {
        message: self.reason.to_string(),
        backtrace: get_backtrace(),
      },
    }
  }
}

impl NapiErrorExt for napi::Error<Status> {
  fn into_rspack_error(self) -> rspack_error::Error {
    rspack_error::Error::Napi {
      status: self.status.as_ref().to_string(),
      reason: self.reason.to_string(),
      backtrace: "".to_owned(),
    }
  }
  fn into_rspack_error_with_detail(self, env: &Env) -> rspack_error::Error {
    let status = self.status.as_ref().to_string();
    let (reason, backtrace) = extract_stack_or_message_from_napi_error(env, self.into_napi_error());
    rspack_error::Error::Napi {
      status,
      reason,
      backtrace: backtrace.unwrap_or_default(),
    }
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

impl<T: 'static> NapiResultExt<T> for napi::Result<T, Status> {
  fn into_rspack_result(self) -> rspack_error::Result<T> {
    self.map_err(|e| e.into_rspack_error())
  }
  fn into_rspack_result_with_detail(self, env: &Env) -> rspack_error::Result<T> {
    self.map_err(|e| e.into_rspack_error_with_detail(env))
  }
}

#[inline(always)]
fn get_backtrace() -> String {
  format!("{}", std::backtrace::Backtrace::force_capture())
}

/// Extract stack or message from a native Node error object,
/// otherwise we try to format the error from the given `Error` object that indicates which was created on the Rust side.
#[inline(always)]
fn extract_stack_or_message_from_napi_error(env: &Env, err: Error) -> (String, Option<String>) {
  if !err.reason.is_empty() {
    return (err.reason.clone(), None);
  }

  let stack_or_message = match unsafe { ToNapiValue::to_napi_value(env.raw(), err) } {
    Ok(napi_error) => match try_extract_string_value_from_property(env, napi_error, "stack") {
      Err(_) => match try_extract_string_value_from_property(env, napi_error, "message") {
        Err(e) => (format!("Unknown NAPI error {e:?}"), Some(get_backtrace())),
        Ok(message) => (message, Some(get_backtrace())),
      },
      Ok(message) => (message, Some(get_backtrace())),
    },
    Err(e) => (
      format!("Failed to extract NAPI error stack or message: {e}"),
      Some(get_backtrace()),
    ),
  };

  stack_or_message
}

fn try_extract_string_value_from_property<S: AsRef<str>>(
  env: &Env,
  napi_object: napi_value,
  property: S,
) -> Result<String> {
  let property = CString::new(property.as_ref()).map_err(|e| {
    Error::new(
      ErrorKind::Napi(Status::GenericFailure),
      format!("Failed to convert property to CString: {e}"),
    )
  })?;

  let mut value_ptr = ptr::null_mut();

  check_status!(
    unsafe {
      sys::napi_get_named_property(env.raw(), napi_object, property.as_ptr(), &mut value_ptr)
    },
    "Failed to get the named property from object (property: {property:?})"
  )
  .into_napi_result()?;

  let mut str_len = 0;
  check_status!(
    unsafe {
      sys::napi_get_value_string_utf8(env.raw(), value_ptr, ptr::null_mut(), 0, &mut str_len)
    },
    "Failed to get the length of the underlying property (property: {property:?})"
  )
  .into_napi_result()?;

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
  )
  .into_napi_result()?;

  let mut buf = std::mem::ManuallyDrop::new(buf);

  let buf = unsafe { Vec::from_raw_parts(buf.as_mut_ptr() as *mut u8, copied_len, copied_len) };

  Ok(String::from_utf8_lossy(&buf).into_owned())
}
