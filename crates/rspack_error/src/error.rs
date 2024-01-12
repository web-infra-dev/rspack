use std::{fmt::Display, path::Path};

use miette::{
  Diagnostic, IntoDiagnostic, LabeledSpan, MietteDiagnostic, Severity, SourceCode, SourceSpan,
};
use swc_core::common::SourceFile;
use thiserror::Error;

use crate::RspackSeverity;

#[derive(Debug, Error)]
#[error(transparent)]
pub struct InternalError(#[from] Box<dyn Diagnostic + Send + Sync + 'static>);

impl InternalError {
  pub fn new(message: String, severity: RspackSeverity) -> Self {
    Self(Box::new(
      MietteDiagnostic::new(message).with_severity(severity.into()),
    ))
  }
}

/// Convenience [`Diagnostic`] that can be used as an "anonymous" wrapper for
/// Errors. This is intended to be paired with [`IntoDiagnostic`].
#[derive(Debug, Error)]
#[error(transparent)]
pub struct DiagnosticError(Box<dyn std::error::Error + Send + Sync + 'static>);
impl Diagnostic for DiagnosticError {}

impl From<Box<dyn std::error::Error + Send + Sync + 'static>> for DiagnosticError {
  fn from(value: Box<dyn std::error::Error + Send + Sync + 'static>) -> Self {
    Self(value)
  }
}

/// Handle [anyhow::Error]
/// Please try NOT to use this as much as possible.
#[derive(Debug, Error, Diagnostic)]
#[error(transparent)]
pub struct AnyhowError(#[from] anyhow::Error);

/// ## Warning
/// For a [TraceableError], the path is required.
/// Because if the source code is missing when you construct a [TraceableError], we could read it from file system later
/// when convert it into [crate::Diagnostic], but the reverse will not working.
#[derive(Debug, Clone, Error)]
#[error("{title}: {message}")]
pub struct TraceableError {
  title: String,
  kind: DiagnosticKind,
  message: String,
  severity: Severity,
  src: String,
  label: SourceSpan,
  help: Option<String>,
  url: Option<String>,
}

impl Diagnostic for TraceableError {
  fn severity(&self) -> Option<Severity> {
    Some(self.severity)
  }

  fn help(&self) -> Option<Box<dyn Display + '_>> {
    self
      .help
      .as_ref()
      .map(Box::new)
      .map(|c| c as Box<dyn Display>)
  }

  fn url(&self) -> Option<Box<dyn Display + '_>> {
    self
      .url
      .as_ref()
      .map(Box::new)
      .map(|c| c as Box<dyn Display>)
  }

  fn source_code(&self) -> Option<&dyn SourceCode> {
    Some(&self.src)
  }

  fn labels(&self) -> Option<Box<dyn Iterator<Item = LabeledSpan> + '_>> {
    use miette::macro_helpers::{OptionalWrapper, ToOption};
    std::option::Option::Some(Box::new(
      vec![OptionalWrapper::<SourceSpan>::new()
        .to_option(&self.label)
        .map(|label| miette::LabeledSpan::new_with_span(None, *label))]
      .into_iter()
      .filter(Option::is_some)
      .flatten(),
    ))
  }
}

impl TraceableError {
  pub fn with_severity(mut self, severity: impl Into<Severity>) -> Self {
    self.severity = severity.into();
    self
  }

  pub fn with_kind(mut self, kind: DiagnosticKind) -> Self {
    self.kind = kind;
    self
  }

  pub fn with_help(mut self, help: Option<impl Into<String>>) -> Self {
    self.help = help.map(|h| h.into());
    self
  }

  pub fn with_url(mut self, url: Option<impl Into<String>>) -> Self {
    self.url = url.map(|u| u.into());
    self
  }

  pub fn from_source_file(
    source_file: &SourceFile,
    start: usize,
    end: usize,
    title: String,
    message: String,
  ) -> Self {
    let file_src = source_file.src.to_string();
    let start = if start >= file_src.len() { 0 } else { start };
    let end = if end >= file_src.len() { 0 } else { end };
    Self {
      title,
      kind: Default::default(),
      message,
      severity: Default::default(),
      src: file_src,
      label: SourceSpan::new(start.into(), end.saturating_sub(start).into()),
      help: None,
      url: None,
    }
  }

  pub fn from_file(
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
      severity: Default::default(),
      src: file_src,
      label: SourceSpan::new(start.into(), end.saturating_sub(start).into()),
      help: None,
      url: None,
    }
  }

  pub fn from_empty_file(start: usize, end: usize, title: String, message: String) -> Self {
    Self {
      title,
      kind: Default::default(),
      message,
      severity: Default::default(),
      src: "".to_string(),
      label: SourceSpan::new(start.into(), end.saturating_sub(start).into()),
      help: None,
      url: None,
    }
  }

  pub fn from_real_file_path(
    path: &Path,
    start: usize,
    end: usize,
    title: String,
    message: String,
  ) -> Result<Self, miette::Error> {
    let file_src = std::fs::read_to_string(path).into_diagnostic()?;
    let start = if start >= file_src.len() { 0 } else { start };
    let end = if end >= file_src.len() { 0 } else { end };
    Ok(Self::from_file(file_src, start, end, title, message))
  }
}

/// Multiple errors to represent different kinds of errors.
/// NEVER implement this with [miette::Diagnostic],
/// because it makes code hard to maintain.
#[derive(Debug, Default)]
pub struct BatchErrors(pub Vec<miette::Error>);

impl BatchErrors {
  pub fn into_inner(self) -> Vec<miette::Error> {
    self.0
  }
}

impl From<BatchErrors> for Vec<crate::Diagnostic> {
  fn from(value: BatchErrors) -> Self {
    value.0.into_iter().map(crate::Diagnostic::from).collect()
  }
}

impl From<miette::Error> for BatchErrors {
  fn from(value: miette::Error) -> Self {
    Self(vec![value])
  }
}

impl From<Vec<miette::Error>> for BatchErrors {
  fn from(value: Vec<miette::Error>) -> Self {
    Self(value)
  }
}

#[macro_export]
macro_rules! impl_diagnostic_transparent {
  (code = $value:expr, $ty:ty) => {
    impl miette::Diagnostic for $ty {
      fn code<'a>(&'a self) -> Option<Box<dyn std::fmt::Display + 'a>> {
        Some(Box::new($value))
      }

      fn severity(&self) -> Option<miette::Severity> {
        self.0.severity()
      }

      fn help<'a>(&'a self) -> Option<Box<dyn std::fmt::Display + 'a>> {
        self.0.help()
      }

      fn url<'a>(&'a self) -> Option<Box<dyn std::fmt::Display + 'a>> {
        self.0.url()
      }

      fn source_code(&self) -> Option<&dyn miette::SourceCode> {
        self.0.source_code()
      }

      fn labels(&self) -> Option<Box<dyn Iterator<Item = miette::LabeledSpan> + '_>> {
        self.0.labels()
      }

      fn related<'a>(
        &'a self,
      ) -> Option<Box<dyn Iterator<Item = &'a dyn miette::Diagnostic> + 'a>> {
        self.0.related()
      }

      fn diagnostic_source(&self) -> Option<&dyn Diagnostic> {
        self.0.diagnostic_source()
      }
    }
  };
  ($ty:ty) => {
    impl miette::Diagnostic for $ty {
      fn code<'a>(&'a self) -> Option<Box<dyn std::fmt::Display + 'a>> {
        self.0.code()
      }

      fn severity(&self) -> Option<miette::Severity> {
        self.0.severity()
      }

      fn help<'a>(&'a self) -> Option<Box<dyn std::fmt::Display + 'a>> {
        self.0.help()
      }

      fn url<'a>(&'a self) -> Option<Box<dyn std::fmt::Display + 'a>> {
        self.0.url()
      }

      fn source_code(&self) -> Option<&dyn miette::SourceCode> {
        self.0.source_code()
      }

      fn labels(&self) -> Option<Box<dyn Iterator<Item = miette::LabeledSpan> + '_>> {
        self.0.labels()
      }

      fn related<'a>(
        &'a self,
      ) -> Option<Box<dyn Iterator<Item = &'a dyn miette::Diagnostic> + 'a>> {
        self.0.related()
      }

      fn diagnostic_source(&self) -> Option<&dyn Diagnostic> {
        self.0.diagnostic_source()
      }
    }
  };
}

impl_diagnostic_transparent!(InternalError);

#[macro_export]
macro_rules! impl_error_transparent {
  ($ty:ty) => {
    impl std::error::Error for ModuleBuildError {
      fn source(&self) -> ::core::option::Option<&(dyn std::error::Error + 'static)> {
        std::error::Error::source(<Error as AsRef<dyn std::error::Error>>::as_ref(&self.0))
      }
    }

    #[allow(unused_qualifications)]
    impl ::core::fmt::Display for ModuleBuildError {
      #[allow(clippy::used_underscore_binding)]
      fn fmt(&self, __formatter: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        ::core::fmt::Display::fmt(&self.0, __formatter)
      }
    }
  };
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

fn _assert() {
  fn _assert_send_sync<T: Send + Sync>() {}
  _assert_send_sync::<InternalError>();
  _assert_send_sync::<DiagnosticError>();
}
