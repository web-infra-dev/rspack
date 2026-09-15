use std::{error, fmt, result};

/// An alias for [std::result::Result<T, rspack_sources::Error>].
pub type Result<T> = result::Result<T, Error>;

/// Error for this crate.
#[derive(Debug)]
pub enum Error {
  /// a JSON parsing related failure
  BadJson(simd_json::Error),
  /// a UTF-8 related failure
  Utf8(std::str::Utf8Error),
  /// an I/O related failure
  Io(std::io::Error),
}

impl fmt::Display for Error {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Error::BadJson(err) => write!(f, "bad json: {err}"),
      Error::Utf8(err) => write!(f, "utf8 error: {err}"),
      Error::Io(err) => write!(f, "io error: {err}"),
    }
  }
}

impl error::Error for Error {}

impl From<simd_json::Error> for Error {
  fn from(err: simd_json::Error) -> Error {
    Error::BadJson(err)
  }
}

impl From<std::str::Utf8Error> for Error {
  fn from(err: std::str::Utf8Error) -> Error {
    Error::Utf8(err)
  }
}

impl From<std::io::Error> for Error {
  fn from(err: std::io::Error) -> Error {
    Error::Io(err)
  }
}
