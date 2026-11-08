mod simd;

use std::fmt::Display;

pub use simd::*;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum ModuleType {
  #[default]
  CommonJs,
  Module,
}

impl Display for ModuleType {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::Module => write!(f, "module"),
      Self::CommonJs => write!(f, "commonjs"),
    }
  }
}

impl TryFrom<&str> for ModuleType {
  type Error = &'static str;
  fn try_from(value: &str) -> Result<Self, Self::Error> {
    match value {
      "module" => Ok(Self::Module),
      "commonjs" => Ok(Self::CommonJs),
      _ => Err("Invalid module type"),
    }
  }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SideEffects {
  Bool(bool),
  String(String),
  Array(Vec<String>),
}

pub fn off_to_location(json: &str, offset: usize) -> (usize, usize) {
  let mut line = 0;
  let mut col = 0;
  let mut current_offset = 0;
  for ch in json.chars() {
    let b = ch.len_utf8();
    current_offset += b;
    if ch == '\n' {
      line += 1;
      col = 0;
    } else {
      col += b;
    }

    if current_offset >= offset {
      break;
    }
  }
  (line + 1, col + 1)
}
