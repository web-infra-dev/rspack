use std::{fmt, ops::Deref, sync::Arc};

use miette::MietteDiagnostic;

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

/// Shouldn't rely on this in deduping.
/// Have to make sure everything is deduped in the first place.
impl std::hash::Hash for Diagnostic {
  fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
    self.0.to_string().hash(state);
    self.0.code().map(|c| c.to_string()).hash(state);
    self.0.help().map(|h| h.to_string()).hash(state);
    self.0.url().map(|u| u.to_string()).hash(state);
    self.0.severity().map(Severity::from).hash(state);
  }
}

/// Shouldn't rely on this in deduping.
/// Have to make sure everything is deduped in the first place.
impl PartialEq for Diagnostic {
  fn eq(&self, other: &Self) -> bool {
    self.0.to_string() == other.0.to_string()
      && self.0.code().map(|c| c.to_string()) == other.0.code().map(|c| c.to_string())
      && self.0.help().map(|h| h.to_string()) == other.0.help().map(|h| h.to_string())
      && self.0.url().map(|u| u.to_string()) == other.0.url().map(|u| u.to_string())
      && self.0.severity() == other.0.severity()
  }
}

impl Eq for Diagnostic {}

impl Diagnostic {
  pub fn title(&self) -> String {
    self.0.code().map(|v| v.to_string()).unwrap_or_default()
  }

  pub fn message(&self) -> String {
    self.0.to_string()
  }

  pub fn labels_string(&self) -> Option<String> {
    self
      .0
      .labels()
      .map(|l| {
        l.into_iter()
          .filter_map(|l| l.label().map(ToString::to_string))
          .collect::<Vec<_>>()
      })
      .map(|s| s.join("\n"))
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

pub fn errors_to_diagnostics(errs: Vec<Error>) -> Vec<Diagnostic> {
  errs.into_iter().map(Diagnostic::from).collect()
}

pub const DIAGNOSTIC_POS_DUMMY: usize = 0;
