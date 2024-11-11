use std::fmt::Debug;

use futures::future::BoxFuture;
use rspack_paths::Utf8Path;

use crate::{FileMetadata, Result};

pub trait AsyncWritableFileSystem: Debug {
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
  fn create_dir<'a>(&'a self, dir: &'a Utf8Path) -> BoxFuture<'a, Result<()>>;

  /// Recursively create a directory and all of its parent components if they are missing.
  fn create_dir_all<'a>(&'a self, dir: &'a Utf8Path) -> BoxFuture<'a, Result<()>>;

  /// Write a slice as the entire contents of a file.
  /// This function will create a file if it does not exist, and will entirely replace its contents if it does.
  fn write<'a>(&'a self, file: &'a Utf8Path, data: &'a [u8]) -> BoxFuture<'a, Result<()>>;

  /// Removes a file from the filesystem.
  fn remove_file<'a>(&'a self, file: &'a Utf8Path) -> BoxFuture<'a, Result<()>>;

  /// Removes a directory at this path, after removing all its contents. Use carefully.
  fn remove_dir_all<'a>(&'a self, dir: &'a Utf8Path) -> BoxFuture<'a, Result<()>>;

  /// Returns a list of all files in a directory.
  fn read_dir<'a>(&'a self, dir: &'a Utf8Path) -> BoxFuture<'a, Result<Vec<String>>>;

  /// Read the entire contents of a file into a bytes vector.
  fn read_file<'a>(&'a self, file: &'a Utf8Path) -> BoxFuture<'a, Result<Vec<u8>>>;

  fn stat<'a>(&'a self, file: &'a Utf8Path) -> BoxFuture<'a, Result<FileMetadata>>;
}

pub trait AsyncReadableFileSystem: Debug {
  /// Read the entire contents of a file into a bytes vector.
  ///
  /// Error: This function will return an error if path does not already exist.
  fn read<'a>(&'a self, file: &'a Utf8Path) -> BoxFuture<'a, Result<Vec<u8>>>;
}

/// Async readable and writable file system representation.
pub trait AsyncFileSystem: AsyncReadableFileSystem + AsyncWritableFileSystem {}

// Blanket implementation for all types that implement both [`AsyncReadableFileSystem`] and [`WritableFileSystem`].
impl<T: AsyncReadableFileSystem + AsyncWritableFileSystem> AsyncFileSystem for T {}
