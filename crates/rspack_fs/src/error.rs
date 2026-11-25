use std::{fmt, path::PathBuf};

pub type Result<T> = std::result::Result<T, Error>;

/// Errors produced by `rspack_fs` path normalization, root validation, and filesystem I/O.
#[derive(Debug)]
pub enum Error {
  Io(std::io::Error),
  NonUtf8Path(PathBuf),
  NotAbsolutePath(String),
  NotRelativePath(String),
  PathEscapesRoot(String),
  InvalidRootName(String),
  UnknownRoot(String),
  OverlappingRoot {
    first_name: String,
    first_path: String,
    second_name: String,
    second_path: String,
  },
}

impl Error {
  pub fn new(
    kind: std::io::ErrorKind,
    message: impl Into<Box<dyn std::error::Error + Send + Sync>>,
  ) -> Self {
    Self::Io(std::io::Error::new(kind, message))
  }
}

impl From<std::io::Error> for Error {
  fn from(value: std::io::Error) -> Self {
    Self::Io(value)
  }
}

impl From<rspack_error::Error> for Error {
  fn from(value: rspack_error::Error) -> Self {
    let kind = match value.code.as_deref() {
      Some("ENOENT") => std::io::ErrorKind::NotFound,
      Some("ENOTDIR") => std::io::ErrorKind::NotADirectory,
      _ => std::io::ErrorKind::Other,
    };
    Self::Io(std::io::Error::new(kind, value.to_string()))
  }
}

impl From<Error> for rspack_error::Error {
  fn from(value: Error) -> Self {
    rspack_error::error!(value.to_string())
  }
}

pub trait RspackResultToFsResultExt<T> {
  fn to_fs_result(self) -> Result<T>;
}

impl<T> RspackResultToFsResultExt<T> for rspack_error::Result<T> {
  fn to_fs_result(self) -> Result<T> {
    self.map_err(Error::from)
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
    self.map_err(|err| match err {
      Error::Io(io_err) => io_err,
      other => std::io::Error::new(std::io::ErrorKind::InvalidInput, other.to_string()),
    })
  }
}

impl fmt::Display for Error {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "Rspack FS Error: ")?;
    match self {
      Self::Io(err) => write!(f, "IO error: {err}"),
      Self::NonUtf8Path(path) => write!(f, "non-UTF-8 path: {}", path.display()),
      Self::NotAbsolutePath(path) => write!(f, "expected absolute path, got `{path}`"),
      Self::NotRelativePath(path) => write!(f, "expected relative path, got `{path}`"),
      Self::PathEscapesRoot(path) => write!(f, "path escapes root: `{path}`"),
      Self::InvalidRootName(name) => write!(f, "invalid root name `{name}`"),
      Self::UnknownRoot(name) => write!(f, "unknown root `{name}`"),
      Self::OverlappingRoot {
        first_name,
        first_path,
        second_name,
        second_path,
      } => write!(
        f,
        "overlapping roots `{first_name}` (`{first_path}`) and `{second_name}` (`{second_path}`)"
      ),
    }
  }
}

impl std::error::Error for Error {
  fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
    match self {
      Self::Io(err) => Some(err),
      _ => None,
    }
  }
}
