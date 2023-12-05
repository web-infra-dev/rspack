use std::fmt;

use crate::{DiagnosticKind, Error, TraceableRspackError};

#[derive(Debug, Clone, Default, Copy, PartialEq, Eq, Hash)]
pub enum Severity {
  #[default]
  Error,
  Warn,
}

impl From<Severity> for miette::Severity {
  fn from(value: Severity) -> Self {
    match value {
      Severity::Error => miette::Severity::Error,
      Severity::Warn => miette::Severity::Warning,
    }
  }
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
  pub(crate) severity: Severity,
  pub(crate) message: String,
  pub(crate) title: String,
  /// Source code and path of current Diagnostic
  pub(crate) source_info: Option<DiagnosticSourceInfo>,
  pub(crate) start: usize,
  pub(crate) end: usize,
  pub(crate) kind: DiagnosticKind,
  pub(crate) notes: Vec<String>,
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

  pub fn title(&self) -> &str {
    &self.title
  }

  pub fn message(&self) -> &str {
    &self.message
  }

  pub fn severity(&self) -> Severity {
    self.severity
  }

  pub fn with_kind(mut self, kind: DiagnosticKind) -> Self {
    self.kind = kind;
    self
  }

  pub fn with_source_info(mut self, source: DiagnosticSourceInfo) -> Self {
    self.source_info = Some(source);
    self
  }
  pub fn with_notes(mut self, notes: Vec<String>) -> Self {
    self.notes = notes;
    self
  }
}

impl From<Error> for Vec<Diagnostic> {
  fn from(err: Error) -> Self {
    let diagnostic = match err {
      Error::InternalError(err) => Diagnostic {
        message: err.error_message().to_string(),
        source_info: None,
        start: 0,
        end: 0,
        severity: err.severity(),
        ..Default::default()
      },
      Error::TraceableRspackError(TraceableRspackError {
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
        ..Default::default()
      },
      Error::BatchErrors(diagnostics) => {
        return diagnostics
          .into_iter()
          .flat_map(Vec::<Diagnostic>::from)
          .collect::<Vec<_>>()
      }
    };
    vec![diagnostic]
  }
}

pub fn errors_to_diagnostics(errs: Vec<Error>) -> Vec<Diagnostic> {
  errs.into_iter().flat_map(Vec::<Diagnostic>::from).collect()
}

pub const DIAGNOSTIC_POS_DUMMY: usize = 0;
