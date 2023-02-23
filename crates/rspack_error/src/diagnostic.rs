use std::{backtrace::Backtrace, fmt};

use crate::{DiagnosticKind, Error, TraceableError};

#[derive(Debug, Clone, Default, Copy, PartialEq, Eq, Hash)]
pub enum Severity {
  #[default]
  Error,
  Warn,
}

impl fmt::Display for Severity {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(
      f,
      "{}",
      match self {
        Severity::Error => "error",
        Severity::Warn => "warning",
      }
    )
  }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DiagnosticSourceInfo {
  pub path: String,
  pub source: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub struct Diagnostic {
  pub severity: Severity,
  pub message: String,
  pub title: String,
  /// Source code and path of current Diagnostic
  pub source_info: Option<DiagnosticSourceInfo>,
  pub start: usize,
  pub end: usize,
  pub kind: DiagnosticKind,
}

impl Diagnostic {
  pub fn warn(title: String, message: String, start: usize, end: usize) -> Self {
    Self {
      severity: Severity::Warn,
      title,
      message,
      source_info: None,
      start,
      end,
      ..Default::default()
    }
  }

  pub fn error(title: String, message: String, start: usize, end: usize) -> Self {
    Self {
      severity: Severity::Error,
      message,
      source_info: None,
      start,
      end,
      title,
      ..Default::default()
    }
  }

  pub fn with_kind(mut self, kind: DiagnosticKind) -> Self {
    self.kind = kind;
    self
  }

  pub fn with_source_info(mut self, source: DiagnosticSourceInfo) -> Self {
    self.source_info = Some(source);
    self
  }
}

impl From<Error> for Vec<Diagnostic> {
  fn from(err: Error) -> Self {
    let kind = err.kind();
    let severity = err.severity();
    let diagnostic = match err {
      Error::InternalError(err) => Diagnostic {
        message: format!("{err}"),
        source_info: None,
        start: 0,
        end: 0,
        severity: err.severity,
        ..Default::default()
      },
      Error::Napi {
        status,
        reason,
        backtrace,
      } => Diagnostic {
        message: format!("Napi Error: {status} - {reason}\n{backtrace}"),
        source_info: None,
        start: 0,
        end: 0,
        severity: Severity::Error,
        ..Default::default()
      },
      Error::TraceableError(TraceableError {
        start,
        end,
        error_message,
        title,
        kind,
        severity,
        file_path,
        file_src,
      }) => Diagnostic {
        message: error_message,
        source_info: Some(DiagnosticSourceInfo {
          source: file_src,
          path: file_path,
        }),
        start,
        end,
        title,
        kind,
        severity,
      },
      Error::Io { source } => Diagnostic {
        message: source.to_string(),
        kind,
        severity,
        ..Default::default()
      },
      Error::Anyhow { source } => Diagnostic {
        kind,
        severity,
        message: {
          let backtrace = match Backtrace::capture().status() {
            std::backtrace::BacktraceStatus::Captured => {
              format!("\nbacktrace:\n{}", source.backtrace())
            }
            _ => "".to_string(),
          };
          format!("{source}{backtrace}")
        },
        ..Default::default()
      },
      Error::BatchErrors(diagnostics) => {
        return diagnostics
          .into_iter()
          .flat_map(Vec::<Diagnostic>::from)
          .collect::<Vec<_>>()
      }
      Error::Panic { message, backtrace } => Diagnostic {
        severity: Severity::Error,
        message: format!("Unexpected Panic: {message}\n{backtrace}\n\nPlease file an issue: https://github.com/modern-js-dev/rspack/issues/new?assignees=&labels=C-bug&template=1-bug-report.en-US.yml&title=%5BBug+Report%5D%3A+"),
        kind: DiagnosticKind::Panic,
        ..Default::default()
      },
    };
    vec![diagnostic]
  }
}
pub fn errors_to_diagnostics(errs: Vec<Error>) -> Vec<Diagnostic> {
  errs.into_iter().flat_map(Vec::<Diagnostic>::from).collect()
}
