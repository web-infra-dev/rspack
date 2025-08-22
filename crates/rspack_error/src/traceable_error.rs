use swc_core::common::SourceFile;

use crate::error::{Error, Label};

/// ## Warning
/// For a [TraceableError], the path is required.
/// Because if the source code is missing when you construct a [TraceableError], we could read it from file system later
/// when convert it into [crate::Diagnostic], but the reverse will not working.
#[derive(Debug, Clone)]
pub struct TraceableError(Error);

impl std::ops::Deref for TraceableError {
  type Target = Error;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl std::ops::DerefMut for TraceableError {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.0
  }
}

impl TraceableError {
  pub fn from_source_file(
    source_file: &SourceFile,
    start: usize,
    end: usize,
    title: String,
    message: String,
  ) -> Self {
    Self::from_string(
      Some(source_file.src.clone().into_string()),
      start,
      end,
      title,
      message,
    )
  }

  pub fn from_file(
    file_src: String,
    start: usize,
    end: usize,
    title: String,
    message: String,
  ) -> Self {
    Self::from_string(Some(file_src), start, end, title, message)
  }
  // lazy set source_file if we can't know the source content in advance
  pub fn from_lazy_file(start: usize, end: usize, title: String, message: String) -> Self {
    Self::from_string(None, start, end, title, message)
  }

  pub fn from_string(
    src: Option<String>,
    start: usize,
    end: usize,
    title: String,
    message: String,
  ) -> Self {
    let mut inner = Error::error(format!("{title}: {message}"));
    inner.src = src;
    inner.labels = Some(vec![Label {
      label: None,
      offset: start,
      len: end.saturating_sub(start),
    }]);
    Self(inner)
  }
}

impl From<TraceableError> for Error {
  fn from(value: TraceableError) -> Error {
    value.0
  }
}
