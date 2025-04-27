use std::{path::PathBuf, sync::Arc};

use futures::future::BoxFuture;
use rspack_error::Result;
use rspack_paths::Utf8PathBuf;
use rspack_regex::RspackRegex;

pub type KeepFunc = Arc<dyn Fn(String) -> BoxFuture<'static, Result<bool>> + Send + Sync>;

/// rust representation of the clean options
pub enum CleanOptions {
  // if true, clean all files
  CleanAll(bool),
  // keep the files under this path
  KeepPath(Utf8PathBuf),
  KeepRegex(RspackRegex),
  KeepFunc(KeepFunc),
}

impl std::fmt::Debug for CleanOptions {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::CleanAll(value) => write!(f, "CleanAll({value})"),
      Self::KeepPath(value) => write!(f, "KeepPath({value})"),
      Self::KeepRegex(value) => write!(f, "KeepRegex({})", value.to_source_string()),
      Self::KeepFunc(_) => write!(f, "KeepFunc()"),
    }
  }
}

impl CleanOptions {
  pub async fn keep(&self, path: &str) -> bool {
    match self {
      Self::CleanAll(value) => !value,
      Self::KeepPath(value) => {
        let path = PathBuf::from(path);
        path.starts_with(value)
      }
      Self::KeepRegex(value) => value.test(path),
      Self::KeepFunc(value) => value(path.to_owned())
        .await
        .expect("should call 'clean.keep' function"),
    }
  }
}
