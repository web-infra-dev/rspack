use std::fmt::Display;

use miette::{Diagnostic as MietteDiagnostic, LabeledSpan};
use rspack_cacheable::cacheable;

/// Error severity. Defaults to [`Severity::Error`].
#[cacheable]
#[derive(Debug, Clone, Default, Copy, PartialEq, Eq, Hash)]
pub enum Severity {
  #[default]
  Error,
  Warning,
}

/// Label for source code.
#[cacheable]
#[derive(Debug, Clone, Default)]
pub struct Label {
  /// Label name.
  pub name: Option<String>,
  /// Source code offset.
  pub offset: usize,
  /// Length of impact.
  pub len: usize,
}

/// Core error type.
///
/// See the test case for specific usage.
#[cacheable]
#[derive(Debug, Clone, Default)]
pub struct ErrorData {
  /// Error severity.
  pub severity: Severity,
  /// Message.
  pub message: String,
  /// Source code.
  pub src: Option<String>,
  /// Labels displayed in source code.
  ///
  /// The source code block will be displayed only if both source code and labels exist.
  pub labels: Option<Vec<Label>>,
  /// Help text.
  pub help: Option<String>,
  /// Source error.
  #[cacheable(omit_bounds)]
  pub source_error: Option<Box<Error>>,
  /// Error Code.
  ///
  /// This field is used to distinguish error types and will not be used for error display.
  pub code: Option<String>,
  /// Detail info.
  ///
  /// This field is used to save extra info when hide stack and will not be used for error display.
  /// TODO: remove this field and hide stack, just use stack field.
  pub details: Option<String>,
  /// Error stack.
  pub stack: Option<String>,
  /// Whether to hide the stack.
  ///
  /// TODO: replace Option<bool> to bool.
  pub hide_stack: Option<bool>,
}

/// ErrorData wrapper type.
///
/// Wrap ErrorData to avoid result_large_err.
#[cacheable]
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
      name: None,
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

  fn source_code(&self) -> Option<&dyn miette::SourceCode> {
    self.src.as_ref().map(|s| s as &dyn miette::SourceCode)
  }

  fn labels(&self) -> Option<Box<dyn Iterator<Item = LabeledSpan> + '_>> {
    let Some(labels) = &self.labels else {
      return None;
    };
    Some(Box::new(labels.iter().map(|item| {
      LabeledSpan::new(item.name.clone(), item.offset, item.len)
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

#[cfg(test)]
mod test {
  use super::{Error, ErrorData, Label};
  use crate::{Renderer, Severity};
  #[test]
  fn should_error_display() {
    let renderer = Renderer::new(false);
    let sub_err = Error(Box::new(ErrorData {
      severity: Severity::Warning,
      message: "An unexpected keyword.".into(),
      src: Some("const a = { const };\nconst b = { var };".into()),
      labels: Some(vec![
        Label {
          name: Some("keyword 1".into()),
          offset: 12,
          len: 5,
        },
        Label {
          name: Some("keyword 2".into()),
          offset: 33,
          len: 3,
        },
      ]),
      help: Some("Maybe you should remove it.".into()),
      source_error: None,
      code: Some("ModuleAnalysisWarning".into()),
      details: Some("detail info".into()),
      stack: Some("stack info".into()),
      hide_stack: None,
    }));
    let mid_err = Error(Box::new(ErrorData {
      severity: Severity::Error,
      message: "Can not parse current module.".into(),
      src: Some("const a = { const };".into()),
      labels: Some(vec![Label {
        name: Some("parse failed".into()),
        offset: 0,
        len: 1,
      }]),
      help: Some("See follow info.".into()),
      source_error: Some(Box::new(sub_err)),
      code: Some("ModuleParseError".into()),
      details: Some("detail info".into()),
      stack: Some("stack info".into()),
      hide_stack: None,
    }));
    let root_err = Error(Box::new(ErrorData {
      severity: Severity::Error,
      message: "Build Module Failed".into(),
      src: None,
      labels: None,
      help: None,
      source_error: Some(Box::new(mid_err)),
      code: Some("ModuleBuildError".into()),
      details: Some("detail info".into()),
      stack: Some("stack info".into()),
      hide_stack: None,
    }));
    let expect_display = r#"
 × Build Module Failed
  ├─▶   × Can not parse current module.
  │      ╭────
  │    1 │ const a = { const };
  │      · ┬
  │      · ╰── parse failed
  │      ╰────
  │     help: See follow info.
  │   
  ╰─▶   ⚠ An unexpected keyword.
         ╭─[1:12]
       1 │ const a = { const };
         ·             ──┬──
         ·               ╰── keyword 1
       2 │ const b = { var };
         ·             ─┬─
         ·              ╰── keyword 2
         ╰────
        help: Maybe you should remove it.
"#;
    assert_eq!(
      renderer.render(&root_err).unwrap().trim(),
      expect_display.trim()
    );
  }
}
