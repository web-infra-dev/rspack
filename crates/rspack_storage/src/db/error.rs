use std::{fmt, io};

use crate::fs::FSError;

#[derive(Debug)]
pub enum DBError {
  IO(io::Error),
  FS(FSError),
  InvalidFormat(String),
  CorruptedData(String),
}

impl fmt::Display for DBError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      DBError::IO(e) => write!(f, "IO error: {}", e),
      DBError::FS(e) => write!(f, "FS error: {}", e),
      DBError::InvalidFormat(s) => write!(f, "Invalid format: {}", s),
      DBError::CorruptedData(s) => write!(f, "Corrupted data: {}", s),
    }
  }
}

impl std::error::Error for DBError {}

impl From<io::Error> for DBError {
  fn from(e: io::Error) -> Self {
    DBError::IO(e)
  }
}

impl From<FSError> for DBError {
  fn from(e: FSError) -> Self {
    DBError::FS(e)
  }
}

pub type DBResult<T> = Result<T, DBError>;
