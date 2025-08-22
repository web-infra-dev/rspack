use crate::error::Error;

/// Multiple errors to represent different kinds of errors.
/// NEVER implement this with [miette::Diagnostic],
/// because it makes code hard to maintain.
#[derive(Debug, Default)]
pub struct BatchErrors(pub Vec<Error>);

impl BatchErrors {
  pub fn into_inner(self) -> Vec<Error> {
    self.0
  }
}

impl From<BatchErrors> for Vec<crate::Diagnostic> {
  fn from(value: BatchErrors) -> Self {
    value.0.into_iter().map(crate::Diagnostic::from).collect()
  }
}

impl From<Error> for BatchErrors {
  fn from(value: Error) -> Self {
    Self(vec![value])
  }
}

impl From<Vec<Error>> for BatchErrors {
  fn from(value: Vec<Error>) -> Self {
    Self(value)
  }
}
