use std::sync::Arc;

use rspack_cacheable::{cacheable, utils::PortablePath, with::As};
use rspack_fs::IntermediateFileSystem;
use rspack_paths::Utf8PathBuf;
pub use rspack_storage::{BoxStorage, MemoryStorage, Storage, Version};
use rspack_storage::{FileSystemOptions, FileSystemStorage};

/// Storage Options
///
/// This enum contains all of supported storage options.
#[cacheable]
#[derive(Debug, Clone, Hash)]
pub enum StorageOptions {
  FileSystem {
    #[cacheable(with=As<PortablePath>)]
    directory: Utf8PathBuf,
  },
}

pub fn create_storage(
  options: StorageOptions,
  version: Version,
  max_age: u64,
  max_versions: u32,
  fs: Arc<dyn IntermediateFileSystem>,
) -> BoxStorage {
  match options {
    StorageOptions::FileSystem { directory } => {
      let option = FileSystemOptions {
        directory,
        version,
        max_versions,
        max_pack_size: 500 * 1024,
        expire: max_age,
        fs,
      };
      Box::new(FileSystemStorage::new(option))
    }
  }
}
