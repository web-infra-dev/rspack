#[derive(Debug)]
pub enum Error {
  /// Generic I/O error
  Io(std::io::Error),
  /// File system operation context attached to an underlying error.
  Context {
    operation: &'static str,
    paths: Vec<String>,
    source: Box<Error>,
  },
}

impl Error {
  pub fn new(kind: std::io::ErrorKind, message: &str) -> Self {
    Error::Io(std::io::Error::new(kind, message))
  }

  pub fn with_path(self, operation: &'static str, path: impl ToString) -> Self {
    Self::Context {
      operation,
      paths: vec![path.to_string()],
      source: Box::new(self),
    }
  }

  pub fn with_paths(
    self,
    operation: &'static str,
    paths: impl IntoIterator<Item = impl ToString>,
  ) -> Self {
    Self::Context {
      operation,
      paths: paths.into_iter().map(|path| path.to_string()).collect(),
      source: Box::new(self),
    }
  }

  pub fn io_error(&self) -> &std::io::Error {
    match self {
      Error::Io(error) => error,
      Error::Context { source, .. } => source.io_error(),
    }
  }

  pub fn into_io_error(self) -> std::io::Error {
    match self {
      Error::Io(error) => error,
      context => std::io::Error::new(context.io_error().kind(), context.to_string()),
    }
  }
}

impl From<std::io::Error> for Error {
  fn from(value: std::io::Error) -> Self {
    Self::Io(value)
  }
}

impl From<rspack_error::Error> for Error {
  fn from(e: rspack_error::Error) -> Self {
    Error::Io(std::io::Error::other(e.to_string()))
  }
}

impl std::fmt::Display for Error {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Error::Io(err) => write!(f, "Rspack FS Error: IO error: {err}"),
      Error::Context {
        operation,
        paths,
        source,
      } => {
        write!(f, "Rspack FS Error: {operation}")?;
        for path in paths {
          write!(f, " '{path}'")?;
        }
        write!(f, " failed: {}", source.io_error())
      }
    }
  }
}

impl From<Error> for rspack_error::Error {
  fn from(value: Error) -> Self {
    rspack_error::error!(value.to_string())
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
    self.map_err(Error::into_io_error)
  }
}

#[cfg(test)]
mod tests {
  use super::Error;

  #[test]
  fn test_error_context_includes_operation_and_paths() {
    let error = Error::from(std::io::Error::from_raw_os_error(66))
      .with_paths("rename", ["/cache/.temp/make/_meta", "/cache/make/_meta"]);

    assert_eq!(error.io_error().raw_os_error(), Some(66));
    assert_eq!(
      error.to_string(),
      "Rspack FS Error: rename '/cache/.temp/make/_meta' '/cache/make/_meta' failed: Directory not empty (os error 66)"
    );
  }
}
