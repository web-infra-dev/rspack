use std::io::ErrorKind;

use cow_utils::CowUtils;
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
  Redirect,
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
      Self::Redirect => write!(f, "redirect"),
    }
  }
}

#[derive(Debug)]
pub struct StorageFSError {
  file: String,
  inner: rspack_fs::Error,
  opt: StorageFSOperation,
}

impl std::error::Error for StorageFSError {}

impl StorageFSError {
  pub fn from_fs_error(file: &Utf8Path, opt: StorageFSOperation, error: rspack_fs::Error) -> Self {
    Self {
      file: file.to_string(),
      inner: error,
      opt,
    }
  }
  pub fn from_message(file: &Utf8Path, opt: StorageFSOperation, message: String) -> Self {
    Self {
      file: file.to_string(),
      inner: rspack_fs::Error::Io(std::io::Error::new(std::io::ErrorKind::Other, message)),
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

#[derive(Debug)]
pub struct BatchStorageFSError {
  message: String,
  join_error: Option<JoinError>,
  errors: Vec<Box<dyn std::error::Error + std::marker::Send + Sync>>,
}

impl From<StorageFSError> for BatchStorageFSError {
  fn from(error: StorageFSError) -> Self {
    Self {
      message: "".to_string(),
      join_error: None,
      errors: vec![Box::new(error)],
    }
  }
}

impl std::fmt::Display for BatchStorageFSError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.message)?;
    if let Some(join_error) = &self.join_error {
      write!(f, " due to `{}`", join_error)?;
    }
    if self.errors.len() == 1 {
      write!(f, "{}", self.errors[0])?;
    } else {
      for error in &self.errors {
        write!(f, "\n- {}", error)?;
      }
    }

    Ok(())
  }
}

impl BatchStorageFSError {
  pub fn try_from_joined_result<T: std::error::Error + std::marker::Send + Sync + 'static, R>(
    message: &str,
    res: Result<Vec<Result<R, T>>, JoinError>,
  ) -> Result<Vec<R>, Self> {
    match res {
      Ok(res) => Self::try_from_results(message, res),
      Err(join_error) => Err(Self {
        message: message.to_string(),
        errors: vec![],
        join_error: Some(join_error),
      }),
    }
  }

  pub fn try_from_results<T: std::error::Error + std::marker::Send + Sync + 'static, R>(
    message: &str,
    results: Vec<Result<R, T>>,
  ) -> Result<Vec<R>, Self> {
    let mut errors = vec![];
    let mut res = vec![];
    for r in results {
      match r {
        Ok(r) => res.push(r),
        Err(e) => errors.push(Box::new(e).into()),
      }
    }
    if errors.is_empty() {
      Ok(res)
    } else {
      Err(Self {
        message: message.to_string(),
        errors,
        join_error: None,
      })
    }
  }
}

impl std::error::Error for BatchStorageFSError {}
