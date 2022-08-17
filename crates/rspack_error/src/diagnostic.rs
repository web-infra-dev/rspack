use crate::{Error, TraceableError};

#[derive(Debug, Clone)]
pub enum Severity {
  Error,
  Warn,
}
#[derive(Debug, Clone)]
pub struct DiagnosticSourceInfo {
  pub(crate) path: String,
  pub(crate) source: String,
}
#[derive(Debug, Clone)]
pub struct Diagnostic {
  pub severity: Severity,
  pub message: String,
  /// Source code and path of current Diagnostic
  pub source_info: Option<DiagnosticSourceInfo>,
  pub start: usize,
  pub end: usize,
}

impl Diagnostic {
  pub fn warn(message: String, start: usize, end: usize) -> Self {
    Self {
      severity: Severity::Warn,
      message,
      source_info: None,
      start,
      end,
    }
  }

  pub fn error(message: String, start: usize, end: usize) -> Self {
    Self {
      severity: Severity::Error,
      message,
      source_info: None,
      start,
      end,
    }
  }

  pub fn with_source_info(mut self, source: DiagnosticSourceInfo) -> Self {
    self.source_info = Some(source);
    self
  }
}

impl From<Error> for Diagnostic {
  fn from(err: Error) -> Self {
    match err {
      Error::InternalError(message) => Self {
        severity: Severity::Error,
        message,
        source_info: None,
        start: 0,
        end: 0,
      },
      Error::TraceableError(TraceableError {
        path,
        start,
        end,
        error_message,
        source,
      }) => {
        let source = if let Some(source) = source {
          source
        } else {
          std::fs::read_to_string(&path).unwrap()
        };
        Self {
          severity: Severity::Error,
          message: error_message,
          source_info: Some(DiagnosticSourceInfo { source, path }),
          start,
          end,
        }
      }
    }
  }
}
