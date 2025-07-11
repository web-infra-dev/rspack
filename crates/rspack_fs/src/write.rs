use std::fmt::Debug;

use rspack_paths::Utf8Path;

use super::{FileMetadata, Result};
use crate::file_metadata::FilePermissions;

#[async_trait::async_trait]
pub trait WritableFileSystem: Debug + Send + Sync {
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
  async fn create_dir(&self, dir: &Utf8Path) -> Result<()>;

  /// Recursively create a directory and all of its parent components if they are missing.
  async fn create_dir_all(&self, dir: &Utf8Path) -> Result<()>;

  /// Write a slice as the entire contents of a file.
  /// This function will create a file if it does not exist, and will entirely replace its contents if it does.
  async fn write(&self, file: &Utf8Path, data: &[u8]) -> Result<()>;

  /// Removes a file from the filesystem.
  async fn remove_file(&self, file: &Utf8Path) -> Result<()>;

  /// Removes a directory at this path, after removing all its contents. Use carefully.
  async fn remove_dir_all(&self, dir: &Utf8Path) -> Result<()>;

  /// Returns a list of all files in a directory.
  async fn read_dir(&self, dir: &Utf8Path) -> Result<Vec<String>>;

  /// Read the entire contents of a file into a bytes vector.
  async fn read_file(&self, file: &Utf8Path) -> Result<Vec<u8>>;

  async fn stat(&self, file: &Utf8Path) -> Result<FileMetadata>;

  /// See [std::fs::set_permissions]
  async fn set_permissions(&self, _path: &Utf8Path, _perm: FilePermissions) -> Result<()>;
}
