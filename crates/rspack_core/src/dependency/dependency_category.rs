use std::fmt::{Debug, Display};
use std::hash::Hash;

#[derive(Default, Clone, PartialEq, Eq, Hash, Debug)]
pub enum DependencyCategory {
  #[default]
  Unknown,
  Esm,
  CommonJS,
  Url,
  CssImport,
  CssCompose,
  Wasm,
  Worker,
  Custom(Box<str>),
}

impl From<&str> for DependencyCategory {
  fn from(value: &str) -> Self {
    match value {
      "esm" => Self::Esm,
      "commonjs" => Self::CommonJS,
      "url" => Self::Url,
      "wasm" => Self::Wasm,
      "css-import" => Self::CssImport,
      "css-compose" => Self::CssCompose,
      "worker" => Self::Worker,
      "unknown" => Self::Unknown,
      _ => Self::Custom(value.into()),
    }
  }
}

impl DependencyCategory {
  pub fn as_str(&self) -> &str {
    match self {
      DependencyCategory::Unknown => "unknown",
      DependencyCategory::Esm => "esm",
      DependencyCategory::CommonJS => "commonjs",
      DependencyCategory::Url => "url",
      DependencyCategory::CssImport => "css-import",
      DependencyCategory::CssCompose => "css-compose",
      DependencyCategory::Wasm => "wasm",
      DependencyCategory::Worker => "worker",
      DependencyCategory::Custom(str) => str.as_ref(),
    }
  }
}

impl Display for DependencyCategory {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.as_str())
  }
}
