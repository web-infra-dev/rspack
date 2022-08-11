use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub struct TraceableError {
  pub path: String,
  pub start: usize,
  pub end: usize,
  pub error_message: String,
}

impl TraceableError {
  pub fn new(path: String, start: usize, end: usize, error_message: String) -> Self {
    Self {
      path,
      start,
      end,
      error_message,
    }
  }
}

#[derive(Error, Debug)]
pub enum Error {
  #[error("{0}")]
  InternalError(String),

  #[error("")]
  TraceableError(TraceableError),

  #[error("invalid data store response")]
  NestedArray(Vec<Self>),
}

impl Error {
  pub fn flatten(self) -> Vec<Error> {
    match self {
      Error::NestedArray(error_list) => error_list
        .into_iter()
        .flat_map(|error| error.flatten())
        .collect(),
      _ => {
        vec![self]
      }
    }
  }
}

impl From<anyhow::Error> for Error {
  fn from(error: anyhow::Error) -> Self {
    Self::InternalError(error.to_string())
  }
}
