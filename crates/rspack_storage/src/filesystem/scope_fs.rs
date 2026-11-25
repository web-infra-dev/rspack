use std::sync::Arc;

#[cfg(test)]
use cow_utils::CowUtils;
use rspack_fs::{AbsPath, FileMetadata, IntermediateFileSystem, ReadStream, RelPath, WriteStream};

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
  workspace: AbsPath,
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
  pub fn new_memory_fs(workspace: &str) -> Self {
    let workspace = if cfg!(windows) && workspace.starts_with('/') {
      if workspace == "/" {
        AbsPath::new(r"C:\").expect("valid windows root")
      } else {
        let rest = workspace.trim_start_matches('/').cow_replace('/', "\\");
        AbsPath::new(&format!("C:\\{rest}")).expect("valid windows test workspace")
      }
    } else {
      AbsPath::new(workspace).expect("valid test workspace")
    };
    Self {
      workspace,
      fs: Arc::new(rspack_fs::MemoryFileSystem::default()),
    }
  }

  /// Creates a new scoped file system
  pub fn new(workspace: AbsPath, fs: Arc<dyn IntermediateFileSystem>) -> Self {
    Self { workspace, fs }
  }

  fn resolve_path(&self, relative_path: &RelPath) -> Result<AbsPath> {
    if relative_path.escapes_root() {
      return Err(rspack_fs::Error::PathEscapesRoot(relative_path.as_str().to_string()).into());
    }
    Ok(self.workspace.join_rel(relative_path))
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

  /// Moves a file or directory between two scoped file systems
  ///
  /// # Arguments
  /// * `from` - Source scoped file system
  /// * `to` - Target scoped file system
  /// * `relative_path` - Relative path of the file or directory
  pub async fn move_to(
    from: &ScopeFileSystem,
    to: &ScopeFileSystem,
    relative_path: &RelPath,
  ) -> Result<()> {
    let from_file = from.resolve_path(relative_path)?;
    let to_file = to.resolve_path(relative_path)?;
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
  pub fn child_fs(&self, relative_path: &RelPath) -> Result<Self> {
    Ok(Self {
      workspace: self.resolve_path(relative_path)?,
      fs: self.fs.clone(),
    })
  }

  /// Gets file or directory metadata
  pub async fn stat(&self, relative_path: &RelPath) -> Result<FileMetadata> {
    let path = self.resolve_path(relative_path)?;
    let stat = self.fs.stat(&path).await?;
    Ok(stat)
  }

  /// Removes the specified file
  ///
  /// Does not return an error if the file doesn't exist
  pub async fn remove_file(&self, relative_path: &RelPath) -> Result<()> {
    let path = self.resolve_path(relative_path)?;
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
  pub async fn write(&self, relative_path: &RelPath, bytes: &[u8]) -> Result<()> {
    let path = self.resolve_path(relative_path)?;
    if let Some(parent) = path.parent() {
      self.fs.create_dir_all(&parent).await?;
    }
    self.fs.write(&path, bytes).await?;
    Ok(())
  }

  /// Reads entire file content
  #[cfg(test)]
  pub async fn read(&self, relative_path: &RelPath) -> Result<Vec<u8>> {
    let path = self.resolve_path(relative_path)?;
    let data = self.fs.read_file(&path).await?;
    Ok(data)
  }

  /// Creates a file read stream (for large files)
  pub async fn stream_read(&self, relative_path: &RelPath) -> Result<Reader> {
    let path = self.resolve_path(relative_path)?;
    let reader = self.fs.create_read_stream(&path).await?;
    Ok(reader)
  }

  /// Creates a file write stream (for large files)
  ///
  /// If the file already exists, it will be deleted first
  pub async fn stream_write(&self, relative_path: &RelPath) -> Result<Writer> {
    let path = self.resolve_path(relative_path)?;
    let _ = self.fs.remove_file(&path).await;
    if let Some(parent) = path.parent() {
      self.fs.create_dir_all(&parent).await?;
    }
    let writer = self.fs.create_write_stream(&path).await?;
    Ok(writer)
  }

  /// Lists all direct children in the workspace directory
  pub async fn list_child(&self) -> Result<Vec<RelPath>> {
    let children = self.fs.read_dir(&self.workspace).await?;
    children
      .into_iter()
      .map(|child| RelPath::new_contained(&child).map_err(Into::into))
      .collect()
  }
}

#[cfg(test)]
mod tests {
  use rspack_fs::RelPath;

  use super::{Result, ScopeFileSystem};

  fn rel(path: &str) -> RelPath {
    RelPath::new_contained(path).expect("valid test relative path")
  }

  #[tokio::test]
  #[cfg_attr(miri, ignore)]
  async fn test_read_and_write() -> Result<()> {
    let fs = ScopeFileSystem::new_memory_fs("/");
    assert!(fs.read(&rel("a.txt")).await.is_err());

    fs.write(&rel("a.txt"), "hello world".as_bytes()).await?;
    assert_eq!(fs.read(&rel("a.txt")).await?, "hello world".as_bytes());
    Ok(())
  }

  #[tokio::test]
  #[cfg_attr(miri, ignore)]
  async fn test_stream_read_and_write() -> Result<()> {
    let fs = ScopeFileSystem::new_memory_fs("/");
    assert!(fs.read(&rel("a.txt")).await.is_err());

    let mut writer = fs.stream_write(&rel("a.txt")).await?;
    writer.write_line("hello").await?;
    writer.write("world".as_bytes()).await?;
    writer.flush().await?;

    let mut reader = fs.stream_read(&rel("a.txt")).await?;
    assert_eq!(reader.read_line().await?, "hello");
    assert_eq!(reader.read_to_end().await?, "world".as_bytes());
    Ok(())
  }

  #[tokio::test]
  #[cfg_attr(miri, ignore)]
  async fn test_move_to() -> Result<()> {
    let fs = ScopeFileSystem::new_memory_fs("/");
    fs.write(&rel("a.txt"), "1".as_bytes()).await?;

    let temp_fs = fs.child_fs(&rel(".temp"))?;
    temp_fs.write(&rel("a.txt"), "2".as_bytes()).await?;

    // Moving a non-existent file should succeed
    ScopeFileSystem::move_to(&temp_fs, &fs, &rel("b.txt")).await?;
    assert_eq!(fs.read(&rel("a.txt")).await?, "1".as_bytes());

    ScopeFileSystem::move_to(&temp_fs, &fs, &rel("a.txt")).await?;
    assert_eq!(fs.read(&rel("a.txt")).await?, "2".as_bytes());

    Ok(())
  }
}
