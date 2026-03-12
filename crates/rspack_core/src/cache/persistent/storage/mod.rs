use std::sync::Arc;

use rspack_cacheable::{cacheable, utils::PortablePath, with::As};
use rspack_fs::IntermediateFileSystem;
use rspack_paths::Utf8PathBuf;
pub use rspack_storage::Storage;
use rspack_storage::{FileSystemOptions, FileSystemStorage, MemoryStorage};

/// Storage Options
///
/// This enum contains all of supported storage options.
/// NOTE. MemoryStorage is only used in unit test
#[cacheable]
#[derive(Debug, Clone, Hash)]
pub enum StorageOptions {
  FileSystem {
    #[cacheable(with=As<PortablePath>)]
    directory: Utf8PathBuf,
  },
  Memory,
}

pub fn create_storage(
  options: StorageOptions,
  version: String,
  fs: Arc<dyn IntermediateFileSystem>,
) -> Arc<dyn Storage> {
  match options {
    StorageOptions::FileSystem { directory } => {
      let option = FileSystemOptions {
        directory: directory.join(version),
        max_pack_size: 500 * 1024,
        expire: 7 * 24 * 60 * 60 * 1000,
        fs: fs,
      };
      Arc::new(FileSystemStorage::new(option))
    }
    StorageOptions::Memory => Arc::new(MemoryStorage::default()),
  }
}
