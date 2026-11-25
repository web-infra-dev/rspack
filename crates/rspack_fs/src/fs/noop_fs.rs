use std::io::ErrorKind;

use crate::{AbsPath, Error, FileMetadata, FilePermissions, Result, WritableFileSystem};

/// No-op writable filesystem used for benchmarks and dry runs.
#[derive(Debug, Default)]
pub struct NoopFileSystem;

#[async_trait::async_trait]
impl WritableFileSystem for NoopFileSystem {
  async fn create_dir(&self, _dir: &AbsPath) -> Result<()> {
    Ok(())
  }

  async fn create_dir_all(&self, _dir: &AbsPath) -> Result<()> {
    Ok(())
  }

  async fn write(&self, _file: &AbsPath, _data: &[u8]) -> Result<()> {
    Ok(())
  }

  async fn remove_file(&self, _file: &AbsPath) -> Result<()> {
    Ok(())
  }

  async fn remove_dir_all(&self, _dir: &AbsPath) -> Result<()> {
    Ok(())
  }

  async fn read_dir(&self, _dir: &AbsPath) -> Result<Vec<String>> {
    Ok(Vec::new())
  }

  async fn read_file(&self, _file: &AbsPath) -> Result<Vec<u8>> {
    Err(not_found())
  }

  async fn stat(&self, _file: &AbsPath) -> Result<FileMetadata> {
    Err(not_found())
  }

  async fn set_permissions(&self, _path: &AbsPath, _perm: FilePermissions) -> Result<()> {
    Ok(())
  }
}

fn not_found() -> Error {
  Error::new(ErrorKind::NotFound, "file not exist")
}
