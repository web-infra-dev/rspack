use std::{env, str::FromStr};

#[derive(Clone, Copy, Debug)]
pub enum Mode {
  Development,
  Production,
}

impl FromStr for Mode {
  type Err = anyhow::Error;

  fn from_str(s: &str) -> anyhow::Result<Mode> {
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
