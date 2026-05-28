use std::{error, fmt, result};

/// An alias for [std::result::Result<T, rspack_sources::Error>].
pub type Result<T> = result::Result<T, Error>;

/// Error for this crate.
#[derive(Debug)]
pub enum Error {
  /// a JSON parsing related failure
  BadJson(simd_json::Error),
}

impl fmt::Display for Error {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Error::BadJson(err) => write!(f, "bad json: {err}"),
    }
  }
}

impl error::Error for Error {}

impl From<simd_json::Error> for Error {
  fn from(err: simd_json::Error) -> Error {
    Error::BadJson(err)
  }
}
