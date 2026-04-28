use rspack_paths::Utf8Path;

use crate::{Error, FileMetadata, FilePermissions, Result, WritableFileSystem};

#[derive(Debug, Default)]
pub struct NoopFileSystem;

#[async_trait::async_trait]
impl WritableFileSystem for NoopFileSystem {
  async fn create_dir(&self, _dir: &Utf8Path) -> Result<()> {
    Ok(())
  }

  async fn create_dir_all(&self, _dir: &Utf8Path) -> Result<()> {
    Ok(())
  }

  async fn write(&self, _file: &Utf8Path, _data: &[u8]) -> Result<()> {
    Ok(())
  }

  async fn remove_file(&self, _file: &Utf8Path) -> Result<()> {
    Ok(())
  }

  async fn remove_dir_all(&self, _dir: &Utf8Path) -> Result<()> {
    Ok(())
  }

  async fn read_dir(&self, _dir: &Utf8Path) -> Result<Vec<String>> {
    Ok(Vec::new())
  }

  async fn read_file(&self, _file: &Utf8Path) -> Result<Vec<u8>> {
    Err(not_found())
  }

  async fn stat(&self, _file: &Utf8Path) -> Result<FileMetadata> {
    Err(not_found())
  }

  async fn set_permissions(&self, _path: &Utf8Path, _perm: FilePermissions) -> Result<()> {
    Ok(())
  }
}

fn not_found() -> Error {
  Error::Io(std::io::Error::new(
    std::io::ErrorKind::NotFound,
    "file not exist",
  ))
}
