use std::path::Path;

use futures::future::BoxFuture;

use crate::Result;

pub trait AsyncWritableFileSystem {
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
  fn create_dir<P: AsRef<Path>>(&self, dir: P) -> BoxFuture<'_, Result<()>>;

  /// Recursively create a directory and all of its parent components if they are missing.
  fn create_dir_all<P: AsRef<Path>>(&self, dir: P) -> BoxFuture<'_, Result<()>>;

  /// Write a slice as the entire contents of a file.
  /// This function will create a file if it does not exist, and will entirely replace its contents if it does.
  fn write<P: AsRef<Path>, D: AsRef<[u8]>>(&self, file: P, data: D) -> BoxFuture<'_, Result<()>>;

  fn remove_dir_all<P: AsRef<Path>>(&self, dir: P) -> BoxFuture<'_, Result<()>>;
}

pub trait AsyncReadableFileSystem {
  /// Read the entire contents of a file into a bytes vector.
  ///
  /// Error: This function will return an error if path does not already exist.
  fn read<P: AsRef<Path>>(&self, file: P) -> BoxFuture<'_, Result<Vec<u8>>>;
}

/// Async readable and writable file system representation.
pub trait AsyncFileSystem: AsyncReadableFileSystem + AsyncWritableFileSystem {}

// Blanket implementation for all types that implement both [`AsyncReadableFileSystem`] and [`WritableFileSystem`].
impl<T: AsyncReadableFileSystem + AsyncWritableFileSystem> AsyncFileSystem for T {}
