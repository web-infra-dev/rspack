use std::{
  fmt::Display,
  ptr::{self, NonNull},
};

use napi::{
  bindgen_prelude::{FromNapiValue, Object, ToNapiValue},
  sys::{self, napi_env, napi_value},
  Env, JsValue, Status,
};
use napi_derive::napi;
use rspack_core::{diagnostics::ModuleBuildError, Compilation};
use rspack_error::{
  miette::{self, Severity},
  Diagnostic, DiagnosticExt, Error, Result, RspackSeverity,
};
use rspack_napi::napi::check_status;

use crate::{DependencyLocation, ModuleObject};

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
  pub details: Option<String>,
  pub stack: Option<String>,
  pub module: Option<ModuleObject>,
  pub loc: Option<DependencyLocation>,
  pub hide_stack: Option<bool>,
  pub file: Option<String>,
  pub error: Option<Box<RspackError>>,
  // Only used for display on the Rust side; this value is set when converting to a Diagnostic struct.
  pub severity: Option<Severity>,
  // Only used for display on the Rust side; the name of the parent error in the error chain, used to determine rendering logic.
  pub parent_error_name: Option<String>,
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
      details,
      stack,
      module,
      loc,
      hide_stack,
      file,
      error,
      ..
    } = val;

    let js_string = env_wrapper.create_string(&message)?;
    let mut obj = ptr::null_mut();
    check_status!(unsafe {
      sys::napi_create_error(env, ptr::null_mut(), js_string.raw(), &mut obj)
    })?;

    let mut obj = Object::from_raw(env, obj);
    obj.set("name", name)?;
    if let Some(details) = details {
      obj.set("details", details)?;
    }
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
    let obj = Object::from_napi_value(env, napi_val)?;
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

    let details: Option<String> = obj.get("details").ok().flatten();
    let stack: Option<String> = obj.get("stack").ok().flatten();
    let module: Option<ModuleObject> = obj.get("module").ok().flatten();
    let loc: Option<DependencyLocation> = obj.get("loc").ok().flatten();
    let hide_stack: Option<bool> = obj.get("hideStack").ok().flatten();
    let file: Option<String> = obj.get("file").ok().flatten();
    let error: Option<RspackError> = obj.get("error").ok().flatten();

    let val = Self {
      severity: None,
      name,
      message,
      details,
      stack,
      module,
      loc,
      hide_stack,
      file,
      error: error.map(Box::new),
      parent_error_name: None,
    };
    Ok(val)
  }
}

impl napi::bindgen_prelude::ValidateNapiValue for RspackError {}

// The error printing logic here is consistent with Webpack Stats.
impl std::fmt::Display for RspackError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    if self.parent_error_name.as_deref() == Some("ModuleBuildError") {
      // https://github.com/webpack/webpack/blob/93743d233ab4fa36738065ebf8df5f175323b906/lib/ModuleBuildError.js
      let message = if let Some(stack) = &self.stack {
        write!(f, "{}: ", self.name)?;
        if self.hide_stack != Some(true) {
          stack
        } else {
          &self.message
        }
      } else {
        &self.message
      };
      write!(f, "{message}")
    } else {
      if let Some(stack) = &self.stack
        && self.hide_stack != Some(true)
      {
        write!(f, "{stack}")?;
      } else {
        write!(f, "{}: ", self.name)?;
        write!(f, "{}", &self.message)?;
      }
      Ok(())
    }
  }
}

impl std::error::Error for RspackError {}

impl miette::Diagnostic for RspackError {
  fn code<'a>(&'a self) -> Option<Box<dyn Display + 'a>> {
    Some(Box::new(&self.name))
  }

  fn severity(&self) -> Option<Severity> {
    self.severity
  }
}

impl From<&dyn miette::Diagnostic> for RspackError {
  fn from(value: &dyn miette::Diagnostic) -> Self {
    let mut name = "Error".to_string();
    if let Some(code) = value.code() {
      name = code.to_string();
    }

    let mut message = value.to_string();
    let prefix = format!("{name}: ");
    if message.starts_with(&prefix) {
      message = message[prefix.len()..].to_string();
    }

    Self {
      severity: None,
      name,
      message,
      details: None,
      stack: None,
      module: None,
      loc: None,
      hide_stack: None,
      file: None,
      error: None,
      parent_error_name: None,
    }
  }
}

impl RspackError {
  pub fn with_parent_error_name<T: Into<String>>(mut self, parent_error_name: T) -> Self {
    self.parent_error_name = Some(parent_error_name.into());
    self
  }

  pub fn try_from_top_level_diagnostic(
    compilation: &Compilation,
    diagnostic: &Diagnostic,
  ) -> napi::Result<Self> {
    let message = diagnostic.message();

    let mut module = None;
    if let Some(module_identifier) = diagnostic.module_identifier() {
      if let Some(m) = compilation.module_by_identifier(&module_identifier) {
        module =
          Some(ModuleObject::with_ptr(
            #[allow(clippy::unwrap_used)]
            NonNull::new(
              m.as_ref() as *const dyn rspack_core::Module as *mut dyn rspack_core::Module
            )
            .unwrap(),
            compilation.compiler_id(),
          ));
      }
    }

    let error: Option<RspackError> = diagnostic.diagnostic_source().map(Into::into);

    Ok(Self {
      severity: None,
      name: diagnostic
        .code()
        .map(|n| n.to_string())
        .unwrap_or_else(|| "Error".to_string()),
      message,
      details: diagnostic.details(),
      stack: diagnostic.stack(),
      module,
      loc: diagnostic.loc().map(Into::into),
      file: diagnostic.file().map(|f| f.as_str().to_string()),
      hide_stack: diagnostic.hide_stack(),
      error: error.map(Box::new),
      parent_error_name: None,
    })
  }

  pub fn into_diagnostic(mut self, severity: RspackSeverity) -> Diagnostic {
    self.severity = Some(severity.into());

    let message = self.message.clone();
    let details = self.details.clone();
    let file = self.file.clone();
    let loc = self.loc.as_ref().map(Into::into);
    let module = self.module.as_ref().map(|module| *module.identifier());
    let stack = self.stack.clone();
    let hide_stack = self.hide_stack;

    let diagnostic = if self.name == "ModuleBuildError" {
      let source = if let Some(error) = self.error {
        miette::Error::new(error.with_parent_error_name("ModuleBuildError"))
      } else {
        miette::Error::new(miette::MietteDiagnostic::new(self.message))
      };
      Diagnostic::from(ModuleBuildError::new(source).boxed())
    } else {
      Diagnostic::from(self.boxed())
    };

    diagnostic
      .with_message(message)
      .with_details(details)
      .with_file(file.map(Into::into))
      .with_loc(loc)
      .with_module_identifier(module)
      .with_stack(stack)
      .with_hide_stack(hide_stack)
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
