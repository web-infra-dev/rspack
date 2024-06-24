use napi_derive::napi;
use rspack_error::{Diagnostic, Result, RspackSeverity};

#[napi(object)]
pub struct JsDiagnostic {
  pub severity: JsRspackSeverity,
  pub error: JsRspackError,
}

impl From<JsDiagnostic> for Diagnostic {
  fn from(value: JsDiagnostic) -> Self {
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

#[napi(object)]
pub struct JsRspackError {
  pub name: String,
  pub message: String,
  pub module_identifier: Option<String>,
  pub file: Option<String>,
}

impl JsRspackError {
  pub fn try_from_diagnostic(diagnostic: &Diagnostic, colored: bool) -> Result<Self> {
    Ok(Self {
      name: diagnostic.code().map(|n| n.to_string()).unwrap_or_else(|| {
        match diagnostic.severity() {
          rspack_error::RspackSeverity::Error => "Error".to_string(),
          rspack_error::RspackSeverity::Warn => "Warn".to_string(),
        }
      }),
      message: diagnostic.render_report(colored)?,
      module_identifier: diagnostic.module_identifier().map(|d| d.to_string()),
      file: diagnostic.file().map(|f| f.to_string_lossy().to_string()),
    })
  }

  pub fn into_diagnostic(self, severity: RspackSeverity) -> Diagnostic {
    (match severity {
      RspackSeverity::Error => Diagnostic::error,
      RspackSeverity::Warn => Diagnostic::warn,
    })(self.name, self.message)
    .with_file(self.file.map(Into::into))
    .with_module_identifier(self.module_identifier.map(Into::into))
  }
}
