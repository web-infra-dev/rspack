use std::{path::PathBuf, str::FromStr};

use rspack_paths::Utf8PathBuf;

/// rust representation of the clean options
// TODO: support RegExp and function type
#[derive(Debug)]
pub enum CleanOptions {
  // if true, clean all files
  CleanAll(bool),
  // keep the files under this path
  KeepPath(Utf8PathBuf),
}

impl CleanOptions {
  pub fn keep(&self, path: &str) -> bool {
    match self {
      Self::CleanAll(value) => !*value,
      Self::KeepPath(value) => {
        let path = PathBuf::from(path);
        path.starts_with(value)
      }
    }
  }
}

impl From<bool> for CleanOptions {
  fn from(value: bool) -> Self {
    Self::CleanAll(value)
  }
}

impl From<&'_ str> for CleanOptions {
  fn from(value: &str) -> Self {
    let pb = Utf8PathBuf::from_str(value).expect("should be a valid path");
    Self::KeepPath(pb)
  }
}
impl From<&String> for CleanOptions {
  fn from(value: &String) -> Self {
    let pb = Utf8PathBuf::from_str(value).expect("should be a valid path");
    Self::KeepPath(pb)
  }
}

impl From<String> for CleanOptions {
  fn from(value: String) -> Self {
    let pb = Utf8PathBuf::from_str(&value).expect("should be a valid path");
    Self::KeepPath(pb)
  }
}
