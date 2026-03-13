use std::sync::Arc;

use rspack_cacheable::{cacheable, utils::PortablePath, with::As};
use rspack_fs::IntermediateFileSystem;
use rspack_paths::Utf8PathBuf;
use rspack_storage::{FileSystemOptions, FileSystemStorage};
pub use rspack_storage::{MemoryStorage, Storage};

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
  version: String,
  fs: Arc<dyn IntermediateFileSystem>,
) -> Arc<dyn Storage> {
  match options {
    StorageOptions::FileSystem { directory } => {
      let option = FileSystemOptions {
        directory,
        version,
        max_pack_size: 500 * 1024,
        expire: 7 * 24 * 60 * 60 * 1000,
        fs,
      };
      Arc::new(FileSystemStorage::new(option))
    }
  }
}
