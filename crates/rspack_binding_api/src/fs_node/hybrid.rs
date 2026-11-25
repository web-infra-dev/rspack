use async_trait::async_trait;
use rspack_fs::{
  AbsPath, FileMetadata, FilePermissions, NativeFileSystem, ReadableFileSystem, Result,
};
use rspack_regex::RspackRegex;

use super::NodeFileSystem;

#[derive(Debug)]
pub struct HybridFileSystem {
  allowlist: Vec<RspackRegex>,
  node_fs: NodeFileSystem,
  native_fs: NativeFileSystem,
}

impl HybridFileSystem {
  pub fn new(
    allowlist: Vec<RspackRegex>,
    node_fs: NodeFileSystem,
    native_fs: NativeFileSystem,
  ) -> Self {
    Self {
      allowlist,
      node_fs,
      native_fs,
    }
  }

  fn pick_fs_for_path(&self, path: &AbsPath) -> &dyn ReadableFileSystem {
    if self
      .allowlist
      .iter()
      .any(|regexp| regexp.test(path.as_str()))
    {
      &self.node_fs
    } else {
      &self.native_fs
    }
  }
}

#[async_trait]
impl ReadableFileSystem for HybridFileSystem {
  async fn read(&self, path: &AbsPath) -> Result<Vec<u8>> {
    self.pick_fs_for_path(path).read(path).await
  }
  fn read_sync(&self, path: &AbsPath) -> Result<Vec<u8>> {
    self.pick_fs_for_path(path).read_sync(path)
  }

  fn is_virtual(&self, path: &AbsPath) -> bool {
    self.pick_fs_for_path(path).is_virtual(path)
  }

  async fn metadata(&self, path: &AbsPath) -> Result<FileMetadata> {
    self.pick_fs_for_path(path).metadata(path).await
  }
  fn metadata_sync(&self, path: &AbsPath) -> Result<FileMetadata> {
    self.pick_fs_for_path(path).metadata_sync(path)
  }

  async fn symlink_metadata(&self, path: &AbsPath) -> Result<FileMetadata> {
    self.pick_fs_for_path(path).symlink_metadata(path).await
  }

  async fn read_link(&self, path: &AbsPath) -> Result<String> {
    self.pick_fs_for_path(path).read_link(path).await
  }

  async fn canonicalize(&self, path: &AbsPath) -> Result<AbsPath> {
    self.pick_fs_for_path(path).canonicalize(path).await
  }

  fn canonicalize_sync(&self, path: &AbsPath) -> Result<AbsPath> {
    self.pick_fs_for_path(path).canonicalize_sync(path)
  }

  async fn read_dir(&self, path: &AbsPath) -> Result<Vec<String>> {
    self.pick_fs_for_path(path).read_dir(path).await
  }

  fn read_dir_sync(&self, path: &AbsPath) -> Result<Vec<String>> {
    self.pick_fs_for_path(path).read_dir_sync(path)
  }

  async fn permissions(&self, path: &AbsPath) -> Result<Option<FilePermissions>> {
    self.pick_fs_for_path(path).permissions(path).await
  }
}
