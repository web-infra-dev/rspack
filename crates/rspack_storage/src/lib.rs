//! Rspack persistent cache storage layer
//!
//! Provides two storage implementations:
//! - `FileSystemStorage`: File system-based persistent storage using pack file format
//! - `MemoryStorage`: Memory-based temporary storage for testing or non-persistent scenarios

mod error;
mod filesystem;
mod memory;

pub use self::{
  error::{Error, Result},
  filesystem::{FileSystemOptions, FileSystemStorage, Version},
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
  fn set(&mut self, scope: &'static str, key: Vec<u8>, value: Vec<u8>);

  /// Removes a key from the specified scope (staged in memory)
  fn remove(&mut self, scope: &'static str, key: &[u8]);

  /// Enqueues a persistence operation, writing all staged memory changes to storage.
  ///
  /// The write and any follow-up storage maintenance are performed asynchronously
  /// in the background. Call [`Storage::flush`] to wait until all enqueued work
  /// has completed.
  fn save(&mut self);

  /// Waits until all work enqueued by previous [`Storage::save`] calls has completed.
  ///
  /// Must be called before process exit to ensure no background I/O is lost.
  async fn flush(&self);

  /// Starts best-effort cleanup for stale storage internals.
  ///
  /// This cleanup is intentionally detached from [`Storage::flush`].
  fn cleanup_stale(&self) {}

  /// Resets the specified scope, clearing all its data.
  ///
  /// The clean is performed asynchronously in the background. Call [`Storage::flush`]
  /// to wait until all enqueued work has completed.
  fn reset(&mut self, scope: &'static str);

  /// Gets a list of all available scopes in the storage
  async fn scopes(&self) -> Result<Vec<String>>;
}

/// Box-wrapped Storage trait object
pub type BoxStorage = Box<dyn Storage>;
