use std::fmt::Debug;

use crate::{AbsPath, Result};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileMetadata {
  pub is_file: bool,
  pub is_directory: bool,
  pub is_symlink: bool,
  pub atime_ms: u64,
  pub mtime_ms: u64,
  pub ctime_ms: u64,
  pub size: u64,
}

/// Low-level read-only filesystem interface operating strictly on UTF-8 [`AbsPath`]s.
#[async_trait::async_trait]
pub trait ReadableFileSystem: Debug + Send + Sync {
  async fn read(&self, path: &AbsPath) -> Result<Vec<u8>>;
  fn read_sync(&self, path: &AbsPath) -> Result<Vec<u8>>;

  async fn metadata(&self, path: &AbsPath) -> Result<FileMetadata>;
  fn metadata_sync(&self, path: &AbsPath) -> Result<FileMetadata>;

  async fn symlink_metadata(&self, path: &AbsPath) -> Result<FileMetadata>;

  async fn read_link(&self, path: &AbsPath) -> Result<String>;

  async fn canonicalize(&self, path: &AbsPath) -> Result<AbsPath>;
  fn canonicalize_sync(&self, path: &AbsPath) -> Result<AbsPath>;

  async fn read_dir(&self, dir: &AbsPath) -> Result<Vec<String>>;
  fn read_dir_sync(&self, dir: &AbsPath) -> Result<Vec<String>>;
}
