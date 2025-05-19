use std::ops::Deref;

use derive_more::Debug;
use napi::bindgen_prelude::ToNapiValue;
use napi_derive::napi;
use rspack_error::{
  miette::{self, Severity},
  Diagnostic, Result, RspackSeverity,
};

use crate::ModuleObject;

#[napi(object)]
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

#[derive(Debug, Clone)]
pub struct RspackError {
  pub name: String,
  pub message: String,
  pub severity: Option<Severity>,
  // TODO: Consider removing `module_identifier` if it is no longer needed.
  pub module_identifier: Option<String>,
  pub module: Option<ModuleObject>,
  pub loc: Option<String>,
  pub file: Option<String>,
  pub stack: Option<String>,
  pub hide_stack: Option<bool>,
  pub error: Option<Box<RspackError>>,
}

impl napi::bindgen_prelude::TypeName for RspackError {
  fn type_name() -> &'static str {
    "RspackError"
  }
  fn value_type() -> napi::ValueType {
    napi::ValueType::Object
  }
}

#[automatically_derived]
impl napi::bindgen_prelude::ToNapiValue for RspackError {
  unsafe fn to_napi_value(
    env: napi::bindgen_prelude::sys::napi_env,
    val: RspackError,
  ) -> napi::bindgen_prelude::Result<napi::bindgen_prelude::sys::napi_value> {
    let env_wrapper = napi::bindgen_prelude::Env::from(env);
    let Self {
      name,
      message,
      severity,
      module_identifier,
      loc,
      file,
      stack,
      hide_stack,
      module,
      error,
    } = val;
    let mut obj = env_wrapper.create_error(napi::Error::from_reason(message))?;
    obj.set("name", name)?;
    if module_identifier.is_some() {
      obj.set("moduleIdentifier", module_identifier)?;
    }
    if let Some(loc) = loc {
      obj.set("loc", ToNapiValue::to_napi_value(env, loc)?)?;
    }
    if file.is_some() {
      obj.set("file", file)?;
    }
    if stack.is_some() {
      obj.set("stack", stack)?;
    }
    if hide_stack.is_some() {
      obj.set("hideStack", hide_stack)?;
    }
    if hide_stack.is_some() {
      obj.set("hideStack", hide_stack)?;
    }
    if module.is_some() {
      obj.set("module", module)?;
    }
    if let Some(error) = error {
      obj.set("error", *error)?;
    }
    napi::bindgen_prelude::Object::to_napi_value(env, obj)
  }
}

impl napi::bindgen_prelude::FromNapiValue for RspackError {
  unsafe fn from_napi_value(
    env: napi::bindgen_prelude::sys::napi_env,
    napi_val: napi::bindgen_prelude::sys::napi_value,
  ) -> napi::bindgen_prelude::Result<RspackError> {
    let unknown = napi::bindgen_prelude::Unknown::from_napi_value(env, napi_val)?;
    if !unknown.is_error()? {
      let error = unknown.coerce_to_string()?.into_utf8()?.into_owned()?;
      return Ok(RspackError {
        name: "NonErrorEmittedError".to_string(),
        message: format!("(Emitted value instead of an instance of Error) {}", error),
        severity: None,
        module_identifier: None,
        loc: None,
        file: None,
        stack: None,
        hide_stack: None,
        module: None,
        error: None,
      });
    }

    let obj = napi::bindgen_prelude::Object::from_napi_value(env, napi_val)?;
    let name: String = obj
      .get("name")
      .map_err(|mut err| {
        err.reason = format!("{} on {}.{}", err.reason, "RspackError", "name");
        err
      })?
      .ok_or_else(|| {
        napi::bindgen_prelude::Error::new(
          napi::bindgen_prelude::Status::InvalidArg,
          "Missing field `name`",
        )
      })?;
    let message: String = obj
      .get("message")
      .map_err(|mut err| {
        err.reason = format!("{} on {}.{}", err.reason, "RspackError", "message");
        err
      })?
      .ok_or_else(|| {
        napi::bindgen_prelude::Error::new(
          napi::bindgen_prelude::Status::InvalidArg,
          "Missing field `message`",
        )
      })?;
    let module_identifier: Option<String> = obj.get("moduleIdentifier").map_err(|mut err| {
      err.reason = format!("{} on {}.{}", err.reason, "RspackError", "moduleIdentifier");
      err
    })?;
    let file: Option<String> = obj.get("file").map_err(|mut err| {
      err.reason = format!("{} on {}.{}", err.reason, "RspackError", "file");
      err
    })?;
    let stack: Option<String> = obj.get("stack").map_err(|mut err| {
      err.reason = format!("{} on {}.{}", err.reason, "RspackError", "stack");
      err
    })?;
    let hide_stack: Option<bool> = obj.get("hideStack").map_err(|mut err| {
      err.reason = format!("{} on {}.{}", err.reason, "RspackError", "hideStack");
      err
    })?;
    let val = Self {
      name,
      message,
      severity: None,
      module_identifier,
      // TODO: Currently, Rspack does not handle `loc` from JavaScript very well.
      loc: None,
      file,
      stack,
      hide_stack,
      module: None,
      error: None,
    };
    Ok(val)
  }
}

impl napi::bindgen_prelude::ValidateNapiValue for RspackError {}

impl RspackError {
  pub fn try_from_diagnostic(
    compilation: &rspack_core::Compilation,
    diagnostic: &Diagnostic,
  ) -> napi::Result<Self> {
    let error = match diagnostic.source() {
      Some(source) => match source.downcast_ref::<RspackError>() {
        Some(rspack_error) => Some(Box::new(rspack_error.clone())),
        None => Some(Box::new(RspackError {
          name: "Error".to_string(),
          message: format!("{}", source),
          severity: None,
          module_identifier: None,
          loc: None,
          file: None,
          stack: None,
          hide_stack: None,
          module: None,
          error: None,
        })),
      },
      None => None,
    };

    if let Some(rspack_error) =
      (diagnostic.deref() as &dyn std::error::Error).downcast_ref::<RspackError>()
    {
      return Ok(rspack_error.clone());
    }

    let message = diagnostic
      .render_report(compilation.options.stats.colors)
      .map_err(|e| napi::Error::from_reason(format!("{}", e)))?;

    Ok(Self {
      name: diagnostic.code().map(|n| n.to_string()).unwrap_or_else(|| {
        match diagnostic.severity() {
          rspack_error::RspackSeverity::Error => "Error".to_string(),
          rspack_error::RspackSeverity::Warn => "Warn".to_string(),
        }
      }),
      message,
      severity: None,
      module_identifier: diagnostic.module_identifier().map(|d| d.to_string()),
      loc: diagnostic.loc(),
      file: diagnostic.file().map(|f| f.as_str().to_string()),
      stack: diagnostic.stack(),
      hide_stack: diagnostic.hide_stack(),
      module: match diagnostic.module_identifier() {
        Some(identifier) => compilation
          .module_by_identifier(&identifier)
          .map(|module| ModuleObject::with_ref(module.as_ref(), compilation.compiler_id())),
        None => None,
      },
      error,
    })
  }

  pub fn into_diagnostic(mut self, severity: RspackSeverity) -> Diagnostic {
    self.severity = Some(severity.into());
    let file = self.file.clone();
    let module_identifier = self.module.as_ref().map(|module| module.identifier());
    let stack = self.stack.clone();
    let hide_stack = self.hide_stack;
    Diagnostic::from(Box::new(self) as Box<dyn miette::Diagnostic + Send + Sync>)
      .with_file(file.map(Into::into))
      .with_module_identifier(module_identifier)
      // Used in
      .with_stack(stack)
      .with_hide_stack(hide_stack)
  }
}

impl std::fmt::Display for RspackError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    if let Some(stack) = &self.stack {
      if !matches!(self.hide_stack, Some(true)) {
        write!(f, "{}", stack)?;
      }
    } else if !self.name.is_empty() {
      write!(f, "{}: ", self.name)?;
    }
    write!(f, "{}", self.message)
  }
}

impl std::error::Error for RspackError {}

impl miette::Diagnostic for RspackError {
  fn code(&self) -> Option<Box<dyn std::fmt::Display>> {
    Some(Box::new(self.name.clone()))
  }

  fn severity(&self) -> Option<Severity> {
    self.severity
  }
}

pub trait RspackResultToNapiResultExt<T, E: ToString> {
  fn to_napi_result(self) -> napi::Result<T>;
  fn to_napi_result_with_message(self, f: impl FnOnce(E) -> String) -> napi::Result<T>;
}

impl<T, E: ToString> RspackResultToNapiResultExt<T, E> for Result<T, E> {
  fn to_napi_result(self) -> napi::Result<T> {
    self.map_err(|e| napi::Error::from_reason(e.to_string()))
  }
  fn to_napi_result_with_message(self, f: impl FnOnce(E) -> String) -> napi::Result<T> {
    self.map_err(|e| napi::Error::from_reason(f(e)))
  }
}
