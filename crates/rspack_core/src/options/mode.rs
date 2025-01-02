#[derive(Clone, Copy, Debug, PartialEq, Hash)]
pub enum Mode {
  Development,
  Production,
  None,
}

impl Mode {
  pub fn is_development(&self) -> bool {
    matches!(self, Mode::Development)
  }
}

impl<T: AsRef<str>> From<T> for Mode {
  fn from(value: T) -> Self {
    match value.as_ref() {
      "none" => Self::None,
      "development" => Self::Development,
      _ => Self::Production,
    }
  }
}
