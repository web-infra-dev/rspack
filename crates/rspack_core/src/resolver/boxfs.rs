use std::{
  io::{self},
  sync::Arc,
};

use rspack_fs::ReadableFileSystem;
use rspack_paths::AssertUtf8;
use rspack_resolver::{FileMetadata, FileSystem as ResolverFileSystem};

#[derive(Clone)]
pub struct BoxFS(Arc<dyn ReadableFileSystem>);

impl BoxFS {
  pub fn new(fs: Arc<dyn ReadableFileSystem>) -> Self {
    Self(fs)
  }
}
#[async_trait::async_trait]
impl ResolverFileSystem for BoxFS {
  async fn read(&self, path: &std::path::Path) -> io::Result<Vec<u8>> {
    self.0.read(path.assert_utf8()).await
  }
  async fn read_to_string(&self, path: &std::path::Path) -> std::io::Result<String> {
    let x = self.0.read(path.assert_utf8()).await?;
    String::from_utf8(x).map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))
  }
  async fn metadata(&self, path: &std::path::Path) -> io::Result<FileMetadata> {
    let meta = self.0.metadata(path.assert_utf8()).await?;
    Ok(FileMetadata {
      is_dir: meta.is_directory,
      is_file: meta.is_file,
      is_symlink: meta.is_symlink,
    })
  }

  async fn symlink_metadata(&self, path: &std::path::Path) -> io::Result<FileMetadata> {
    let meta = self.0.symlink_metadata(path.assert_utf8()).await?;
    Ok(FileMetadata {
      is_dir: meta.is_directory,
      is_file: meta.is_file,
      is_symlink: meta.is_symlink,
    })
  }

  async fn canonicalize(&self, path: &std::path::Path) -> io::Result<std::path::PathBuf> {
    let path = self.0.canonicalize(path.assert_utf8()).await?;
    Ok(path.into())
  }
}
