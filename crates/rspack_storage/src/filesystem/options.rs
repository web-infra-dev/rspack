use std::sync::Arc;

use rspack_fs::IntermediateFileSystem;
use rspack_paths::Utf8PathBuf;

/// File system storage configuration options
#[derive(Debug)]
pub struct FileSystemOptions {
  /// Storage root directory path (contains all DB versions)
  pub directory: Utf8PathBuf,
  /// Version identifier for the specific DB instance within directory
  pub version: String,
  /// Maximum pack file size (bytes), creates new pack file when exceeded
  pub max_pack_size: usize,
  /// Data expiration time (seconds), 0 means never expire
  pub expire: u64,
  /// File system implementation
  pub fs: Arc<dyn IntermediateFileSystem>,
}
