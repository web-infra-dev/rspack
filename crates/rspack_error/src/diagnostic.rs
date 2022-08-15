use crate::Error;

pub enum Severity {
  Error,
  Warn,
}

pub struct Diagnostic {
  severity: Severity,
  message: String,
  /// Source code of current Diagnostic
  source: Option<String>,
  start: usize,
  end: usize,
}

impl Diagnostic {
  pub fn warn(message: String, start: usize, end: usize) -> Self {
    Self {
      severity: Severity::Warn,
      message,
      source: None,
      start,
      end,
    }
  }

  pub fn error(message: String, start: usize, end: usize) -> Self {
    Self {
      severity: Severity::Error,
      message,
      source: None,
      start,
      end,
    }
  }

  pub fn with_source(mut self, source: String) -> Self {
    self.source = Some(source);
    self
  }
}

impl From<Error> for Diagnostic {
  fn from(err: Error) -> Self {
    match err {
      Error::InternalError(message) => Self {
        severity: Severity::Error,
        message,
        source: None,
        start: 0,
        end: 0,
      },
      Error::TraceableError(traceable_error) => Self {
        severity: Severity::Error,
        message: traceable_error.error_message,
        source: traceable_error.source,
        start: traceable_error.start,
        end: traceable_error.end,
      },
    }
  }
}
