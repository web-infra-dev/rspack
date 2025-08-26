use std::fmt::Display;

use miette::{Diagnostic as MietteDiagnostic, LabeledSpan};

#[derive(Debug, Clone, Default, Copy, PartialEq, Eq, Hash)]
pub enum Severity {
  #[default]
  Error,
  Warning,
}

#[derive(Debug, Clone, Default)]
pub struct Label {
  pub label: Option<String>,
  pub offset: usize,
  pub len: usize,
}

#[derive(Debug, Clone, Default)]
pub struct ErrorData {
  pub severity: Severity,
  pub message: String,
  pub details: Option<String>,
  pub src: Option<String>,
  pub code: Option<String>,
  pub help: Option<String>,
  pub url: Option<String>,
  pub labels: Option<Vec<Label>>,
  pub source_error: Option<Box<Error>>,
  pub stack: Option<String>,
  pub hide_stack: Option<bool>,
}

#[derive(Debug, Clone, Default)]
pub struct Error(Box<ErrorData>);

impl std::ops::Deref for Error {
  type Target = ErrorData;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl std::ops::DerefMut for Error {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.0
  }
}

impl Error {
  #[allow(clippy::self_named_constructors)]
  pub fn error(message: String) -> Self {
    Self(Box::new(ErrorData {
      message,
      ..Default::default()
    }))
  }
  pub fn warning(message: String) -> Self {
    Self(Box::new(ErrorData {
      severity: Severity::Warning,
      message,
      ..Default::default()
    }))
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

  pub fn from_string(
    src: Option<String>,
    start: usize,
    end: usize,
    title: String,
    message: String,
  ) -> Self {
    let mut error = Error::error(format!("{title}: {message}"));
    error.src = src;
    error.labels = Some(vec![Label {
      label: None,
      offset: start,
      len: end.saturating_sub(start),
    }]);
    error
  }

  pub fn from_error<T>(value: T) -> Self
  where
    T: std::error::Error,
  {
    let mut error = Error::error(value.to_string());
    error.source_error = value.source().map(|e| Box::new(Error::from_error(e)));
    error
  }

  pub fn is_error(&self) -> bool {
    self.severity == Severity::Error
  }
  pub fn is_warn(&self) -> bool {
    self.severity == Severity::Warning
  }

  pub fn wrap_err<D>(self, msg: D) -> Self
  where
    D: std::fmt::Display,
  {
    Self(Box::new(ErrorData {
      message: msg.to_string(),
      source_error: Some(Box::new(self)),
      ..Default::default()
    }))
  }
}

impl Display for Error {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", &self.message)
  }
}

impl std::error::Error for Error {
  fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
    self
      .source_error
      .as_ref()
      .map(|e| e as &(dyn std::error::Error + 'static))
  }
}

impl MietteDiagnostic for Error {
  fn code(&self) -> Option<Box<dyn Display + '_>> {
    self
      .code
      .as_ref()
      .map(Box::new)
      .map(|c| c as Box<dyn Display>)
  }

  fn severity(&self) -> Option<miette::Severity> {
    match self.severity {
      Severity::Error => Some(miette::Severity::Error),
      Severity::Warning => Some(miette::Severity::Warning),
    }
  }

  fn help(&self) -> Option<Box<dyn Display + '_>> {
    self
      .help
      .as_ref()
      .map(Box::new)
      .map(|c| c as Box<dyn Display>)
  }

  fn url(&self) -> Option<Box<dyn Display + '_>> {
    self
      .url
      .as_ref()
      .map(Box::new)
      .map(|c| c as Box<dyn Display>)
  }

  fn source_code(&self) -> Option<&dyn miette::SourceCode> {
    self.src.as_ref().map(|s| s as &dyn miette::SourceCode)
  }

  fn labels(&self) -> Option<Box<dyn Iterator<Item = LabeledSpan> + '_>> {
    let Some(labels) = &self.labels else {
      return None;
    };
    Some(Box::new(labels.iter().map(|item| {
      LabeledSpan::new(item.label.clone(), item.offset, item.len)
    })))
  }

  fn diagnostic_source(&self) -> Option<&dyn MietteDiagnostic> {
    self
      .source_error
      .as_ref()
      .map(|s| &**s as &dyn MietteDiagnostic)
  }
}

macro_rules! impl_from_error {
  ($($t:ty),*) => {
    $(
      impl From<$t> for Error {
        fn from(value: $t) -> Error {
          Error::from_error(value)
        }
      }
    ) *
  }
}

impl_from_error! {
    std::fmt::Error,
    std::io::Error,
    std::string::FromUtf8Error
}

impl<T> From<std::sync::mpsc::SendError<T>> for Error {
  fn from(value: std::sync::mpsc::SendError<T>) -> Self {
    Error::from_error(value)
  }
}

impl From<anyhow::Error> for Error {
  fn from(value: anyhow::Error) -> Self {
    let mut error = Error::error(value.to_string());
    error.source_error = value.source().map(|e| Box::new(Error::from_error(e)));
    error
  }
}
