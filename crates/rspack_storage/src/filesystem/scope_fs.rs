use std::sync::Arc;

use rspack_fs::{
  FileMetadata, IntermediateFileSystem, ReadStream, Result as FsResult, WriteStream,
};
use rspack_paths::{Utf8Path, Utf8PathBuf};

use crate::{Error, Result};

pub type Reader = Box<dyn ReadStream>;
pub type Writer = Box<dyn WriteStream>;

#[derive(Debug)]
struct PathReader {
  inner: Reader,
  path: Utf8PathBuf,
}

#[async_trait::async_trait]
impl ReadStream for PathReader {
  async fn read(&mut self, length: usize) -> FsResult<Vec<u8>> {
    self
      .inner
      .read(length)
      .await
      .map_err(|error| error.with_path("read", &self.path))
  }

  async fn read_until(&mut self, byte: u8) -> FsResult<Vec<u8>> {
    self
      .inner
      .read_until(byte)
      .await
      .map_err(|error| error.with_path("read", &self.path))
  }

  async fn read_to_end(&mut self) -> FsResult<Vec<u8>> {
    self
      .inner
      .read_to_end()
      .await
      .map_err(|error| error.with_path("read", &self.path))
  }

  async fn skip(&mut self, offset: usize) -> FsResult<()> {
    self
      .inner
      .skip(offset)
      .await
      .map_err(|error| error.with_path("seek", &self.path))
  }

  async fn close(&mut self) -> FsResult<()> {
    self
      .inner
      .close()
      .await
      .map_err(|error| error.with_path("close", &self.path))
  }
}

#[derive(Debug)]
struct PathWriter {
  inner: Writer,
  path: Utf8PathBuf,
}

#[async_trait::async_trait]
impl WriteStream for PathWriter {
  async fn write(&mut self, buf: &[u8]) -> FsResult<usize> {
    self
      .inner
      .write(buf)
      .await
      .map_err(|error| error.with_path("write", &self.path))
  }

  async fn write_all(&mut self, buf: &[u8]) -> FsResult<()> {
    self
      .inner
      .write_all(buf)
      .await
      .map_err(|error| error.with_path("write", &self.path))
  }

  async fn flush(&mut self) -> FsResult<()> {
    self
      .inner
      .flush()
      .await
      .map_err(|error| error.with_path("flush", &self.path))
  }

  async fn close(&mut self) -> FsResult<()> {
    self
      .inner
      .close()
      .await
      .map_err(|error| error.with_path("close", &self.path))
  }
}

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
    self
      .fs
      .create_dir_all(&self.workspace)
      .await
      .map_err(|error| error.with_path("create directory", &self.workspace))?;
    Ok(())
  }

  /// Removes the entire workspace directory and its contents
  pub async fn remove(&self) -> Result<()> {
    if let Err(e) = self
      .fs
      .remove_dir_all(&self.workspace)
      .await
      .map_err(|error| error.with_path("remove directory", &self.workspace))
    {
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
    relative_path: impl AsRef<Utf8Path>,
  ) -> Result<()> {
    let from_file = from.workspace.join(relative_path.as_ref());
    let to_file = to.workspace.join(relative_path.as_ref());
    if let Err(e) = from
      .fs
      .rename(&from_file, &to_file)
      .await
      .map_err(|error| error.with_paths("rename", [&from_file, &to_file]))
    {
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
    let stat = self
      .fs
      .stat(&path)
      .await
      .map_err(|error| error.with_path("stat", &path))?;
    Ok(stat)
  }

  /// Removes the specified file
  ///
  /// Does not return an error if the file doesn't exist
  pub async fn remove_file(&self, relative_path: impl AsRef<Utf8Path>) -> Result<()> {
    let path = self.workspace.join(relative_path);
    if let Err(e) = self
      .fs
      .remove_file(&path)
      .await
      .map_err(|error| error.with_path("remove file", &path))
    {
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
    let parent = path.parent().expect("should have parent");
    self
      .fs
      .create_dir_all(parent)
      .await
      .map_err(|error| error.with_path("create directory", parent))?;
    self
      .fs
      .write(&path, bytes)
      .await
      .map_err(|error| error.with_path("write", &path))?;
    Ok(())
  }

  /// Reads entire file content
  #[cfg(test)]
  pub async fn read(&self, relative_path: impl AsRef<Utf8Path>) -> Result<Vec<u8>> {
    let path = self.workspace.join(relative_path);
    let data = self
      .fs
      .read_file(&path)
      .await
      .map_err(|error| error.with_path("read", &path))?;
    Ok(data)
  }

  /// Creates a file read stream (for large files)
  pub async fn stream_read(&self, relative_path: impl AsRef<Utf8Path>) -> Result<Reader> {
    let path = self.workspace.join(relative_path);
    let reader = self
      .fs
      .create_read_stream(&path)
      .await
      .map_err(|error| error.with_path("open for reading", &path))?;
    Ok(Box::new(PathReader {
      inner: reader,
      path,
    }))
  }

  /// Creates a file write stream (for large files)
  ///
  /// If the file already exists, it will be deleted first
  pub async fn stream_write(&self, relative_path: impl AsRef<Utf8Path>) -> Result<Writer> {
    let _ = self.remove_file(&relative_path).await;

    let path = self.workspace.join(relative_path);
    let parent = path.parent().expect("should have parent");
    self
      .fs
      .create_dir_all(parent)
      .await
      .map_err(|error| error.with_path("create directory", parent))?;
    let writer = self
      .fs
      .create_write_stream(&path)
      .await
      .map_err(|error| error.with_path("open for writing", &path))?;
    Ok(Box::new(PathWriter {
      inner: writer,
      path,
    }))
  }

  /// Lists all direct children in the workspace directory
  pub async fn list_child(&self) -> Result<Vec<String>> {
    let children = self
      .fs
      .read_dir(&self.workspace)
      .await
      .map_err(|error| error.with_path("read directory", &self.workspace))?;
    Ok(children)
  }
}

#[cfg(test)]
mod tests {
  use rspack_fs::{Error as FsError, WriteStream};

  use super::{PathWriter, Result, ScopeFileSystem, Writer};

  #[derive(Debug)]
  struct FailingWriter;

  #[async_trait::async_trait]
  impl WriteStream for FailingWriter {
    async fn write(&mut self, buf: &[u8]) -> rspack_fs::Result<usize> {
      Ok(buf.len())
    }

    async fn write_all(&mut self, _buf: &[u8]) -> rspack_fs::Result<()> {
      Ok(())
    }

    async fn flush(&mut self) -> rspack_fs::Result<()> {
      Err(FsError::new(std::io::ErrorKind::Other, "flush failed"))
    }

    async fn close(&mut self) -> rspack_fs::Result<()> {
      Ok(())
    }
  }

  #[tokio::test]
  #[cfg_attr(miri, ignore)]
  async fn test_read_and_write() -> Result<()> {
    let fs = ScopeFileSystem::new_memory_fs("/".into());
    assert!(fs.read("/a.txt").await.is_err());

    fs.write("a.txt", "hello world".as_bytes()).await?;
    assert_eq!(fs.read("a.txt").await?, "hello world".as_bytes());
    Ok(())
  }

  #[tokio::test]
  #[cfg_attr(miri, ignore)]
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
  #[cfg_attr(miri, ignore)]
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

  #[tokio::test]
  #[cfg_attr(miri, ignore)]
  async fn test_error_includes_operation_path() {
    let fs = ScopeFileSystem::new_memory_fs("/cache".into());

    let error = fs
      .stat("missing")
      .await
      .expect_err("stat should fail for a missing path");

    assert!(error.to_string().contains("stat '/cache/missing' failed"));
  }

  #[tokio::test]
  #[cfg_attr(miri, ignore)]
  async fn test_stream_error_includes_operation_path() {
    let inner: Writer = Box::new(FailingWriter);
    let mut writer = PathWriter {
      inner,
      path: "/cache/make/_meta".into(),
    };

    let error = writer
      .flush()
      .await
      .expect_err("flush should return the inner error");

    assert!(
      error
        .to_string()
        .contains("flush '/cache/make/_meta' failed")
    );
  }
}
