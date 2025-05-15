use derive_more::Debug;
use napi_derive::napi;
use rspack_error::{miette, Diagnostic, Result, RspackSeverity};

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

#[derive(Debug)]
pub struct RspackError {
  pub name: String,
  pub message: String,
  pub module_identifier: Option<String>,
  pub loc: Option<String>,
  pub file: Option<String>,
  pub stack: Option<String>,
  pub hide_stack: Option<bool>,
  pub module: Option<ModuleObject>,
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
    #[allow(unused_variables)]
    let env_wrapper = napi::bindgen_prelude::Env::from(env);
    #[allow(unused_mut)]
    let mut obj = env_wrapper.create_object()?;
    let Self {
      name,
      message,
      module_identifier,
      loc,
      file,
      stack,
      hide_stack,
      module,
    } = val;
    obj.set("name", name)?;
    obj.set("message", message)?;
    if module_identifier.is_some() {
      obj.set("moduleIdentifier", module_identifier)?;
    }
    if loc.is_some() {
      obj.set("loc", loc)?;
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
    napi::bindgen_prelude::Object::to_napi_value(env, obj)
  }
}

impl napi::bindgen_prelude::FromNapiValue for RspackError {
  unsafe fn from_napi_value(
    env: napi::bindgen_prelude::sys::napi_env,
    napi_val: napi::bindgen_prelude::sys::napi_value,
  ) -> napi::bindgen_prelude::Result<RspackError> {
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
    let loc: Option<String> = obj.get("loc").map_err(|mut err| {
      err.reason = format!("{} on {}.{}", err.reason, "RspackError", "loc");
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
      module_identifier,
      loc,
      file,
      stack,
      hide_stack,
      module: None,
    };
    Ok(val)
  }
}

impl napi::bindgen_prelude::ValidateNapiValue for RspackError {}

impl RspackError {
  pub fn try_from_diagnostic(
    compilation: &rspack_core::Compilation,
    diagnostic: &Diagnostic,
    colored: bool,
  ) -> Result<Self> {
    println!("diagnostic {:#?}", diagnostic);
    Ok(Self {
      name: diagnostic.code().map(|n| n.to_string()).unwrap_or_else(|| {
        match diagnostic.severity() {
          rspack_error::RspackSeverity::Error => "Error".to_string(),
          rspack_error::RspackSeverity::Warn => "Warn".to_string(),
        }
      }),
      message: diagnostic.render_report(colored)?,
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
    })
  }

  pub fn into_diagnostic(self, severity: RspackSeverity) -> Diagnostic {
    (match severity {
      RspackSeverity::Error => Diagnostic::error,
      RspackSeverity::Warn => Diagnostic::warn,
    })(self.name, self.message)
    .with_file(self.file.map(Into::into))
    .with_module_identifier(self.module_identifier.map(Into::into))
    .with_stack(self.stack)
    .with_hide_stack(self.hide_stack)
  }
}

impl std::fmt::Display for RspackError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    if self.name.is_empty() {
      write!(f, "{}", self.message)
    } else {
      write!(f, "{}: {}", self.name, self.message)
    }
  }
}

impl std::error::Error for RspackError {}

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
