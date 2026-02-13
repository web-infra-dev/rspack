use std::{fmt, io};

use crate::fs::FSError;

#[derive(Debug)]
pub enum Error {
  IO(io::Error),
  FS(FSError),
  InvalidFormat(String),
  CorruptedData(String),
}

impl fmt::Display for Error {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Error::IO(e) => write!(f, "IO error: {}", e),
      Error::FS(e) => write!(f, "FS error: {}", e),
      Error::InvalidFormat(s) => write!(f, "Invalid format: {}", s),
      Error::CorruptedData(s) => write!(f, "Corrupted data: {}", s),
    }
  }
}

impl std::error::Error for Error {}

impl From<io::Error> for Error {
  fn from(e: io::Error) -> Self {
    Error::IO(e)
  }
}

impl From<FSError> for Error {
  fn from(e: FSError) -> Self {
    Error::FS(e)
  }
}

pub type Result<T> = std::result::Result<T, Error>;
