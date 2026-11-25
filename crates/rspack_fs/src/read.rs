use std::fmt::Debug;

use rspack_paths::{Utf8Path, Utf8PathBuf};

use crate::{Error, FileMetadata, FilePermissions, Result};

#[async_trait::async_trait]
pub trait ReadableFileSystem: Debug + Send + Sync {
  /// See [std::fs::read]
  async fn read(&self, path: &Utf8Path) -> Result<Vec<u8>>;
  fn read_sync(&self, path: &Utf8Path) -> Result<Vec<u8>>;

  // See [std::fs::read_to_string]
  async fn read_to_string(&self, path: &Utf8Path) -> Result<String> {
    let data = self.read(path).await?;
    String::from_utf8(data).map_err(|_| {
      Error::Io(std::io::Error::new(
        std::io::ErrorKind::InvalidData,
        "stream did not contain valid UTF-8",
      ))
    })
  }

  fn read_to_string_sync(&self, path: &Utf8Path) -> Result<String> {
    let data = self.read_sync(path)?;
    String::from_utf8(data).map_err(|_| {
      Error::Io(std::io::Error::new(
        std::io::ErrorKind::InvalidData,
        "stream did not contain valid UTF-8",
      ))
    })
  }

  /// Whether this path is supplied by a virtual overlay, including directories
  /// containing virtual files.
  fn is_virtual(&self, _path: &Utf8Path) -> bool {
    false
  }

  /// See [std::fs::metadata]
  async fn metadata(&self, path: &Utf8Path) -> Result<FileMetadata>;
  fn metadata_sync(&self, path: &Utf8Path) -> Result<FileMetadata>;

  /// Read metadata for preserving file attributes when copying a file.
  ///
  /// Native PnP reads ZIP entry attributes here so `metadata` can keep serving
  /// resolution and snapshot probes from the archive's file types.
  async fn metadata_with_file_attributes(&self, path: &Utf8Path) -> Result<FileMetadata> {
    self.metadata(path).await
  }

  /// See [std::fs::symlink_metadata]
  async fn symlink_metadata(&self, path: &Utf8Path) -> Result<FileMetadata>;

  /// See [std::fs::read_link].
  async fn read_link(&self, path: &Utf8Path) -> Result<Utf8PathBuf>;

  /// See [std::fs::canonicalize]
  async fn canonicalize(&self, path: &Utf8Path) -> Result<Utf8PathBuf>;

  /// Read the entries of a directory synchronously
  ///
  /// Returns a Vec of entry names in the directory
  async fn read_dir(&self, dir: &Utf8Path) -> Result<Vec<String>>;
  fn read_dir_sync(&self, dir: &Utf8Path) -> Result<Vec<String>>;

  /// In [std::fs], the file permission is saved in file metadata.
  /// We use this extra function to improve performance because it's rarely used.
  ///
  /// Returns `None` if the filesystem doesn't support permissions.
  async fn permissions(&self, path: &Utf8Path) -> Result<Option<FilePermissions>>;
}
