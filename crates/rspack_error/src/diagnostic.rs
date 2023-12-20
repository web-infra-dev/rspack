use std::{fmt, ops::Deref, sync::Arc};

use miette::{GraphicalReportHandler, GraphicalTheme, IntoDiagnostic, MietteDiagnostic};

use crate::Error;

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
pub struct Diagnostic(Arc<miette::Error>);

impl From<miette::Error> for Diagnostic {
  fn from(value: miette::Error) -> Self {
    Self(value.into())
  }
}

impl Deref for Diagnostic {
  type Target = miette::Error;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl Diagnostic {
  pub fn render_report(&self, colored: bool) -> crate::Result<String> {
    let h = GraphicalReportHandler::new().with_theme(if colored {
      GraphicalTheme::unicode()
    } else {
      GraphicalTheme::unicode_nocolor()
    });
    let mut buf = String::new();
    h.render_report(&mut buf, self.as_ref()).into_diagnostic()?;
    Ok(buf)
  }

  pub fn message(&self) -> String {
    self.0.to_string()
  }

  pub fn severity(&self) -> Severity {
    self.0.severity().unwrap_or_default().into()
  }

  pub fn warn(title: String, message: String) -> Self {
    Self(
      Error::from(
        MietteDiagnostic::new(message)
          .with_code(title)
          .with_severity(miette::Severity::Warning),
      )
      .into(),
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
    )
  }
}

pub trait Diagnosable {
  fn add_diagnostic(&self, _diagnostic: Diagnostic) {
    unimplemented!("`<T as Diagnostable>::add_diagnostic` is not implemented")
  }
  fn add_diagnostics(&self, _diagnostics: Vec<Diagnostic>) {
    unimplemented!("`<T as Diagnostable>::add_diagnostics` is not implemented")
  }
  fn clone_diagnostics(&self) -> Vec<Diagnostic> {
    vec![]
  }
}

#[macro_export]
macro_rules! impl_empty_diagnosable_trait {
  ($ty:ty) => {
    impl $crate::Diagnosable for $ty {
      fn add_diagnostic(&self, _diagnostic: $crate::Diagnostic) {
        unimplemented!(
          "`<{ty} as Diagnostable>::add_diagnostic` is not implemented",
          ty = stringify!($ty)
        )
      }
      fn add_diagnostics(&self, _diagnostics: Vec<$crate::Diagnostic>) {
        unimplemented!(
          "`<{ty} as Diagnostable>::add_diagnostics` is not implemented",
          ty = stringify!($ty)
        )
      }
      fn clone_diagnostics(&self) -> Vec<$crate::Diagnostic> {
        vec![]
      }
    }
  };
}

pub fn errors_to_diagnostics(errs: Vec<Error>) -> Vec<Diagnostic> {
  errs.into_iter().map(Diagnostic::from).collect()
}

pub const DIAGNOSTIC_POS_DUMMY: usize = 0;
