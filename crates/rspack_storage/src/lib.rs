//! Rspack persistent cache storage layer
//!
//! Provides two storage implementations:
//! - `FileSystemStorage`: File system-based persistent storage using pack file format
//! - `MemoryStorage`: Memory-based temporary storage for testing or non-persistent scenarios

mod error;
mod filesystem;
mod memory;

use std::sync::Arc;

pub use self::{
  error::{Error, Result},
  filesystem::{FileSystemOptions, FileSystemStorage},
  memory::MemoryStorage,
};

/// Persistent storage abstraction interface
///
/// Provides scope-grouped key-value storage with batch operations and async persistence
#[async_trait::async_trait]
pub trait Storage: std::fmt::Debug + Sync + Send {
  /// Loads all key-value pairs from the specified scope
  async fn load(&self, scope: &'static str) -> Result<Vec<(Vec<u8>, Vec<u8>)>>;

  /// Sets a key-value pair in the specified scope (staged in memory)
  fn set(&self, scope: &'static str, key: Vec<u8>, value: Vec<u8>);

  /// Removes a key from the specified scope (staged in memory)
  fn remove(&self, scope: &'static str, key: &[u8]);

  /// Enqueues a persistence operation, writing all staged memory changes to storage.
  ///
  /// The write is performed asynchronously in the background. Call [`Storage::flush`]
  /// to wait until all enqueued writes have completed.
  async fn save(&self) -> Result<()>;

  /// Waits until all previously enqueued [`Storage::save`] operations have completed.
  ///
  /// Must be called before process exit to ensure no background I/O is lost.
  async fn flush(&self);

  /// Resets the storage, clearing all data
  async fn reset(&self);

  /// Gets a list of all available scopes in the storage
  async fn scopes(&self) -> Result<Vec<String>>;
}

/// Arc-wrapped Storage trait object
pub type ArcStorage = Arc<dyn Storage>;
