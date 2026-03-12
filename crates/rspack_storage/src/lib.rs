//! Rspack persistent cache storage layer
//!
//! Provides two storage implementations:
//! - `FileSystemStorage`: File system-based persistent storage using pack file format
//! - `MemoryStorage`: Memory-based temporary storage for testing or non-persistent scenarios

mod error;
mod filesystem;
mod memory;

use std::sync::Arc;

use tokio::sync::oneshot::Receiver;

pub use self::{
  error::{Error, Result},
  filesystem::{FileSystemOptions, FileSystemStorage},
  memory::MemoryStorage,
};

/// Storage key type
pub type Key = Vec<u8>;
/// Storage value type
pub type Value = Vec<u8>;
/// Key-value pair collection
pub type KVPairs<V = Value> = Vec<(Key, V)>;

/// Persistent storage abstraction interface
///
/// Provides scope-grouped key-value storage with batch operations and async persistence
#[async_trait::async_trait]
pub trait Storage: std::fmt::Debug + Sync + Send {
  /// Loads all key-value pairs from the specified scope
  async fn load(&self, scope: &'static str) -> Result<KVPairs>;

  /// Sets a key-value pair in the specified scope (staged in memory)
  fn set(&self, scope: &'static str, key: Key, value: Value);

  /// Removes a key from the specified scope (staged in memory)
  fn remove(&self, scope: &'static str, key: &[u8]);

  /// Triggers persistence operation, saving memory changes to storage
  ///
  /// Returns a Receiver to asynchronously receive the save result
  fn trigger_save(&self) -> Result<Receiver<Result<()>>>;

  /// Resets the storage, clearing all data
  async fn reset(&self);

  /// Gets a list of all available scopes in the storage
  async fn scopes(&self) -> Result<Vec<String>>;
}

/// Arc-wrapped Storage trait object
pub type ArcStorage = Arc<dyn Storage>;
