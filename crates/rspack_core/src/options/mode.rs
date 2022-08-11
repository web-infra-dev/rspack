use std::{env, str::FromStr, string::ParseError};

#[derive(Clone, Copy, Debug)]
pub enum Mode {
  Development,
  Production,
}

impl FromStr for Mode {
  type Err = ParseError;

  fn from_str(s: &str) -> Result<Mode, self::ParseError> {
    let mode_string = if s.is_empty() {
      match env::var("NODE_ENV") {
        Ok(value) => value,
        Err(_) => String::from("production"),
      }
    } else {
      s.to_string()
    };

    if mode_string.eq("development") {
      Ok(Self::Development)
    } else {
      Ok(Self::Production)
    }
  }
}
