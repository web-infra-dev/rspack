use std::fmt::Debug;

use rspack_paths::{Utf8Path, Utf8PathBuf};

use super::{FileMetadata, Result};

pub trait SyncWritableFileSystem: Debug {
  /// Creates a new, empty directory at the provided path.
  ///
  /// NOTE: If a parent of the given path doesn’t exist, this function is supposed to return an error.
  /// To create a directory and all its missing parents at the same time, use the [`create_dir_all`] function.
  ///
  /// Error:
  /// This function is supposed to return an error in the following situations, but is not limited to just these cases:
  /// - User lacks permissions to create directory at path.
  /// - A parent of the given path doesn’t exist. (To create a directory and all its missing parents at the same time, use the create_dir_all function.)
  /// - Path already exists.
  fn create_dir(&self, dir: &Utf8Path) -> Result<()>;

  /// Recursively create a directory and all of its parent components if they are missing.
  fn create_dir_all(&self, dir: &Utf8Path) -> Result<()>;

  /// Write a slice as the entire contents of a file.
  /// This function will create a file if it does not exist, and will entirely replace its contents if it does.
  fn write(&self, file: &Utf8Path, data: &[u8]) -> Result<()>;
}

pub trait SyncReadableFileSystem: Debug + Send + Sync {
  /// See [std::fs::read]
  fn read(&self, path: &Utf8Path) -> Result<Vec<u8>>;

  /// See [std::fs::metadata]
  fn metadata(&self, path: &Utf8Path) -> Result<FileMetadata>;

  /// See [std::fs::symlink_metadata]
  fn symlink_metadata(&self, path: &Utf8Path) -> Result<FileMetadata>;

  /// See [std::fs::canonicalize]
  fn canonicalize(&self, path: &Utf8Path) -> Result<Utf8PathBuf>;
}

/// Readable and writable file system representation.
pub trait SyncFileSystem: SyncReadableFileSystem + SyncWritableFileSystem {}

// Blanket implementation for all types that implement both [`ReadableFileSystem`] and [`WritableFileSystem`].
impl<T: SyncReadableFileSystem + SyncWritableFileSystem> SyncFileSystem for T {}
