#[cfg(target_family = "wasm")]
mod noop;
#[cfg(not(target_family = "wasm"))]
mod turbo;

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

#[cfg(target_family = "wasm")]
pub type DatabaseValue = std::sync::Arc<[u8]>;
#[cfg(target_family = "wasm")]
pub use noop::NoopDatabase as Database;
#[cfg(not(target_family = "wasm"))]
pub type DatabaseValue = turbo_persistence::ArcBytes;
#[cfg(not(target_family = "wasm"))]
pub use turbo::TurboDatabase as Database;
