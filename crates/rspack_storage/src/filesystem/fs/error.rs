use std::io::ErrorKind;

use cow_utils::CowUtils;
use rspack_paths::Utf8Path;
use tokio::task::JoinError;

pub type FSResult<T> = Result<T, FSError>;
pub type BatchFSResult<T> = Result<T, BatchFSError>;

pub trait FsResultToStorageFsResult<T> {
  fn to_storage_fs_result(self, path: &Utf8Path, opt: FSOperation) -> FSResult<T>;
}

impl<T> FsResultToStorageFsResult<T> for Result<T, rspack_fs::Error> {
  fn to_storage_fs_result(self, path: &Utf8Path, opt: FSOperation) -> FSResult<T> {
    self.map_err(|e| FSError {
      file: path.to_string(),
      inner: e,
      opt,
    })
  }
}

#[derive(Debug)]
pub enum FSOperation {
  Read,
  Write,
  Dir,
  Remove,
  Stat,
  Move,
  Redirect,
}

impl std::fmt::Display for FSOperation {
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
pub struct FSError {
  file: String,
  inner: rspack_fs::Error,
  opt: FSOperation,
}

impl std::error::Error for FSError {}

impl FSError {
  pub fn from_message(file: &Utf8Path, opt: FSOperation, message: String) -> Self {
    Self {
      file: file.to_string(),
      inner: rspack_fs::Error::Io(std::io::Error::other(message)),
      opt,
    }
  }
  pub fn is_not_found(&self) -> bool {
    if matches!(self.kind(), ErrorKind::NotFound) {
      return true;
    }
    let error_content = self.inner.to_string();
    let lower_case_error_content = error_content.cow_to_ascii_lowercase();
    lower_case_error_content.contains("no such file")
      || lower_case_error_content.contains("file not exists")
  }
  pub fn kind(&self) -> ErrorKind {
    match &self.inner {
      rspack_fs::Error::Io(e) => e.kind(),
    }
  }
}

impl std::fmt::Display for FSError {
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
pub struct BatchFSError {
  message: String,
  join_error: Option<JoinError>,
  errors: Vec<Box<dyn std::error::Error + std::marker::Send + Sync>>,
}

impl From<FSError> for BatchFSError {
  fn from(error: FSError) -> Self {
    Self {
      message: String::new(),
      join_error: None,
      errors: vec![Box::new(error)],
    }
  }
}

impl std::fmt::Display for BatchFSError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.message)?;
    if let Some(join_error) = &self.join_error {
      write!(f, " due to `{join_error}`")?;
    } else {
      for error in self.errors.iter().take(5) {
        write!(f, "\n{error}")?;
      }
      if self.errors.len() > 5 {
        write!(f, "\n...")?;
      }
    }
    Ok(())
  }
}

impl BatchFSError {
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

impl std::error::Error for BatchFSError {}
