use std::path::Path;

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
  fn create_dir<P: AsRef<Path>>(&self, dir: P) -> Result<()>;

  /// Recursively create a directory and all of its parent components if they are missing.
  fn create_dir_all<P: AsRef<Path>>(&self, dir: P) -> Result<()>;

  /// Write a slice as the entire contents of a file.
  /// This function will create a file if it does not exist, and will entirely replace its contents if it does.
  fn write<P: AsRef<Path>, D: AsRef<[u8]>>(&self, file: P, data: D) -> Result<()>;
}

pub trait ReadableFileSystem {
  /// Read the entire contents of a file into a bytes vector.
  ///
  /// Error: This function will return an error if path does not already exist.
  fn read<P: AsRef<Path>>(&self, file: P) -> Result<Vec<u8>>;
}

/// Readable and writable file system representation.
pub trait FileSystem: ReadableFileSystem + WritableFileSystem {}

// Blanket implementation for all types that implement both [`ReadableFileSystem`] and [`WritableFileSystem`].
impl<T: ReadableFileSystem + WritableFileSystem> FileSystem for T {}
