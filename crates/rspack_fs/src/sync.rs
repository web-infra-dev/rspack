use std::fs::Metadata;
use std::io;
use std::path::Path;
use std::path::PathBuf;

use rspack_paths::Utf8Path;

// pubResolverFileSystem
use super::Result;

pub trait WritableFileSystem {
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

pub trait ReadableFileSystem: Send + Sync {
  /// See [std::fs::read]
  fn read(&self, path: &Path) -> io::Result<Vec<u8>>;

  /// See [std::fs::metadata]
  fn metadata(&self, path: &Path) -> io::Result<Metadata>;

  /// See [std::fs::symlink_metadata]
  fn symlink_metadata(&self, path: &Path) -> io::Result<Metadata>;

  /// See [std::fs::canonicalize]
  fn canonicalize(&self, path: &Path) -> io::Result<PathBuf>;
}

/// Readable and writable file system representation.
pub trait FileSystem: ReadableFileSystem + WritableFileSystem {}

// Blanket implementation for all types that implement both [`ReadableFileSystem`] and [`WritableFileSystem`].
impl<T: ReadableFileSystem + WritableFileSystem> FileSystem for T {}
