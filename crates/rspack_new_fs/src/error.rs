use std::{fmt, path::PathBuf};

pub type Result<T> = std::result::Result<T, Error>;

/// Errors produced by `rspack_new_fs` path normalization, root validation, and filesystem I/O.
#[derive(Debug)]
pub enum Error {
  Io(std::io::Error),
  NonUtf8Path(PathBuf),
  NotAbsolutePath(String),
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

impl From<std::io::Error> for Error {
  fn from(value: std::io::Error) -> Self {
    Self::Io(value)
  }
}

impl From<Error> for rspack_error::Error {
  fn from(value: Error) -> Self {
    rspack_error::error!(value.to_string())
  }
}

impl fmt::Display for Error {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "Rspack FS Error: ")?;
    match self {
      Self::Io(err) => write!(f, "IO error: {err}"),
      Self::NonUtf8Path(path) => write!(f, "non-UTF-8 path: {}", path.display()),
      Self::NotAbsolutePath(path) => write!(f, "expected absolute path, got `{path}`"),
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
