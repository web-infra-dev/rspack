// TODO add #[cfg(test)]
mod memory;

pub use memory::MemoryStorage;

/// Storage Options
///
/// This enum contains all of supported storage options.
/// Since MemoryStorage is only used in unit test, there is no need to add it here.
#[derive(Debug, Clone)]
pub enum StorageOptions {
  // TODO change to FileSystem(configuration)
  FileSystem,
}

// TODO: add batch set/remove
pub trait Storage: std::fmt::Debug + Sync + Send {
  fn get_all(&self, scope: &str) -> Vec<(Vec<u8>, Vec<u8>)>;
  // using immutable reference to support concurrency
  fn set(&self, scope: &str, key: Vec<u8>, value: Vec<u8>);
  fn remove(&self, scope: &str, key: &[u8]);
  fn idle(&self);
}
