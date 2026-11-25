use std::sync::Arc;

use rspack_fs::{
  AbsPath, Error, FileMetadata, FilePermissions, MemoryFileSystem, ReadableFileSystem, Result,
};

#[derive(Debug)]
pub struct VirtualFileSystem {
  real_fs: Arc<dyn ReadableFileSystem>,
  memory_fs: MemoryFileSystem,
}

impl VirtualFileSystem {
  pub fn new(real_fs: Arc<dyn ReadableFileSystem>, memory_fs: MemoryFileSystem) -> Self {
    Self { real_fs, memory_fs }
  }
}

#[async_trait::async_trait]
impl ReadableFileSystem for VirtualFileSystem {
  async fn read(&self, path: &AbsPath) -> Result<Vec<u8>> {
    if let Ok(content) = self.memory_fs.read_sync(path) {
      return Ok(content);
    }

    self.real_fs.read(path).await
  }

  fn read_sync(&self, path: &AbsPath) -> Result<Vec<u8>> {
    if let Ok(content) = self.memory_fs.read_sync(path) {
      return Ok(content);
    }

    self.real_fs.read_sync(path)
  }

  fn is_virtual(&self, path: &AbsPath) -> bool {
    self.memory_fs.exists(path) || self.real_fs.is_virtual(path)
  }

  async fn metadata(&self, path: &AbsPath) -> Result<FileMetadata> {
    if let Ok(metadata) = self.memory_fs.metadata_sync(path) {
      return Ok(metadata);
    }

    self.real_fs.metadata(path).await
  }

  fn metadata_sync(&self, path: &AbsPath) -> Result<FileMetadata> {
    if let Ok(metadata) = self.memory_fs.metadata_sync(path) {
      return Ok(metadata);
    }

    self.real_fs.metadata_sync(path)
  }

  async fn symlink_metadata(&self, path: &AbsPath) -> Result<FileMetadata> {
    if let Ok(metadata) = self.memory_fs.metadata_sync(path) {
      return Ok(metadata);
    }

    self.real_fs.symlink_metadata(path).await
  }

  async fn read_link(&self, path: &AbsPath) -> Result<String> {
    if self.memory_fs.exists(path) {
      return Err(Error::new(
        std::io::ErrorKind::InvalidInput,
        "virtual file is not a symbolic link",
      ));
    }

    self.real_fs.read_link(path).await
  }

  async fn canonicalize(&self, path: &AbsPath) -> Result<AbsPath> {
    if self.memory_fs.exists(path) {
      return Ok(path.clone());
    }

    self.real_fs.canonicalize(path).await
  }

  fn canonicalize_sync(&self, path: &AbsPath) -> Result<AbsPath> {
    if self.memory_fs.exists(path) {
      return Ok(path.clone());
    }

    self.real_fs.canonicalize_sync(path)
  }

  async fn read_dir(&self, dir: &AbsPath) -> Result<Vec<String>> {
    if let Ok(mut vlist) = self.memory_fs.read_dir_sync(dir) {
      let mut list = self.real_fs.read_dir(dir).await.unwrap_or_default();
      list.append(&mut vlist);
      Ok(list)
    } else {
      self.real_fs.read_dir(dir).await
    }
  }

  fn read_dir_sync(&self, dir: &AbsPath) -> Result<Vec<String>> {
    if let Ok(mut vlist) = self.memory_fs.read_dir_sync(dir) {
      let mut list = self.real_fs.read_dir_sync(dir).unwrap_or_default();
      list.append(&mut vlist);
      Ok(list)
    } else {
      self.real_fs.read_dir_sync(dir)
    }
  }

  async fn permissions(&self, path: &AbsPath) -> Result<Option<FilePermissions>> {
    if self.memory_fs.exists(path) {
      return Ok(Some(FilePermissions::from_mode(0o700)));
    }

    self.real_fs.permissions(path).await
  }
}
