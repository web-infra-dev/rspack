use std::fmt::Debug;

use futures::future::BoxFuture;
use rspack_paths::Utf8Path;
use rspack_paths::Utf8PathBuf;

use crate::{FileMetadata, Result};
pub trait ReadableFileSystem: Debug + Send + Sync {
  /// See [std::fs::read]
  fn read(&self, path: &Utf8Path) -> Result<Vec<u8>>;

  /// See [std::fs::metadata]
  fn metadata(&self, path: &Utf8Path) -> Result<FileMetadata>;

  /// See [std::fs::symlink_metadata]
  fn symlink_metadata(&self, path: &Utf8Path) -> Result<FileMetadata>;

  /// See [std::fs::canonicalize]
  fn canonicalize(&self, path: &Utf8Path) -> Result<Utf8PathBuf>;
  /// Read the entire contents of a file into a bytes vector.
  ///
  /// Error: This function will return an error if path does not already exist.
  fn async_read<'a>(&'a self, file: &'a Utf8Path) -> BoxFuture<'a, Result<Vec<u8>>>;
}
