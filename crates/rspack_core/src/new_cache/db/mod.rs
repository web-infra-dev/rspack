#[cfg(target_family = "wasm")]
mod memory;
#[cfg(not(target_family = "wasm"))]
mod turbo;

#[cfg(target_family = "wasm")]
pub use memory::{Database, DatabaseValue};
#[cfg(not(target_family = "wasm"))]
pub use turbo::{Database, DatabaseValue};

#[derive(Debug, Clone, Copy)]
pub enum DatabaseFamily {
  Cache,
  Validator,
}

impl DatabaseFamily {
  pub const COUNT: usize = 2;

  pub const fn index(self) -> usize {
    match self {
      Self::Cache => 0,
      Self::Validator => 1,
    }
  }
}
