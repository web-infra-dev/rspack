use std::{fmt, io};

use crate::Severity;

#[macro_export]
macro_rules! internal_error {
  ($str:expr) => {
    InternalError {
      error_message: $str,
      ..Default::default()
    }
  };
}
#[derive(Debug, Default)]
pub struct InternalError {
  pub error_message: String,
  pub severity: Severity,
}

impl InternalError {
  pub fn with_severity(mut self, severity: Severity) -> Self {
    self.severity = severity;
    self
  }
}

impl fmt::Display for InternalError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    writeln!(f, "{}[internal]: {}", self.severity, self.error_message)
  }
}

#[derive(Debug)]
/// ## Warning
/// For a [TraceableError], the path is required.
/// Because if the source code is missing when you construct a [TraceableError], we could read it from file system later
/// when convert it into [crate::Diagnostic], but the reverse will not working.
pub struct TraceableError {
  pub path: String,
  pub start: usize,
  pub end: usize,
  pub error_message: String,
  pub title: String,
  pub source: Option<String>,
  pub kind: DiagnosticKind,
  pub severity: Severity,
}

impl TraceableError {
  pub fn from_path(
    path: String,
    start: usize,
    end: usize,
    title: String,
    error_message: String,
  ) -> Self {
    Self {
      path,
      start,
      end,
      error_message,
      source: None,
      title,
      kind: DiagnosticKind::Internal,
      severity: Severity::Error,
    }
  }
  pub fn with_source(mut self, source: String) -> Self {
    self.source = Some(source);
    self
  }

  pub fn with_kind(mut self, kind: DiagnosticKind) -> Self {
    self.kind = kind;
    self
  }
  pub fn with_severity(mut self, severity: Severity) -> Self {
    self.severity = severity;
    self
  }
}

impl fmt::Display for TraceableError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    writeln!(f, "{}[{}]: {}", self.severity, self.kind, self.title)?;
    writeln!(f, "{}", self.error_message)?;
    writeln!(f, "in {}", self.path)
  }
}

#[derive(Debug)]
pub enum Error {
  InternalError(InternalError),
  TraceableError(TraceableError),
  Io { source: io::Error },
  Anyhow { source: anyhow::Error },
  BatchErrors(Vec<Error>),
}

impl std::error::Error for Error {
  fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
    match self {
      Error::InternalError { .. } => None,
      Error::TraceableError { .. } => None,
      Error::Io { source, .. } => Some(source as &(dyn std::error::Error + 'static)),
      Error::Anyhow { source, .. } => Some(source.as_ref()),
      Error::BatchErrors { .. } => None,
    }
  }
}

impl fmt::Display for Error {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      Error::InternalError(e) => write!(f, "{}", e),
      Error::TraceableError(v) => write!(f, "{}", v),
      Error::Io { source } => write!(f, "{source}"),
      Error::Anyhow { source } => write!(f, "{source}"),
      Error::BatchErrors(errs) => write!(
        f,
        "{}",
        errs
          .iter()
          .map(|e| e.to_string())
          .collect::<Vec<String>>()
          .join("\n")
      ),
    }
  }
}

impl From<io::Error> for Error {
  fn from(source: io::Error) -> Self {
    Error::Io { source }
  }
}

impl From<anyhow::Error> for Error {
  fn from(source: anyhow::Error) -> Self {
    Error::Anyhow { source }
  }
}

impl Error {
  pub fn kind(&self) -> DiagnosticKind {
    match self {
      Error::InternalError(_) => DiagnosticKind::Internal,
      Error::TraceableError(TraceableError { kind, .. }) => *kind,
      Error::Io { .. } => DiagnosticKind::Io,
      Error::Anyhow { .. } => DiagnosticKind::Internal,
      Error::BatchErrors(_) => DiagnosticKind::Internal,
    }
  }
  pub fn severity(&self) -> Severity {
    match self {
      Error::InternalError(_) => Severity::Error,
      Error::TraceableError(TraceableError { severity, .. }) => *severity,
      Error::Io { .. } => Severity::Error,
      Error::Anyhow { .. } => Severity::Error,
      Error::BatchErrors(_) => Severity::Error,
    }
  }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum DiagnosticKind {
  JavaScript,
  Typescript,
  Jsx,
  Tsx,
  Scss,
  Css,
  #[default]
  Internal,
  Io,
  Json,
  Html,
}

/// About the manually implementation,
/// dispaly string should be snake, for consistency.
impl std::fmt::Display for DiagnosticKind {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      DiagnosticKind::JavaScript => write!(f, "javascript"),
      DiagnosticKind::Typescript => write!(f, "typescript"),
      DiagnosticKind::Jsx => write!(f, "jsx"),
      DiagnosticKind::Tsx => write!(f, "tsx"),
      DiagnosticKind::Scss => write!(f, "scss"),
      DiagnosticKind::Css => write!(f, "css"),
      DiagnosticKind::Internal => write!(f, "internal"),
      DiagnosticKind::Io => write!(f, "io"),
      DiagnosticKind::Json => write!(f, "json"),
      DiagnosticKind::Html => write!(f, "html"),
    }
  }
}

#[cfg(feature = "napi")]
impl From<napi::Error> for Error {
  fn from(err: napi::Error) -> Self {
    Error::InternalError(internal_error!(err.to_string()))
  }
}

#[cfg(feature = "napi")]
impl From<Error> for napi::Error {
  fn from(err: Error) -> Self {
    Self::from_reason(format!("{}", err))
  }
}
