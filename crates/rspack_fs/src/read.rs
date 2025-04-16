use std::fmt::Debug;

use rspack_paths::{Utf8Path, Utf8PathBuf};

use crate::{FileMetadata, Result};

#[async_trait::async_trait]
pub trait ReadableFileSystem: Debug + Send + Sync {
  /// See [std::fs::read]
  async fn read(&self, path: &Utf8Path) -> Result<Vec<u8>>;
  fn read_sync(&self, path: &Utf8Path) -> Result<Vec<u8>>;

  /// See [std::fs::metadata]
  async fn metadata(&self, path: &Utf8Path) -> Result<FileMetadata>;
  fn metadata_sync(&self, path: &Utf8Path) -> Result<FileMetadata>;

  /// See [std::fs::symlink_metadata]
  async fn symlink_metadata(&self, path: &Utf8Path) -> Result<FileMetadata>;

  /// See [std::fs::canonicalize]
  async fn canonicalize(&self, path: &Utf8Path) -> Result<Utf8PathBuf>;

  /// Read the entries of a directory synchronously
  ///
  /// Returns a Vec of entry names in the directory
  async fn read_dir(&self, dir: &Utf8Path) -> Result<Vec<String>>;
  fn read_dir_sync(&self, dir: &Utf8Path) -> Result<Vec<String>>;
}
