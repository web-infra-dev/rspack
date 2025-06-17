use async_trait::async_trait;
use rspack_fs::{FileMetadata, NativeFileSystem, ReadableFileSystem, Result};
use rspack_paths::{Utf8Path, Utf8PathBuf};
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

  fn pick_fs_for_path(&self, path: &Utf8Path) -> &dyn ReadableFileSystem {
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
  async fn read(&self, path: &Utf8Path) -> Result<Vec<u8>> {
    self.pick_fs_for_path(path).read(path).await
  }
  fn read_sync(&self, path: &Utf8Path) -> Result<Vec<u8>> {
    self.pick_fs_for_path(path).read_sync(path)
  }

  async fn metadata(&self, path: &Utf8Path) -> Result<FileMetadata> {
    self.pick_fs_for_path(path).metadata(path).await
  }
  fn metadata_sync(&self, path: &Utf8Path) -> Result<FileMetadata> {
    self.pick_fs_for_path(path).metadata_sync(path)
  }

  async fn symlink_metadata(&self, path: &Utf8Path) -> Result<FileMetadata> {
    self.pick_fs_for_path(path).symlink_metadata(path).await
  }

  async fn canonicalize(&self, path: &Utf8Path) -> Result<Utf8PathBuf> {
    self.pick_fs_for_path(path).canonicalize(path).await
  }

  async fn read_dir(&self, path: &Utf8Path) -> Result<Vec<String>> {
    self.pick_fs_for_path(path).read_dir(path).await
  }
  fn read_dir_sync(&self, path: &Utf8Path) -> Result<Vec<String>> {
    self.pick_fs_for_path(path).read_dir_sync(path)
  }
}
