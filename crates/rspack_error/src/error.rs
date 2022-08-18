use std::io;

use thiserror::Error;

#[derive(Debug)]
/// # Warning
/// For a TraceableError, the path is required.
/// Because if the source code is missing, we could read it from file system later
/// when conert it into [crate::Diagnostic], the reverse it not true
pub struct TraceableError {
  pub path: String,
  pub start: usize,
  pub end: usize,
  pub error_message: String,
  pub source: Option<String>,
}

impl TraceableError {
  pub fn from_path(path: String, start: usize, end: usize, error_message: String) -> Self {
    // dbg!(&path, &start, &end, &error_message);
    Self {
      path,
      start,
      end,
      error_message,
      source: None,
    }
  }
  pub fn with_source(mut self, source: String) -> Self {
    self.source = Some(source);
    self
  }
}

#[derive(Error, Debug)]
pub enum Error {
  #[error("{0}")]
  InternalError(String),

  #[error("")]
  TraceableError(TraceableError),

  #[error("")]
  Io {
    #[from]
    source: io::Error,
  }, /*   #[error("invalid data store response")]
      *   NestedArray(Vec<Self>), */
  #[error("")]
  Anyhow {
    #[from]
    source: anyhow::Error,
  },
  #[error("")]
  Json {
    #[from]
    source: json::Error,
  },
}
