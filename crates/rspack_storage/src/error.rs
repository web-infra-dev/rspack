use std::io::ErrorKind;

use cow_utils::CowUtils;
use rspack_fs::Error as FSError;

/// Rspack storage errors.
#[derive(Debug)]
pub enum Error {
  /// File system operation error
  FS(FSError),
  /// Data format parsing error (e.g., invalid pack file structure)
  InvalidFormat(String),
  /// Data integrity error (e.g., hash mismatch)
  CorruptedData(String),
}

impl std::fmt::Display for Error {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Error::FS(e) => write!(f, "{e}"),
      Error::InvalidFormat(s) => write!(f, "{s}"),
      Error::CorruptedData(s) => write!(f, "{s}"),
    }
  }
}

impl std::error::Error for Error {}

impl From<FSError> for Error {
  fn from(e: FSError) -> Self {
    Error::FS(e)
  }
}

impl From<Error> for rspack_error::Error {
  fn from(value: Error) -> Self {
    rspack_error::error!(value.to_string())
  }
}

impl Error {
  /// Returns true if the error is caused by a missing file or directory.
  pub fn is_not_found(&self) -> bool {
    match self {
      Error::FS(FSError::Io(e)) => {
        if matches!(e.kind(), ErrorKind::NotFound) {
          return true;
        }
        let error_string = e.to_string();
        let error_content = error_string.cow_to_ascii_lowercase();
        error_content.contains("no such file") || error_content.contains("file not exists")
      }
      _ => false,
    }
  }
}

pub type Result<T> = std::result::Result<T, Error>;
