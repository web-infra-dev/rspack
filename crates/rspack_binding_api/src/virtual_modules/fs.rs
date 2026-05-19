use std::{
  fmt::Debug,
  sync::{Arc, RwLock},
};

use rspack_fs::{FileMetadata, FilePermissions, ReadableFileSystem, Result};
use rspack_paths::{Utf8Path, Utf8PathBuf};

use crate::virtual_modules::VirtualFileStore;

pub struct VirtualFileSystem {
  real_fs: Arc<dyn ReadableFileSystem>,
  virtual_file_store: Arc<RwLock<dyn VirtualFileStore>>,
}

impl VirtualFileSystem {
  pub fn new(
    real_fs: Arc<dyn ReadableFileSystem>,
    virtual_file_store: Arc<RwLock<dyn VirtualFileStore>>,
  ) -> Self {
    Self {
      real_fs,
      virtual_file_store,
    }
  }
}

impl Debug for VirtualFileSystem {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("VirtualFileSystem")
      .field("virtual_file_store", &"<VirtualFileStore>")
      .field("real_fs", &"<ReadableFileSystem>")
      .finish()
  }
}

#[async_trait::async_trait]
impl ReadableFileSystem for VirtualFileSystem {
  async fn read(&self, path: &Utf8Path) -> Result<Vec<u8>> {
    if let Ok(store) = self.virtual_file_store.read()
      && let Some(content) = store.get_file_content(path)
    {
      return Ok(content.clone());
    }

    self.real_fs.read(path).await
  }

  fn read_sync(&self, path: &Utf8Path) -> Result<Vec<u8>> {
    if let Ok(store) = self.virtual_file_store.read()
      && let Some(content) = store.get_file_content(path)
    {
      return Ok(content.clone());
    }

    self.real_fs.read_sync(path)
  }

  async fn metadata(&self, path: &Utf8Path) -> Result<FileMetadata> {
    if let Ok(store) = self.virtual_file_store.read()
      && let Some(metadata) = store.get_file_metadata(path)
    {
      return Ok(metadata);
    }

    self.real_fs.metadata(path).await
  }

  fn metadata_sync(&self, path: &Utf8Path) -> Result<FileMetadata> {
    if let Ok(store) = self.virtual_file_store.read()
      && let Some(metadata) = store.get_file_metadata(path)
    {
      return Ok(metadata);
    }

    self.real_fs.metadata_sync(path)
  }

  async fn symlink_metadata(&self, path: &Utf8Path) -> Result<FileMetadata> {
    if let Ok(store) = self.virtual_file_store.read()
      && let Some(metadata) = store.get_file_metadata(path)
    {
      return Ok(metadata);
    }

    self.real_fs.symlink_metadata(path).await
  }

  async fn canonicalize(&self, path: &Utf8Path) -> Result<Utf8PathBuf> {
    if let Ok(store) = self.virtual_file_store.read()
      && store.contains(path)
    {
      return Ok(path.to_path_buf());
    }

    self.real_fs.canonicalize(path).await
  }

  async fn read_dir(&self, dir: &Utf8Path) -> Result<Vec<String>> {
    if let Some(mut vlist) = self
      .virtual_file_store
      .read()
      .ok()
      .and_then(|store| store.read_dir(dir))
    {
      let mut list = self.real_fs.read_dir(dir).await.unwrap_or_default();
      list.append(&mut vlist);
      Ok(list)
    } else {
      self.real_fs.read_dir(dir).await
    }
  }

  fn read_dir_sync(&self, dir: &Utf8Path) -> Result<Vec<String>> {
    if let Some(mut vlist) = self
      .virtual_file_store
      .read()
      .ok()
      .and_then(|store| store.read_dir(dir))
    {
      let mut list = self.real_fs.read_dir_sync(dir).unwrap_or_default();
      list.append(&mut vlist);
      Ok(list)
    } else {
      self.real_fs.read_dir_sync(dir)
    }
  }

  async fn permissions(&self, path: &Utf8Path) -> Result<Option<FilePermissions>> {
    if let Ok(store) = self.virtual_file_store.read()
      && store.contains(path)
    {
      return Ok(Some(FilePermissions::from_mode(0o700)));
    }

    self.real_fs.permissions(path).await
  }
}
