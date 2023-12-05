use std::{fmt, io, path::Path};

use miette::{Diagnostic, IntoDiagnostic, MietteDiagnostic, NamedSource, SourceSpan};
use rspack_util::swc::normalize_custom_filename;
use swc_core::common::SourceFile;
use thiserror::Error;

use crate::Severity;

#[derive(Debug)]
pub struct InternalError(miette::Report);

impl<T: Diagnostic + Send + Sync + 'static> From<T> for InternalError {
  fn from(value: T) -> Self {
    InternalError(value.into())
  }
}

impl InternalError {
  pub fn new(error_message: String, severity: Severity) -> Self {
    Self(miette::Report::new(
      MietteDiagnostic::new(error_message.clone()).with_severity(severity.into()),
    ))
  }

  fn cast_to_miette(&self) -> &MietteDiagnostic {
    match self.0.downcast_ref::<MietteDiagnostic>() {
      Some(e) => e,
      None => unreachable!(),
    }
  }

  pub fn error_message(&self) -> &str {
    match self.0.downcast_ref::<MietteDiagnostic>() {
      Some(e) => &e.message,
      None => unreachable!(),
    }
  }

  pub fn severity(&self) -> Severity {
    let severity = self.cast_to_miette().severity.as_ref();
    match severity.expect("severity should available") {
      miette::Severity::Advice => unreachable!(),
      miette::Severity::Warning => Severity::Warn,
      miette::Severity::Error => Severity::Error,
    }
  }
}

impl fmt::Display for InternalError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    writeln!(f, "{}[internal]: {}", self.severity(), self.error_message())
  }
}

#[derive(Debug, Error, Diagnostic)]
#[diagnostic(code(TraceableError))]
#[error("error[{kind}]: {title}")]
pub struct TraceableError {
  title: String,
  kind: DiagnosticKind,
  message: String,
  // file_path: String,
  #[source_code]
  src: NamedSource,
  #[label("{message}")]
  label: SourceSpan,
}

impl TraceableError {
  pub fn from_source_file(
    source_file: &SourceFile,
    start: usize,
    end: usize,
    title: String,
    message: String,
  ) -> Self {
    let file_path = normalize_custom_filename(&source_file.name.to_string()).to_string();
    let file_src = source_file.src.to_string();
    Self {
      title,
      kind: Default::default(),
      message,
      src: NamedSource::new(file_path, file_src),
      label: SourceSpan::new(start.into(), (end - start).into()),
    }
  }

  pub fn from_file(
    file_path: String,
    file_src: String,
    start: usize,
    end: usize,
    title: String,
    message: String,
  ) -> Self {
    Self {
      title,
      kind: Default::default(),
      message,
      src: NamedSource::new(file_path, file_src),
      label: SourceSpan::new(start.into(), (end - start).into()),
    }
  }

  pub fn from_real_file_path(
    path: &Path,
    start: usize,
    end: usize,
    title: String,
    message: String,
  ) -> Result<Self, Error> {
    let file_src = std::fs::read_to_string(path).into_diagnostic()?;
    Ok(Self::from_file(
      path.to_string_lossy().into_owned(),
      file_src,
      start,
      end,
      title,
      message,
    ))
  }
}

#[derive(Debug)]
/// ## Warning
/// For a [TraceableRspackError], the path is required.
/// Because if the source code is missing when you construct a [TraceableRspackError], we could read it from file system later
/// when convert it into [crate::Diagnostic], but the reverse will not working.
pub struct TraceableRspackError {
  /// path of a file (real file or virtual file)
  pub(crate) file_path: String,
  /// source content of a file (real file or virtual file)
  pub(crate) file_src: String,
  pub(crate) start: usize,
  pub(crate) end: usize,
  pub(crate) error_message: String,
  pub(crate) title: String,
  pub(crate) kind: DiagnosticKind,
  pub(crate) severity: Severity,
}

impl TraceableRspackError {
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

impl fmt::Display for TraceableRspackError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    writeln!(f, "{}[{}]: {}", self.severity, self.kind, self.title)?;
    writeln!(f, "{}", self.error_message)?;
    writeln!(f, "in {}", self.file_path)
  }
}

#[derive(Debug)]
pub enum Error {
  InternalError(InternalError),
  TraceableRspackError(TraceableRspackError),
  BatchErrors(Vec<Error>),
}

impl From<miette::Error> for Error {
  fn from(value: miette::Error) -> Self {
    Self::InternalError(InternalError(value))
  }
}

impl std::error::Error for Error {
  fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
    match self {
      Error::InternalError(InternalError(i)) => i.source(),
      _ => None,
    }
  }
}

impl fmt::Display for Error {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      Error::InternalError(e) => write!(f, "{e}"),
      Error::TraceableRspackError(v) => write!(f, "{v}"),
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

impl From<serde_json::Error> for Error {
  fn from(value: serde_json::Error) -> Self {
    Error::InternalError(InternalError::new(value.to_string(), Severity::Error))
  }
}

impl From<io::Error> for Error {
  fn from(source: io::Error) -> Self {
    Error::InternalError(DiagnosticError(source.into()).into())
  }
}

impl From<anyhow::Error> for Error {
  fn from(source: anyhow::Error) -> Self {
    Error::InternalError(DiagnosticError(source.into()).into())
  }
}

impl From<rspack_sources::Error> for Error {
  fn from(value: rspack_sources::Error) -> Self {
    Error::InternalError(DiagnosticError(value.into()).into())
  }
}

impl Error {
  pub fn kind(&self) -> DiagnosticKind {
    match self {
      Error::InternalError(_) => DiagnosticKind::Internal,
      Error::TraceableRspackError(TraceableRspackError { kind, .. }) => *kind,
      Error::BatchErrors(_) => DiagnosticKind::Internal,
    }
  }
  pub fn severity(&self) -> Severity {
    match self {
      Error::InternalError(_) => Severity::Error,
      Error::TraceableRspackError(TraceableRspackError { severity, .. }) => *severity,
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
/// display string should be snake, for consistency.
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

/// Convenience [`Diagnostic`] that can be used as an "anonymous" wrapper for
/// Errors. This is intended to be paired with [`IntoDiagnostic`].
#[derive(Debug, Error)]
#[error(transparent)]
pub struct DiagnosticError(pub Box<dyn std::error::Error + Send + Sync + 'static>);
impl Diagnostic for DiagnosticError {}
