use std::{num::NonZeroU32, sync::Arc};

use rspack_cacheable::{
  cacheable,
  utils::PortablePath,
  with::{As, Skip},
};
use rspack_fs::IntermediateFileSystem;
use rspack_paths::Utf8PathBuf;
pub use rspack_storage::{BoxStorage, MemoryStorage, Storage};
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
    #[cacheable(with=Skip)]
    max_versions: Option<NonZeroU32>,
  },
}

pub fn create_storage(
  options: StorageOptions,
  version: String,
  fs: Arc<dyn IntermediateFileSystem>,
) -> BoxStorage {
  match options {
    StorageOptions::FileSystem {
      directory,
      max_versions,
    } => {
      let option = FileSystemOptions {
        directory,
        version,
        max_versions,
        max_pack_size: 500 * 1024,
        expire: 7 * 24 * 60 * 60,
        fs,
      };
      Box::new(FileSystemStorage::new(option))
    }
  }
}
