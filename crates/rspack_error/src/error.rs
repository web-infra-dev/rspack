use std::io;

use thiserror::Error;

#[derive(Debug)]
/// ## Warning
/// For a [TraceableError], the path is required.
/// Because if the source code is missing when you construct a [TraceableError], we could read it from file system later
/// when conert it into [crate::Diagnostic], but the reverse will not working.
pub struct TraceableError {
  pub path: String,
  pub start: usize,
  pub end: usize,
  pub error_message: String,
  pub title: String,
  pub source: Option<String>,
}

impl TraceableError {
  pub fn from_path(
    path: String,
    start: usize,
    end: usize,
    title: String,
    error_message: String,
  ) -> Self {
    Self {
      path,
      start,
      end,
      error_message,
      source: None,
      title,
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
  },
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
