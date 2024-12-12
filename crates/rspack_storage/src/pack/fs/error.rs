use std::io::ErrorKind;

use cow_utils::CowUtils;
use rspack_error::{
  miette::{self},
  thiserror::{self, Error},
  Error,
};
use rspack_paths::Utf8Path;

#[derive(Debug)]
pub enum PackFsErrorOpt {
  Read,
  Write,
  Dir,
  Remove,
  Stat,
  Move,
}

impl std::fmt::Display for PackFsErrorOpt {
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
#[error(r#"Rspack Storage FS Error: {opt} `{file}` failed with `{inner}`"#)]
pub struct PackFsError {
  file: String,
  inner: Error,
  opt: PackFsErrorOpt,
  kind: Option<ErrorKind>,
}

impl PackFsError {
  pub fn from_fs_error(file: &Utf8Path, opt: PackFsErrorOpt, error: rspack_fs::Error) -> Self {
    let kind = match &error {
      rspack_fs::Error::Io(e) => Some(e.kind()),
    };
    Self {
      file: file.to_string(),
      inner: error.into(),
      opt,
      kind,
    }
  }
  pub fn is_not_found(&self) -> bool {
    if self.kind.is_some_and(|k| matches!(k, ErrorKind::NotFound)) {
      return true;
    }
    let error_content = self.inner.to_string();
    let lower_case_error_content = error_content.cow_to_lowercase();
    lower_case_error_content.contains("no such file")
      || lower_case_error_content.contains("file not exists")
  }
}

impl miette::Diagnostic for PackFsError {
  fn code<'a>(&'a self) -> Option<Box<dyn std::fmt::Display + 'a>> {
    Some(Box::new("PackFsError"))
  }
  fn severity(&self) -> Option<miette::Severity> {
    Some(miette::Severity::Warning)
  }
  fn url<'a>(&'a self) -> Option<Box<dyn std::fmt::Display + 'a>> {
    Some(Box::new(self.file.clone()))
  }
  fn diagnostic_source(&self) -> Option<&dyn miette::Diagnostic> {
    Some(self.inner.as_ref())
  }
}
