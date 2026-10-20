use std::sync::Arc;

use rspack_fs::IntermediateFileSystem;
pub use rspack_storage::{BoxStorage, CacheDirectory, MemoryStorage, Storage};
use rspack_storage::{FileSystemOptions, FileSystemStorage};

use crate::StorageOptions;

pub fn create_storage(
  options: StorageOptions,
  cache_directory: CacheDirectory,
  max_age: u64,
  fs: Arc<dyn IntermediateFileSystem>,
  seed_from: Option<&str>,
) -> (BoxStorage, Option<String>) {
  match options {
    StorageOptions::FileSystem { location, .. } => {
      let seed_message = seed_from.map(|donor| {
        match rspack_storage::seed_cache(
          location.as_std_path(),
          std::path::Path::new(donor),
          &cache_directory,
        ) {
          Ok(outcome) => format!("portable cache seed from {donor}: {outcome}"),
          Err(error) => format!("portable cache seed from {donor} skipped: {error}"),
        }
      });
      let option = FileSystemOptions {
        directory: location,
        cache_directory,
        max_pack_size: 500 * 1024,
        expire: max_age,
        fs,
      };
      (Box::new(FileSystemStorage::new(option)), seed_message)
    }
  }
}
