use std::sync::Arc;

use rspack_fs::IntermediateFileSystem;
use rspack_paths::Utf8PathBuf;

use super::CacheDirectory;

/// File system storage configuration options
#[derive(Debug)]
pub struct FileSystemOptions {
  /// Storage root directory path. Filesystem cache entries are stored under
  /// `<directory>/<compiler-path-hash>`.
  pub directory: Utf8PathBuf,
  /// Compiler-path-specific subdirectory for this DB instance.
  pub cache_directory: CacheDirectory,
  /// Maximum pack file size (bytes), creates new pack file when exceeded
  pub max_pack_size: usize,
  /// Data expiration time (seconds), 0 means never expire
  pub expire: u64,
  /// File system implementation
  pub fs: Arc<dyn IntermediateFileSystem>,
}
