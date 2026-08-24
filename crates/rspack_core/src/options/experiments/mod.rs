// BE CAREFUL:
// Add more fields to this struct should result in adding new fields to options builder.
// `impl From<Experiments> for ExperimentsBuilder` should be updated.
pub mod runtime_mode {
  use std::fmt;

  #[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
  pub enum RuntimeMode {
    #[default]
    Webpack,
    Rspack,
  }

  impl fmt::Display for RuntimeMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
      match self {
        RuntimeMode::Webpack => f.write_str("webpack"),
        RuntimeMode::Rspack => f.write_str("rspack"),
      }
    }
  }
}

use runtime_mode::RuntimeMode;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct NewCacheOptions {
  pub code_generation: bool,
  pub minimize: bool,
}

impl NewCacheOptions {
  pub const fn all() -> Self {
    Self {
      code_generation: true,
      minimize: true,
    }
  }

  pub const fn is_enabled(self) -> bool {
    self.code_generation || self.minimize
  }
}

#[derive(Debug)]
pub struct Experiments {
  pub css: bool,
  pub new_cache: NewCacheOptions,
  pub defer_import: bool,
  pub source_import: bool,
  pub pure_functions: bool,
  pub runtime_mode: RuntimeMode,
}
