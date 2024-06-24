use std::{
  fmt,
  ops::Deref,
  path::{Path, PathBuf},
  sync::Arc,
};

use miette::{GraphicalTheme, IntoDiagnostic, MietteDiagnostic};
use rspack_identifier::Identifier;

use crate::{graphical::GraphicalReportHandler, Error};

#[derive(Debug, Clone, Default, Copy, PartialEq, Eq, Hash)]
pub enum RspackSeverity {
  #[default]
  Error,
  Warn,
}

pub type Severity = RspackSeverity;

impl From<RspackSeverity> for miette::Severity {
  fn from(value: RspackSeverity) -> Self {
    match value {
      RspackSeverity::Error => miette::Severity::Error,
      RspackSeverity::Warn => miette::Severity::Warning,
    }
  }
}

impl From<miette::Severity> for RspackSeverity {
  fn from(value: miette::Severity) -> Self {
    match value {
      miette::Severity::Error => RspackSeverity::Error,
      miette::Severity::Warning => RspackSeverity::Warn,
      miette::Severity::Advice => unimplemented!("Not supported miette severity"),
    }
  }
}

impl From<&str> for RspackSeverity {
  fn from(value: &str) -> Self {
    let s = value.to_ascii_lowercase();
    match s.as_str() {
      "warning" => RspackSeverity::Warn,
      _ => RspackSeverity::Error,
    }
  }
}

impl fmt::Display for RspackSeverity {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(
      f,
      "{}",
      match self {
        RspackSeverity::Error => "error",
        RspackSeverity::Warn => "warning",
      }
    )
  }
}

#[derive(Debug, Clone)]
pub struct Diagnostic {
  inner: Arc<miette::Error>,
  module_identifier: Option<Identifier>,
  file: Option<PathBuf>,
}

impl From<Box<dyn miette::Diagnostic + Send + Sync>> for Diagnostic {
  fn from(value: Box<dyn miette::Diagnostic + Send + Sync>) -> Self {
    Diagnostic::from(miette::Error::new_boxed(value))
  }
}

impl From<miette::Error> for Diagnostic {
  fn from(value: miette::Error) -> Self {
    Self {
      inner: Arc::new(value),
      module_identifier: None,
      file: None,
    }
  }
}

impl Deref for Diagnostic {
  type Target = miette::Error;

  fn deref(&self) -> &Self::Target {
    &self.inner
  }
}

impl Diagnostic {
  pub fn warn(title: String, message: String) -> Self {
    Self {
      inner: Error::from(
        MietteDiagnostic::new(message)
          .with_code(title)
          .with_severity(miette::Severity::Warning),
      )
      .into(),
      module_identifier: None,
      file: None,
    }
  }

  pub fn error(title: String, message: String) -> Self {
    Self {
      inner: Error::from(
        MietteDiagnostic::new(message)
          .with_code(title)
          .with_severity(miette::Severity::Error),
      )
      .into(),
      module_identifier: None,
      file: None,
    }
  }
}

impl Diagnostic {
  pub fn render_report(&self, colored: bool) -> crate::Result<String> {
    let mut buf = String::new();
    let h = GraphicalReportHandler::new()
      .with_theme(if colored {
        GraphicalTheme::unicode()
      } else {
        GraphicalTheme::unicode_nocolor()
      })
      .with_context_lines(2)
      .with_width(usize::MAX);
    h.render_report(&mut buf, self.as_ref()).into_diagnostic()?;
    Ok(buf)
  }

  pub fn message(&self) -> String {
    self.inner.to_string()
  }

  pub fn severity(&self) -> Severity {
    self.inner.severity().unwrap_or_default().into()
  }

  pub fn module_identifier(&self) -> Option<Identifier> {
    self.module_identifier
  }

  pub fn with_module_identifier(mut self, module_identifier: Option<Identifier>) -> Self {
    self.module_identifier = module_identifier;
    self
  }

  pub fn file(&self) -> Option<&Path> {
    self.file.as_deref()
  }

  pub fn with_file(mut self, file: Option<PathBuf>) -> Self {
    self.file = file;
    self
  }
}

pub trait Diagnosable {
  fn add_diagnostic(&self, _diagnostic: Diagnostic) {
    unimplemented!("`<T as Diagnosable>::add_diagnostic` is not implemented")
  }
  fn add_diagnostics(&self, _diagnostics: Vec<Diagnostic>) {
    unimplemented!("`<T as Diagnosable>::add_diagnostics` is not implemented")
  }
  /// Clone diagnostics from current [Diagnosable].
  /// This does not drain the diagnostics from the current one.
  fn clone_diagnostics(&self) -> Vec<Diagnostic> {
    vec![]
  }
  /// Take diagnostics from current [Diagnosable].
  /// This drains every diagnostic from the current one.
  fn take_diagnostics(&self) -> Vec<Diagnostic> {
    vec![]
  }
  /// Pipe diagnostics from the current [Diagnosable] to the target one.
  /// This drains every diagnostic from current, and pipe into the target one.
  fn pipe_diagnostics(&self, target: &dyn Diagnosable) {
    target.add_diagnostics(self.take_diagnostics())
  }
}

#[macro_export]
macro_rules! impl_empty_diagnosable_trait {
  ($ty:ty) => {
    impl $crate::Diagnosable for $ty {
      fn add_diagnostic(&self, _diagnostic: $crate::Diagnostic) {
        unimplemented!(
          "`<{ty} as Diagnosable>::add_diagnostic` is not implemented",
          ty = stringify!($ty)
        )
      }
      fn add_diagnostics(&self, _diagnostics: Vec<$crate::Diagnostic>) {
        unimplemented!(
          "`<{ty} as Diagnosable>::add_diagnostics` is not implemented",
          ty = stringify!($ty)
        )
      }
    }
  };
}

pub fn errors_to_diagnostics(errs: Vec<Error>) -> Vec<Diagnostic> {
  errs.into_iter().map(Diagnostic::from).collect()
}
