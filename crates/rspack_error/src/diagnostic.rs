use std::{fmt, ops::Deref, sync::Arc};

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
pub struct Diagnostic(Arc<miette::Error>, DiagnosticMeta);

impl From<Box<dyn miette::Diagnostic + Send + Sync>> for Diagnostic {
  fn from(value: Box<dyn miette::Diagnostic + Send + Sync>) -> Self {
    Diagnostic::from(miette::Error::new_boxed(value))
  }
}

impl From<miette::Error> for Diagnostic {
  fn from(value: miette::Error) -> Self {
    Self(value.into(), DiagnosticMeta::default())
  }
}

impl Deref for Diagnostic {
  type Target = miette::Error;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl Diagnostic {
  pub fn warn(title: String, message: String) -> Self {
    Self(
      Error::from(
        MietteDiagnostic::new(message)
          .with_code(title)
          .with_severity(miette::Severity::Warning),
      )
      .into(),
      DiagnosticMeta::default(),
    )
  }

  pub fn error(title: String, message: String) -> Self {
    Self(
      Error::from(
        MietteDiagnostic::new(message)
          .with_code(title)
          .with_severity(miette::Severity::Error),
      )
      .into(),
      DiagnosticMeta::default(),
    )
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
    self.0.to_string()
  }

  pub fn severity(&self) -> Severity {
    self.0.severity().unwrap_or_default().into()
  }

  pub fn module_identifier(&self) -> Option<Identifier> {
    self.1.module_identifier
  }

  pub fn with_module_identifier(mut self, module_identifier: Option<Identifier>) -> Self {
    self.1.set_module_identifier(module_identifier);
    self
  }
}

#[derive(Debug, Default, Clone)]
struct DiagnosticMeta {
  module_identifier: Option<Identifier>,
}

impl DiagnosticMeta {
  fn set_module_identifier(&mut self, module_identifier: Option<Identifier>) {
    self.module_identifier = module_identifier;
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
