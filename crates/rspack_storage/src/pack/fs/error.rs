use std::io::ErrorKind;

use cow_utils::CowUtils;
use rspack_error::{
  miette::{self},
  thiserror::Error,
  Result,
};
use rspack_paths::Utf8Path;
use tokio::task::JoinError;

#[derive(Debug)]
pub enum StorageFSOperation {
  Read,
  Write,
  Dir,
  Remove,
  Stat,
  Move,
}

impl std::fmt::Display for StorageFSOperation {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::Read => write!(f, "read"),
      Self::Write => write!(f, "write"),
      Self::Dir => write!(f, "create dir"),
      Self::Remove => write!(f, "remove"),
      Self::Stat => write!(f, "stat"),
      Self::Move => write!(f, "move"),
    }
  }
}

#[derive(Debug, Error)]
pub struct StorageFSError {
  file: String,
  inner: rspack_fs::Error,
  opt: StorageFSOperation,
}

impl StorageFSError {
  pub fn from_fs_error(file: &Utf8Path, opt: StorageFSOperation, error: rspack_fs::Error) -> Self {
    Self {
      file: file.to_string(),
      inner: error,
      opt,
    }
  }
  pub fn is_not_found(&self) -> bool {
    if matches!(self.kind(), ErrorKind::NotFound) {
      return true;
    }
    let error_content = self.inner.to_string();
    let lower_case_error_content = error_content.cow_to_lowercase();
    lower_case_error_content.contains("no such file")
      || lower_case_error_content.contains("file not exists")
  }
  pub fn kind(&self) -> ErrorKind {
    match &self.inner {
      rspack_fs::Error::Io(e) => e.kind(),
    }
  }
}

impl std::fmt::Display for StorageFSError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "{} `{}` failed due to `{}`",
      self.opt,
      self.file,
      match &self.inner {
        rspack_fs::Error::Io(e) => e,
      }
    )
  }
}

impl miette::Diagnostic for StorageFSError {
  fn code<'a>(&'a self) -> Option<Box<dyn std::fmt::Display + 'a>> {
    Some(Box::new("StorageFSError"))
  }
  fn severity(&self) -> Option<miette::Severity> {
    Some(miette::Severity::Warning)
  }
  fn url<'a>(&'a self) -> Option<Box<dyn std::fmt::Display + 'a>> {
    Some(Box::new(self.file.clone()))
  }
}

#[derive(Debug, Error)]
pub struct BatchStorageFSError {
  message: String,
  join_error: Option<JoinError>,
  errors: Vec<rspack_error::Error>,
}

impl std::fmt::Display for BatchStorageFSError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.message)?;
    if let Some(join_error) = &self.join_error {
      write!(f, " due to `{}`", join_error)?;
    }
    for error in &self.errors {
      write!(f, "\n- {}", error)?;
    }
    Ok(())
  }
}

impl BatchStorageFSError {
  pub fn try_from_joined_result(
    message: &str,
    res: Result<Vec<Result<()>>, JoinError>,
  ) -> Option<Self> {
    match res {
      Ok(res) => Self::try_from_results(message, res),
      Err(join_error) => Some(Self {
        message: message.to_string(),
        errors: vec![],
        join_error: Some(join_error),
      }),
    }
  }

  pub fn try_from_results(message: &str, results: Vec<Result<()>>) -> Option<Self> {
    let errors = results
      .into_iter()
      .filter_map(|res| res.err())
      .collect::<Vec<_>>();
    if errors.is_empty() {
      None
    } else {
      Some(Self {
        message: message.to_string(),
        errors,
        join_error: None,
      })
    }
  }
}

impl miette::Diagnostic for BatchStorageFSError {
  fn code<'a>(&'a self) -> Option<Box<dyn std::fmt::Display + 'a>> {
    Some(Box::new("BatchStorageFSError"))
  }
  fn severity(&self) -> Option<miette::Severity> {
    Some(miette::Severity::Warning)
  }
}
