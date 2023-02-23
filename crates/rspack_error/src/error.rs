use std::{fmt, io, path::Path};

use rspack_util::swc::normalize_custom_filename;
use swc_core::common::SourceFile;

use crate::Severity;

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
  /// path of a file (real file or virtual file)
  pub file_path: String,
  /// source content of a file (real file or virtual file)
  pub file_src: String,
  pub start: usize,
  pub end: usize,
  pub error_message: String,
  pub title: String,
  pub kind: DiagnosticKind,
  pub severity: Severity,
}

impl TraceableError {
  pub fn from_source_file(
    source_file: &SourceFile,
    start: usize,
    end: usize,
    title: String,
    error_message: String,
  ) -> Self {
    let file_path = normalize_custom_filename(&source_file.name.to_string()).to_string();
    let file_src = source_file.src.to_string();
    Self {
      file_path,
      file_src,
      start,
      end,
      error_message,
      title,
      kind: DiagnosticKind::Internal,
      severity: Severity::Error,
    }
  }

  pub fn from_file(
    file_path: String,
    file_src: String,
    start: usize,
    end: usize,
    title: String,
    error_message: String,
  ) -> Self {
    Self {
      file_path,
      file_src,
      start,
      end,
      error_message,
      title,
      kind: DiagnosticKind::Internal,
      severity: Severity::Error,
    }
  }

  pub fn from_real_file_path(
    path: &Path,
    start: usize,
    end: usize,
    title: String,
    error_message: String,
  ) -> Result<Self, Error> {
    let file_src = std::fs::read_to_string(path)?;
    Ok(Self::from_file(
      path.to_string_lossy().into_owned(),
      file_src,
      start,
      end,
      title,
      error_message,
    ))
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
    writeln!(f, "in {}", self.file_path)
  }
}

#[derive(Debug)]
pub enum Error {
  InternalError(InternalError),
  TraceableError(TraceableError),
  Io {
    source: io::Error,
  },
  Anyhow {
    source: anyhow::Error,
  },
  BatchErrors(Vec<Error>),
  // for some reason, We could not just use `napi:Error` here
  Napi {
    status: String,
    reason: String,
    backtrace: String,
  },
  Panic {
    message: String,
    backtrace: String,
  },
}

impl std::error::Error for Error {
  fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
    match self {
      Error::Io { source, .. } => Some(source as &(dyn std::error::Error + 'static)),
      Error::Anyhow { source, .. } => Some(source.as_ref()),
      _ => None,
    }
  }
}

impl fmt::Display for Error {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      Error::InternalError(e) => write!(f, "{e}"),
      Error::TraceableError(v) => write!(f, "{v}"),
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
      Error::Napi {
        status,
        reason,
        backtrace,
      } => write!(f, "napi error: {status} - {reason}\n{backtrace}"),
      Error::Panic { message, backtrace } => write!(f, "unexpected panic: {message}\n{backtrace}"),
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
      Error::Napi { .. } => DiagnosticKind::Internal,
      Error::Panic { .. } => DiagnosticKind::Panic,
    }
  }
  pub fn severity(&self) -> Severity {
    match self {
      Error::InternalError(_) => Severity::Error,
      Error::TraceableError(TraceableError { severity, .. }) => *severity,
      Error::Io { .. } => Severity::Error,
      Error::Anyhow { .. } => Severity::Error,
      Error::BatchErrors(_) => Severity::Error,
      Error::Napi { .. } => Severity::Error,
      Error::Panic { .. } => Severity::Error,
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
  Panic,
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
      DiagnosticKind::Panic => write!(f, "panic"),
    }
  }
}
