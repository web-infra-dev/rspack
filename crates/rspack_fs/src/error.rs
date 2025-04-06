use std::fmt::Display;

use rspack_error::{
  miette::{self, Diagnostic},
  thiserror::{self, Error},
};

#[derive(Debug, Error, Diagnostic)]
#[error("Rspack FS Error: {0}")]
struct FsError(#[source] std::io::Error);

#[derive(Debug)]
pub enum Error {
  /// Generic I/O error
  Io(std::io::Error),
}

impl Error {
  pub fn new(kind: std::io::ErrorKind, message: &str) -> Self {
    Error::Io(std::io::Error::new(kind, message))
  }
}

impl From<std::io::Error> for Error {
  fn from(value: std::io::Error) -> Self {
    Self::Io(value)
  }
}

impl From<Error> for rspack_error::Error {
  fn from(value: Error) -> Self {
    match value {
      Error::Io(err) => FsError(err).into(),
    }
  }
}

impl From<rspack_error::Error> for Error {
  fn from(e: rspack_error::Error) -> Self {
    Error::Io(std::io::Error::other(e.to_string()))
  }
}

impl Display for Error {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Error::Io(err) => write!(f, "IO error: {err}"),
    }
  }
}

pub type Result<T> = std::result::Result<T, Error>;

pub trait RspackResultToFsResultExt<T> {
  fn to_fs_result(self) -> Result<T>;
}

impl<T, E: ToString> RspackResultToFsResultExt<T> for std::result::Result<T, E> {
  fn to_fs_result(self) -> Result<T> {
    match self {
      Ok(t) => Ok(t),
      Err(e) => Err(Error::Io(std::io::Error::other(e.to_string()))),
    }
  }
}

pub trait IoResultToFsResultExt<T> {
  fn to_fs_result(self) -> Result<T>;
}

impl<T> IoResultToFsResultExt<T> for std::io::Result<T> {
  fn to_fs_result(self) -> Result<T> {
    self.map_err(Error::from)
  }
}

pub trait FsResultToIoResultExt<T> {
  fn to_io_result(self) -> std::io::Result<T>;
}

impl<T> FsResultToIoResultExt<T> for Result<T> {
  fn to_io_result(self) -> std::io::Result<T> {
    self.map_err(|e| match e {
      Error::Io(err) => err,
    })
  }
}
