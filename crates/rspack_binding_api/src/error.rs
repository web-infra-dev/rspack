use napi_derive::napi;
use rspack_error::{miette, Diagnostic, Error, Result, RspackSeverity};

use crate::DependencyLocation;

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

#[napi(object)]
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

#[napi(object)]
#[derive(Debug)]
pub struct JsRspackError {
  pub name: String,
  pub message: String,
  pub module_identifier: Option<String>,
  pub loc: Option<DependencyLocation>,
  pub file: Option<String>,
  pub stack: Option<String>,
  pub hide_stack: Option<bool>,
}

impl JsRspackError {
  pub fn try_from_diagnostic(diagnostic: &Diagnostic, colored: bool) -> napi::Result<Self> {
    let message = diagnostic
      .render_report(colored)
      .map_err(|e| napi::Error::from_reason(e.to_string()))?;

    Ok(Self {
      name: diagnostic.code().map(|n| n.to_string()).unwrap_or_else(|| {
        match diagnostic.severity() {
          rspack_error::RspackSeverity::Error => "Error".to_string(),
          rspack_error::RspackSeverity::Warn => "Warn".to_string(),
        }
      }),
      message,
      module_identifier: diagnostic.module_identifier().map(|d| d.to_string()),
      loc: diagnostic.loc().map(Into::into),
      file: diagnostic.file().map(|f| f.as_str().to_string()),
      stack: diagnostic.stack(),
      hide_stack: diagnostic.hide_stack(),
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
