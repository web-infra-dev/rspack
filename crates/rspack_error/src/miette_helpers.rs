use std::fmt::Display;

use miette::Diagnostic;
use once_cell::sync::OnceCell;
use thiserror::Error;

use crate::Error;

/// Wrap diagnostic with additional help message.
#[derive(Debug, Error)]
#[error("{err}")]
pub(crate) struct WithHelp {
  err: Error,
  help: Option<String>,
  wrap_help: OnceCell<Option<String>>,
}

impl WithHelp {
  pub(crate) fn with_help(mut self, help: impl Into<String>) -> Self {
    self.help = Some(help.into());
    self
  }
}

impl From<Box<dyn Diagnostic + Send + Sync>> for WithHelp {
  fn from(value: Box<dyn Diagnostic + Send + Sync>) -> Self {
    Self {
      err: Error::new_boxed(value),
      help: None,
      wrap_help: OnceCell::new(),
    }
  }
}

impl From<Error> for WithHelp {
  fn from(value: Error) -> Self {
    Self {
      err: value,
      help: None,
      wrap_help: OnceCell::new(),
    }
  }
}

impl miette::Diagnostic for WithHelp {
  fn code<'a>(&'a self) -> Option<Box<dyn Display + 'a>> {
    self.err.code()
  }

  fn severity(&self) -> Option<miette::Severity> {
    self.err.severity()
  }

  fn help<'a>(&'a self) -> Option<Box<dyn Display + 'a>> {
    let help = self.wrap_help.get_or_init(|| {
      let prev = self.err.help().map(|h| h.to_string());
      let help = self.help.as_ref().map(|h| h.to_string());
      if let Some(prev) = prev {
        if let Some(help) = help {
          Some(format!("{prev}\n{help}"))
        } else {
          Some(prev)
        }
      } else if help.is_some() {
        help
      } else {
        None
      }
    });
    // Use overwritten help message instead.
    help.as_ref().map(Box::new).map(|h| h as Box<dyn Display>)
  }

  fn url<'a>(&'a self) -> Option<Box<dyn Display + 'a>> {
    self.err.url()
  }

  fn source_code(&self) -> Option<&dyn miette::SourceCode> {
    self.err.source_code()
  }

  fn labels(&self) -> Option<Box<dyn Iterator<Item = miette::LabeledSpan> + '_>> {
    self.err.labels()
  }

  fn related<'a>(&'a self) -> Option<Box<dyn Iterator<Item = &'a dyn miette::Diagnostic> + 'a>> {
    self.err.related()
  }

  fn diagnostic_source(&self) -> Option<&dyn Diagnostic> {
    self.err.diagnostic_source()
  }
}
