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
  //   #[error("invalid data store response")]
  //   NestedArray(Vec<Self>),
}

impl Error {
  //   pub fn flatten(self) -> Vec<Error> {
  //     match self {
  //       Error::NestedArray(error_list) => error_list
  //         .into_iter()
  //         .flat_map(|error| error.flatten())
  //         .collect(),
  //       _ => {
  //         vec![self]
  //       }
  //     }
  //   }
}

impl From<anyhow::Error> for Error {
  fn from(error: anyhow::Error) -> Self {
    Self::InternalError(error.to_string())
  }
}
