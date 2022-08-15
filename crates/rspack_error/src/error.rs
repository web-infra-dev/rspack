use thiserror::Error;

#[derive(Debug)]
pub struct TraceableError {
  pub path: Option<String>,
  pub start: usize,
  pub end: usize,
  pub error_message: String,
  pub source: Option<String>,
}

impl TraceableError {
  pub fn from_path(path: String, start: usize, end: usize, error_message: String) -> Self {
    Self {
      path: Some(path),
      start,
      end,
      error_message,
      source: None,
    }
  }
  pub fn from_source(source: String, start: usize, end: usize, error_message: String) -> Self {
    Self {
      path: None,
      start,
      end,
      error_message,
      source: Some(source),
    }
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
