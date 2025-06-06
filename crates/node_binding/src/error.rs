use std::ptr;

use napi::{
  bindgen_prelude::{FromNapiValue, Object, ToNapiValue},
  sys::{self, napi_env, napi_value},
  Env, JsValue, Status,
};
use napi_derive::napi;
use rspack_core::Compilation;
use rspack_error::{miette, Diagnostic, Error, Result, RspackSeverity};
use rspack_napi::napi::check_status;

use crate::ModuleObject;

pub enum ErrorCode {
  Napi(napi::Status),
  Custom(String),
}

impl From<napi::Status> for ErrorCode {
  fn from(value: napi::Status) -> Self {
    Self::Napi(value)
  }
}

impl AsRef<str> for ErrorCode {
  fn as_ref(&self) -> &str {
    match self {
      ErrorCode::Napi(status) => status.as_ref(),
      ErrorCode::Custom(code) => code.as_str(),
    }
  }
}

#[napi(object, object_to_js = false)]
pub struct JsRspackDiagnostic {
  pub severity: JsRspackSeverity,
  pub error: JsRspackError,
}

impl From<JsRspackDiagnostic> for Diagnostic {
  fn from(value: JsRspackDiagnostic) -> Self {
    value.error.into_diagnostic(value.severity.into())
  }
}

#[napi(string_enum)]
pub enum JsRspackSeverity {
  Error,
  Warn,
}

impl From<JsRspackSeverity> for RspackSeverity {
  fn from(value: JsRspackSeverity) -> Self {
    match value {
      JsRspackSeverity::Error => RspackSeverity::Error,
      JsRspackSeverity::Warn => RspackSeverity::Warn,
    }
  }
}

impl From<JsRspackSeverity> for miette::Severity {
  fn from(value: JsRspackSeverity) -> Self {
    match value {
      JsRspackSeverity::Error => miette::Severity::Error,
      JsRspackSeverity::Warn => miette::Severity::Warning,
    }
  }
}

#[derive(Debug)]
pub struct JsRspackError {
  pub name: String,
  pub message: String,
  pub stack: Option<String>,
  pub module: Option<ModuleObject>,
  pub loc: Option<String>,
  pub hide_stack: Option<bool>,
  pub file: Option<String>,
}

impl napi::bindgen_prelude::TypeName for JsRspackError {
  fn type_name() -> &'static str {
    "RspackError"
  }
  fn value_type() -> napi::ValueType {
    napi::ValueType::Object
  }
}

impl ToNapiValue for JsRspackError {
  unsafe fn to_napi_value(
    env: napi::sys::napi_env,
    val: Self,
  ) -> napi::Result<napi::sys::napi_value> {
    let env_wrapper = Env::from(env);
    let Self {
      name,
      message,
      stack,
      module,
      loc,
      hide_stack,
      file,
    } = val;

    let message = env_wrapper.create_string(&message)?;
    let mut obj = ptr::null_mut();
    check_status!(unsafe {
      sys::napi_create_error(env, ptr::null_mut(), message.raw(), &mut obj)
    })?;

    let mut obj = Object::from_raw(env, obj);
    obj.set("name", name)?;
    if let Some(stack) = stack {
      obj.set("stack", stack)?;
    }
    if let Some(module) = module {
      obj.set("module", module)?;
    }
    if let Some(loc) = loc {
      obj.set("loc", loc)?;
    }
    if let Some(hide_stack) = hide_stack {
      obj.set("hideStack", hide_stack)?;
    }
    if let Some(file) = file {
      obj.set("file", file)?;
    }
    ToNapiValue::to_napi_value(env, obj)
  }
}

impl FromNapiValue for JsRspackError {
  unsafe fn from_napi_value(env: napi_env, napi_val: napi_value) -> napi::Result<JsRspackError> {
    #[allow(unused_variables)]
    let env_wrapper = Env::from(env);
    #[allow(unused_mut)]
    let mut obj = Object::from_napi_value(env, napi_val)?;
    let name: String = obj
      .get("name")
      .map_err(|mut err| {
        err.reason = format!("{} on {}.{}", err.reason, "JsRspackError", "name");
        err
      })?
      .ok_or_else(|| napi::Error::new(Status::InvalidArg, "Missing field name"))?;
    let message: String = obj
      .get("message")
      .map_err(|mut err| {
        err.reason = format!("{} on {}.{}", err.reason, "JsRspackError", "message");
        err
      })?
      .ok_or_else(|| napi::Error::new(Status::InvalidArg, "Missing field message"))?;
    let stack: Option<String> = obj.get("stack").map_err(|mut err| {
      err.reason = format!("{} on {}.{}", err.reason, "JsRspackError", "stack");
      err
    })?;
    let module: Option<ModuleObject> = obj.get("module").map_err(|mut err| {
      err.reason = format!("{} on {}.{}", err.reason, "JsRspackError", "module");
      err
    })?;
    let loc: Option<String> = obj.get("loc").map_err(|mut err| {
      err.reason = format!("{} on {}.{}", err.reason, "JsRspackError", "loc");
      err
    })?;
    let hide_stack: Option<bool> = obj.get("hideStack").map_err(|mut err| {
      err.reason = format!("{} on {}.{}", err.reason, "JsRspackError", "hideStack");
      err
    })?;
    let file: Option<String> = obj.get("file").map_err(|mut err| {
      err.reason = format!("{} on {}.{}", err.reason, "JsRspackError", "file");
      err
    })?;
    let val = Self {
      name,
      message,
      stack,
      module,
      loc,
      hide_stack,
      file,
    };
    Ok(val)
  }
}

impl napi::bindgen_prelude::ValidateNapiValue for JsRspackError {}

impl JsRspackError {
  pub fn try_from_diagnostic(
    compilation: &Compilation,
    diagnostic: &Diagnostic,
  ) -> napi::Result<Self> {
    let mut module = None;
    if let Some(module_identifier) = diagnostic.module_identifier() {
      if let Some(m) = compilation.module_by_identifier(&module_identifier) {
        module = Some(ModuleObject::with_ref(
          m.as_ref(),
          compilation.compiler_id(),
        ));
      }
    }

    Ok(Self {
      name: diagnostic.code().map(|n| n.to_string()).unwrap_or_else(|| {
        match diagnostic.severity() {
          rspack_error::RspackSeverity::Error => "Error".to_string(),
          rspack_error::RspackSeverity::Warn => "Warn".to_string(),
        }
      }),
      message: diagnostic.message(),
      stack: diagnostic.stack(),
      module,
      loc: diagnostic.loc(),
      file: diagnostic.file().map(|f| f.as_str().to_string()),
      hide_stack: diagnostic.hide_stack(),
    })
  }

  pub fn into_diagnostic(self, severity: RspackSeverity) -> Diagnostic {
    (match severity {
      RspackSeverity::Error => Diagnostic::error,
      RspackSeverity::Warn => Diagnostic::warn,
    })(self.name, self.message)
    .with_file(self.file.map(Into::into))
    .with_module_identifier(self.module.map(|module| *module.identifier()))
    .with_stack(self.stack)
    .with_hide_stack(self.hide_stack)
  }
}

pub trait RspackResultToNapiResultExt<T, E, S: AsRef<str> = napi::Status> {
  fn to_napi_result(self) -> napi::Result<T, S>;
  fn to_napi_result_with_message(self, f: impl FnOnce(E) -> String) -> napi::Result<T, S>;
}

impl<T, E: ToString> RspackResultToNapiResultExt<T, E> for Result<T, E> {
  fn to_napi_result(self) -> napi::Result<T> {
    self.map_err(|e| napi::Error::from_reason(e.to_string()))
  }
  fn to_napi_result_with_message(self, f: impl FnOnce(E) -> String) -> napi::Result<T> {
    self.map_err(|e| napi::Error::from_reason(f(e)))
  }
}

impl<T> RspackResultToNapiResultExt<T, Error, ErrorCode> for Result<T, Error> {
  fn to_napi_result(self) -> napi::Result<T, ErrorCode> {
    self.map_err(|e| {
      napi::Error::new(
        e.code()
          .map(|code| ErrorCode::Custom(code.to_string()))
          .unwrap_or_else(|| ErrorCode::Napi(napi::Status::GenericFailure)),
        e.to_string(),
      )
    })
  }
  fn to_napi_result_with_message(
    self,
    f: impl FnOnce(Error) -> String,
  ) -> napi::Result<T, ErrorCode> {
    self.map_err(|e| {
      napi::Error::new(
        e.code()
          .map(|code| ErrorCode::Custom(code.to_string()))
          .unwrap_or_else(|| ErrorCode::Napi(napi::Status::GenericFailure)),
        f(e),
      )
    })
  }
}
