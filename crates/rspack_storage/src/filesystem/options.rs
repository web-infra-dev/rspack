use std::sync::Arc;

use rspack_fs::IntermediateFileSystem;
use rspack_paths::Utf8PathBuf;

/// File system storage configuration options
#[derive(Debug)]
pub struct FileSystemOptions {
  /// Storage root directory path. Filesystem cache entries are stored under
  /// `<directory>/<version>`.
  pub directory: Utf8PathBuf,
  /// Version identifier for the specific DB instance within directory
  pub version: String,
  /// Maximum pack file size (bytes), creates new pack file when exceeded
  pub max_pack_size: usize,
  /// Data expiration time (seconds), 0 means never expire
  pub expire: u64,
  /// Maximum number of versions retained in the storage directory. 0 means disabled.
  pub max_versions: u32,
  /// File system implementation
  pub fs: Arc<dyn IntermediateFileSystem>,
}
