use std::sync::Arc;

use rspack_fs::IntermediateFileSystem;
pub use rspack_storage::{BoxStorage, CacheDirectory, MemoryStorage, Storage};
use rspack_storage::{FileSystemOptions, FileSystemStorage};

use crate::cache::StorageOptions;

pub fn create_storage(
  options: StorageOptions,
  cache_directory: CacheDirectory,
  max_age: u64,
  fs: Arc<dyn IntermediateFileSystem>,
) -> BoxStorage {
  match options {
    StorageOptions::FileSystem { directory } => {
      let option = FileSystemOptions {
        directory,
        cache_directory,
        max_pack_size: 500 * 1024,
        expire: max_age,
        fs,
      };
      Box::new(FileSystemStorage::new(option))
    }
  }
}
