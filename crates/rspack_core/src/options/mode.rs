#[derive(Clone, Copy, Debug, PartialEq)]
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

impl From<String> for Mode {
  fn from(value: String) -> Self {
    match value.as_ref() {
      "none" => Self::None,
      "development" => Self::Development,
      _ => Self::Production,
    }
  }
}
