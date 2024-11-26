use std::fmt::Debug;

use futures::future::BoxFuture;
use rspack_paths::Utf8Path;

use super::{FileMetadata, Result};

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
  fn remove_file<'a>(&'a self, file: &'a Utf8Path) -> BoxFuture<'a, Result<()>>;

  /// Removes a directory at this path, after removing all its contents. Use carefully.
  fn remove_dir_all<'a>(&'a self, dir: &'a Utf8Path) -> BoxFuture<'a, Result<()>>;

  /// Returns a list of all files in a directory.
  fn read_dir<'a>(&'a self, dir: &'a Utf8Path) -> BoxFuture<'a, Result<Vec<String>>>;

  /// Read the entire contents of a file into a bytes vector.
  fn read_file<'a>(&'a self, file: &'a Utf8Path) -> BoxFuture<'a, Result<Vec<u8>>>;

  fn stat<'a>(&'a self, file: &'a Utf8Path) -> BoxFuture<'a, Result<FileMetadata>>;
}

/// Extension trait for [`AsyncWritableFileSystem`].
pub trait WritableFileSystemExt: WritableFileSystem {
  /// Remove all files and directories in the given directory except the given directory
  ///
  /// example:
  /// ```
  /// #[tokio::test]
  /// async fn remove_dir_except() {
  ///   use crate::rspack_fs::ReadableFileSystem;
  ///   use crate::rspack_fs::WritableFileSystem;
  ///   use crate::rspack_fs::WritableFileSystemExt;
  ///   use crate::rspack_paths::Utf8Path;
  ///   let fs = crate::rspack_fs::NativeFileSystem;
  ///
  ///   // adding files and directories
  ///   fs.create_dir_all(&Utf8Path::new("path/to/dir/except"))
  ///     .await
  ///     .unwrap();
  ///   fs.create_dir_all(&Utf8Path::new("path/to/dir/rm1"))
  ///     .await
  ///     .unwrap();
  ///   fs.create_dir_all(&Utf8Path::new("path/to/dir/rm2"))
  ///     .await
  ///     .unwrap();
  ///
  ///   let dir = Utf8Path::new("path/to/dir");
  ///   let except = Utf8Path::new("path/to/dir/except");
  ///
  ///   fs.remove_dir_except(&dir, &except).await.unwrap();
  ///   assert_eq!(
  ///     fs.read_dir(&dir).await.unwrap(),
  ///     vec![String::from("path/to/dir/except")]
  ///   );
  ///
  ///   fs.remove_dir_all(&dir).await.unwrap();
  /// }
  /// ```
  fn remove_dir_except<'a>(
    &'a self,
    dir: &'a Utf8Path,
    except: &'a Utf8Path,
  ) -> BoxFuture<'a, Result<()>> {
    let fut = async move {
      let mut to_clean = dir;
      while to_clean != except {
        let mut matched = None;
        for entry in self.read_dir(dir).await? {
          let path = dir.join(entry);
          if except.starts_with(&path) {
            matched = Some(except);
            continue;
          }
          if self.stat(&path).await?.is_directory {
            self.remove_dir_all(&path).await?;
          } else {
            self.remove_file(&path).await?;
          }
        }
        let Some(child_to_clean) = matched else {
          break;
        };
        to_clean = child_to_clean;
      }

      Ok(())
    };
    Box::pin(fut)
  }
}
