use std::ptr;

use napi::{
  bindgen_prelude::{FromNapiValue, Object, ToNapiValue},
  sys::{self, napi_env, napi_value},
  Env, JsValue, Status,
};
use napi_derive::napi;
use rspack_core::{diagnostics::ModuleBuildError, Compilation};
use rspack_error::{miette, Diagnostic, DiagnosticExt, Error, Result, RspackSeverity};
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
  pub error: RspackError,
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
pub struct RspackError {
  pub name: String,
  pub message: String,
  pub stack: Option<String>,
  pub module: Option<ModuleObject>,
  pub loc: Option<String>,
  pub hide_stack: Option<bool>,
  pub file: Option<String>,
  pub error: Option<Box<RspackError>>,
  /// The name of the parent error in the error chain, used to determine rendering logic.
  pub display_type: Option<String>,
}

impl napi::bindgen_prelude::TypeName for RspackError {
  fn type_name() -> &'static str {
    "RspackError"
  }
  fn value_type() -> napi::ValueType {
    napi::ValueType::Object
  }
}

impl ToNapiValue for RspackError {
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
      error,
      ..
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
    } else {
      obj.set("stack", ())?;
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
    if let Some(error) = error {
      obj.set("error", *error)?;
    }
    ToNapiValue::to_napi_value(env, obj)
  }
}

impl FromNapiValue for RspackError {
  unsafe fn from_napi_value(env: napi_env, napi_val: napi_value) -> napi::Result<RspackError> {
    #[allow(unused_variables)]
    let env_wrapper = Env::from(env);
    #[allow(unused_mut)]
    let mut obj = Object::from_napi_value(env, napi_val)?;
    let name: String = obj
      .get("name")
      .map_err(|mut err| {
        err.reason = format!("{} on {}.{}", err.reason, "RspackError", "name");
        err
      })?
      .ok_or_else(|| napi::Error::new(Status::InvalidArg, "Missing field name"))?;
    let message: String = obj
      .get("message")
      .map_err(|mut err| {
        err.reason = format!("{} on {}.{}", err.reason, "RspackError", "message");
        err
      })?
      .ok_or_else(|| napi::Error::new(Status::InvalidArg, "Missing field message"))?;
    let stack: Option<String> = obj.get("stack").map_err(|mut err| {
      err.reason = format!("{} on {}.{}", err.reason, "RspackError", "stack");
      err
    })?;
    let module: Option<ModuleObject> = obj.get("module").map_err(|mut err| {
      err.reason = format!("{} on {}.{}", err.reason, "RspackError", "module");
      err
    })?;
    let loc: Option<String> = obj.get("loc").map_err(|mut err| {
      err.reason = format!("{} on {}.{}", err.reason, "RspackError", "loc");
      err
    })?;
    let hide_stack: Option<bool> = obj.get("hideStack").map_err(|mut err| {
      err.reason = format!("{} on {}.{}", err.reason, "RspackError", "hideStack");
      err
    })?;
    let file: Option<String> = obj.get("file").map_err(|mut err| {
      err.reason = format!("{} on {}.{}", err.reason, "RspackError", "file");
      err
    })?;
    let error: Option<RspackError> = obj.get("error").map_err(|mut err| {
      err.reason = format!("{} on {}.{}", err.reason, "RspackError", "file");
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
      error: error.map(Box::new),
      display_type: None,
    };
    Ok(val)
  }
}

impl napi::bindgen_prelude::ValidateNapiValue for RspackError {}

impl std::fmt::Display for RspackError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}: ", self.name)?;
    if self.display_type.as_deref() == Some("ModuleBuildError") {
      // https://github.com/webpack/webpack/blob/93743d233ab4fa36738065ebf8df5f175323b906/lib/ModuleBuildError.js
      let message = if let Some(stack) = &self.stack {
        if self.hide_stack != Some(true) {
          stack
        } else {
          &self.message
        }
      } else {
        &self.message
      };
      write!(f, "{}", message)
    } else {
      write!(f, "{}", &self.message)
    }
  }
}

impl std::error::Error for RspackError {}

impl miette::Diagnostic for RspackError {}

impl From<&dyn miette::Diagnostic> for RspackError {
  fn from(value: &dyn miette::Diagnostic) -> Self {
    let mut name = "Error".to_string();
    if let Some(code) = value.code() {
      name = code.to_string();
    } else if let Some(severity) = value.severity() {
      name = match severity {
        miette::Severity::Advice => "Warn".to_string(),
        miette::Severity::Warning => "Warn".to_string(),
        miette::Severity::Error => "Error".to_string(),
      };
    }

    let mut message = value.to_string();
    let prefix = format!("{}: ", name);
    if message.starts_with(&prefix) {
      message = message[prefix.len()..].to_string();
    }

    Self {
      name,
      message,
      stack: None,
      module: None,
      loc: None,
      hide_stack: None,
      file: None,
      error: None,
      display_type: None,
    }
  }
}

impl RspackError {
  pub fn with_display_type<T: Into<String>>(mut self, display_type: T) -> Self {
    self.display_type = Some(display_type.into());
    self
  }

  pub fn try_from_diagnostic(
    compilation: &Compilation,
    diagnostic: &Diagnostic,
  ) -> napi::Result<Self> {
    let message = diagnostic
      .render_report(false)
      .map_err(|e| napi::Error::from_reason(e.to_string()))?;

    let mut module = None;
    if let Some(module_identifier) = diagnostic.module_identifier() {
      if let Some(m) = compilation.module_by_identifier(&module_identifier) {
        module = Some(ModuleObject::with_ref(
          m.as_ref(),
          compilation.compiler_id(),
        ));
      }
    }

    let error: Option<RspackError> = diagnostic.diagnostic_source().map(Into::into);

    Ok(Self {
      name: diagnostic.code().map(|n| n.to_string()).unwrap_or_else(|| {
        match diagnostic.severity() {
          rspack_error::RspackSeverity::Error => "Error".to_string(),
          rspack_error::RspackSeverity::Warn => "Warn".to_string(),
        }
      }),
      message,
      stack: diagnostic.stack(),
      module,
      loc: diagnostic.loc(),
      file: diagnostic.file().map(|f| f.as_str().to_string()),
      hide_stack: diagnostic.hide_stack(),
      error: error.map(Box::new),
      display_type: None,
    })
  }

  pub fn into_diagnostic(self, severity: RspackSeverity) -> Diagnostic {
    let diagnostic = if self.name == "ModuleBuildError" {
      let source = if let Some(error) = self.error {
        miette::Error::new(error.with_display_type("ModuleBuildError"))
      } else {
        miette::Error::new(miette::MietteDiagnostic::new(self.message))
      };
      Diagnostic::from(ModuleBuildError::new(source).boxed())
    } else {
      (match severity {
        RspackSeverity::Error => Diagnostic::error,
        RspackSeverity::Warn => Diagnostic::warn,
      })(self.name, self.message)
    };

    diagnostic
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
