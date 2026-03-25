use std::sync::Arc;

use rspack_fs::{FileMetadata, IntermediateFileSystem, ReadStream, WriteStream};
use rspack_paths::{Utf8Path, Utf8PathBuf};

use crate::{Error, Result};

pub type Reader = Box<dyn ReadStream>;
pub type Writer = Box<dyn WriteStream>;

/// Scoped file system wrapper
///
/// Confines all file operations to a specified workspace directory,
/// automatically handles relative path conversion, and provides a unified file operation interface.
#[derive(Debug, Clone)]
pub struct ScopeFileSystem {
  /// Workspace root path
  workspace: Utf8PathBuf,
  /// Underlying file system implementation
  fs: Arc<dyn IntermediateFileSystem>,
}

impl std::fmt::Display for ScopeFileSystem {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.workspace)
  }
}

impl ScopeFileSystem {
  /// Creates a memory-based file system
  #[cfg(test)]
  pub fn new_memory_fs(workspace: Utf8PathBuf) -> Self {
    Self {
      workspace,
      fs: Arc::new(rspack_fs::MemoryFileSystem::default()),
    }
  }

  /// Creates a new scoped file system
  pub fn new(workspace: Utf8PathBuf, fs: Arc<dyn IntermediateFileSystem>) -> Self {
    Self { workspace, fs }
  }

  /// Ensures the workspace directory exists, creates it if not
  pub async fn ensure_exist(&self) -> Result<()> {
    self.fs.create_dir_all(&self.workspace).await?;
    Ok(())
  }

  /// Removes the entire workspace directory and its contents
  pub async fn remove(&self) -> Result<()> {
    if let Err(e) = self.fs.remove_dir_all(&self.workspace).await {
      let e: Error = e.into();
      if !e.is_not_found() {
        return Err(e);
      }
    }
    Ok(())
  }

  /// Moves a file between two scoped file systems
  ///
  /// # Arguments
  /// * `from` - Source scoped file system
  /// * `to` - Target scoped file system
  /// * `relative_path` - Relative path of the file
  pub async fn move_to(
    from: &ScopeFileSystem,
    to: &ScopeFileSystem,
    relative_path: impl AsRef<Utf8Path>,
  ) -> Result<()> {
    let from_file = from.workspace.join(relative_path.as_ref());
    let to_file = to.workspace.join(relative_path.as_ref());
    if let Err(e) = from.fs.rename(&from_file, &to_file).await {
      // If the source file is not found, ignore the error.
      let e: Error = e.into();
      if !e.is_not_found() {
        return Err(e);
      }
    }
    Ok(())
  }

  /// Creates a child scoped file system
  ///
  /// Returns a new ScopeFileSystem whose workspace is a subdirectory of the current one
  pub fn child_fs(&self, relative_path: impl AsRef<Utf8Path>) -> Self {
    let workspace = self.workspace.join(relative_path);
    Self {
      workspace,
      fs: self.fs.clone(),
    }
  }

  /// Gets file or directory metadata
  pub async fn stat(&self, relative_path: impl AsRef<Utf8Path>) -> Result<FileMetadata> {
    let path = self.workspace.join(relative_path);
    let stat = self.fs.stat(&path).await?;
    Ok(stat)
  }

  /// Removes the specified file
  ///
  /// Does not return an error if the file doesn't exist
  pub async fn remove_file(&self, relative_path: impl AsRef<Utf8Path>) -> Result<()> {
    let path = self.workspace.join(relative_path);
    if let Err(e) = self.fs.remove_file(&path).await {
      let e: Error = e.into();
      if !e.is_not_found() {
        return Err(e);
      }
    }
    Ok(())
  }

  /// Writes file content
  #[cfg(test)]
  pub async fn write(&self, relative_path: impl AsRef<Utf8Path>, bytes: &[u8]) -> Result<()> {
    let path = self.workspace.join(relative_path);
    self
      .fs
      .create_dir_all(path.parent().expect("should have parent"))
      .await?;
    self.fs.write(&path, bytes).await?;
    Ok(())
  }

  /// Reads entire file content
  #[cfg(test)]
  pub async fn read(&self, relative_path: impl AsRef<Utf8Path>) -> Result<Vec<u8>> {
    let path = self.workspace.join(relative_path);
    let data = self.fs.read_file(&path).await?;
    Ok(data)
  }

  /// Creates a file read stream (for large files)
  pub async fn stream_read(&self, relative_path: impl AsRef<Utf8Path>) -> Result<Reader> {
    let path = self.workspace.join(relative_path);
    let reader = self.fs.create_read_stream(&path).await?;
    Ok(reader)
  }

  /// Creates a file write stream (for large files)
  ///
  /// If the file already exists, it will be deleted first
  pub async fn stream_write(&self, relative_path: impl AsRef<Utf8Path>) -> Result<Writer> {
    let _ = self.remove_file(&relative_path).await;

    let path = self.workspace.join(relative_path);
    self
      .fs
      .create_dir_all(path.parent().expect("should have parent"))
      .await?;
    let writer = self.fs.create_write_stream(&path).await?;
    Ok(writer)
  }

  /// Lists all direct children in the workspace directory
  pub async fn list_child(&self) -> Result<Vec<String>> {
    let children = self.fs.read_dir(&self.workspace).await?;
    Ok(children)
  }
}

#[cfg(test)]
mod tests {
  use super::{Result, ScopeFileSystem};

  #[tokio::test]
  async fn test_read_and_write() -> Result<()> {
    let fs = ScopeFileSystem::new_memory_fs("/".into());
    assert!(fs.read("/a.txt").await.is_err());

    fs.write("a.txt", "hello world".as_bytes()).await?;
    assert_eq!(fs.read("a.txt").await?, "hello world".as_bytes());
    Ok(())
  }

  #[tokio::test]
  async fn test_stream_read_and_write() -> Result<()> {
    let fs = ScopeFileSystem::new_memory_fs("/".into());
    assert!(fs.read("/a.txt").await.is_err());

    let mut writer = fs.stream_write("a.txt").await?;
    writer.write_line("hello").await?;
    writer.write("world".as_bytes()).await?;
    writer.flush().await?;

    let mut reader = fs.stream_read("a.txt").await?;
    assert_eq!(reader.read_line().await?, "hello");
    assert_eq!(reader.read_to_end().await?, "world".as_bytes());
    Ok(())
  }

  #[tokio::test]
  async fn test_move_to() -> Result<()> {
    let fs = ScopeFileSystem::new_memory_fs("/".into());
    fs.write("a.txt", "1".as_bytes()).await?;

    let temp_fs = fs.child_fs(".temp");
    temp_fs.write("a.txt", "2".as_bytes()).await?;

    // Moving a non-existent file should succeed
    ScopeFileSystem::move_to(&temp_fs, &fs, "b.txt").await?;
    assert_eq!(fs.read("a.txt").await?, "1".as_bytes());

    ScopeFileSystem::move_to(&temp_fs, &fs, "a.txt").await?;
    assert_eq!(fs.read("a.txt").await?, "2".as_bytes());

    Ok(())
  }
}
