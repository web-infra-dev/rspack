// TODO add #[cfg(test)]
mod memory;

use std::sync::Arc;

pub use memory::MemoryStorage;
use rspack_fs::IntermediateFileSystem;
pub use rspack_storage::{PackBridgeFS, PackStorage, PackStorageOptions, Storage};

/// Storage Options
///
/// This enum contains all of supported storage options.
/// Since MemoryStorage is only used in unit test, there is no need to add it here.
#[derive(Debug, Clone)]
pub enum StorageOptions {
  // TODO change to FileSystem(configuration)
  FileSystem(PackStorageOptions),
}

pub fn create_storage(
  options: StorageOptions,
  fs: Arc<dyn IntermediateFileSystem>,
) -> Arc<dyn Storage> {
  match options {
    StorageOptions::FileSystem(o) => Arc::new(PackStorage::new(o, Arc::new(PackBridgeFS(fs)))),
  }
}
