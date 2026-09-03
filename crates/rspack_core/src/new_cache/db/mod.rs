mod noop;
#[cfg(not(target_family = "wasm"))]
mod turbo;

pub use noop::NoopDatabase;
use rspack_error::Result;

use crate::new_cache::CacheKey;

#[derive(Debug, Clone, Copy)]
pub enum DatabaseFamily {
  Cache,
  Validator,
  Meta,
}

impl DatabaseFamily {
  pub const COUNT: usize = 3;

  pub const fn index(self) -> usize {
    match self {
      Self::Cache => 0,
      Self::Validator => 1,
      Self::Meta => 2,
    }
  }
}

#[cfg(target_family = "wasm")]
pub type DatabaseValue = std::sync::Arc<[u8]>;
#[cfg(target_family = "wasm")]
pub use noop::NoopDatabase as TurboDatabase;
#[cfg(not(target_family = "wasm"))]
pub type DatabaseValue = turbo_persistence::ArcBytes;
#[cfg(not(target_family = "wasm"))]
pub use turbo::TurboDatabase;

pub(crate) trait Database: Send + Sync {
  fn get(&self, family: DatabaseFamily, key: &CacheKey) -> Result<Option<DatabaseValue>>;

  fn is_empty(&self) -> bool;

  fn write_batch(&self, writes: Vec<(DatabaseFamily, CacheKey, Vec<u8>)>) -> Result<()>;

  fn compact(&self) -> Result<()>;

  fn reset(&mut self) -> Result<()>;

  fn cleanup_stale(&self) -> Result<()>;

  fn shutdown(&self) -> Result<()>;
}
