use std::{
  fmt::{Debug, Display},
  hash::Hash,
};

#[rspack_cacheable::cacheable]
#[derive(Default, Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum DependencyCategory {
  #[default]
  Unknown,
  Esm,
  CommonJS,
  Amd,
  Url,
  CssImport,
  CssCompose,
  CssExport,
  CssLocalIdent,
  Wasm,
  Worker,
  LoaderImport,
  ClientReference,
}

impl From<&str> for DependencyCategory {
  fn from(value: &str) -> Self {
    match value {
      "esm" => Self::Esm,
      "commonjs" => Self::CommonJS,
      "url" => Self::Url,
      "wasm" => Self::Wasm,
      "css-import" => Self::CssImport,
      "css-export" => Self::CssExport,
      "css-compose" => Self::CssCompose,
      "css-local-ident" => Self::CssLocalIdent,
      "loaderImport" => Self::LoaderImport,
      "worker" => Self::Worker,
      "unknown" => Self::Unknown,
      "client-reference" => Self::Unknown,
      _ => Self::Unknown,
    }
  }
}

impl DependencyCategory {
  pub fn as_str(&self) -> &'static str {
    match self {
      DependencyCategory::Unknown => "unknown",
      DependencyCategory::Esm => "esm",
      DependencyCategory::CommonJS => "commonjs",
      DependencyCategory::Amd => "amd",
      DependencyCategory::Url => "url",
      DependencyCategory::CssImport => "css-import",
      DependencyCategory::CssCompose => "css-compose",
      DependencyCategory::CssExport => "css-export",
      DependencyCategory::CssLocalIdent => "css-local-ident",
      DependencyCategory::Wasm => "wasm",
      DependencyCategory::Worker => "worker",
      DependencyCategory::LoaderImport => "loaderImport",
      DependencyCategory::ClientReference => "client-reference",
    }
  }
}

impl Display for DependencyCategory {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.as_str())
  }
}
