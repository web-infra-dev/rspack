use std::{path::PathBuf, str::FromStr};

/// rust representation of the clean options
// TODO: support RegExp and function type
#[derive(Debug)]
pub enum CleanOptions {
  // if true, clean all files
  Boolean(bool),
  // keep the files under this path
  KeepPath(PathBuf),
}

impl CleanOptions {
  pub fn keep(&self, path: &str) -> bool {
    match self {
      Self::Boolean(value) => !*value,
      Self::KeepPath(value) => {
        let path = PathBuf::from(path);
        path.starts_with(value)
      }
    }
  }
}

impl From<bool> for CleanOptions {
  fn from(value: bool) -> Self {
    Self::Boolean(value)
  }
}

impl From<&'_ str> for CleanOptions {
  fn from(value: &str) -> Self {
    let pb = PathBuf::from_str(value).expect("should be a valid path");
    Self::KeepPath(pb)
  }
}

impl From<String> for CleanOptions {
  fn from(value: String) -> Self {
    let pb = PathBuf::from_str(&value).expect("should be a valid path");
    Self::KeepPath(pb)
  }
}
