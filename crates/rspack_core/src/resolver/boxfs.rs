use std::{
  io::{self},
  sync::Arc,
};

use rspack_fs::{Error, FsResultToIoResultExt, ReadableFileSystem};
use rspack_paths::AssertUtf8;
use rspack_resolver::{FileMetadata, FileSystem as ResolverFileSystem};

#[derive(Clone)]
pub(super) struct BoxFS(Arc<dyn ReadableFileSystem>);

impl BoxFS {
  pub(super) fn new(fs: Arc<dyn ReadableFileSystem>) -> Self {
    Self(fs)
  }
}
#[async_trait::async_trait]
impl ResolverFileSystem for BoxFS {
  async fn read(&self, path: &std::path::Path) -> io::Result<Vec<u8>> {
    self.0.read(path.assert_utf8()).await.to_io_result()
  }
  async fn read_to_string(&self, path: &std::path::Path) -> std::io::Result<String> {
    match self.0.read(path.assert_utf8()).await {
      Ok(x) => String::from_utf8(x).map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err)),
      Err(Error::Io(e)) => Err(e),
    }
  }
  async fn metadata(&self, path: &std::path::Path) -> io::Result<FileMetadata> {
    match self.0.metadata(path.assert_utf8()).await {
      Ok(meta) => Ok(FileMetadata {
        is_dir: meta.is_directory,
        is_file: meta.is_file,
        is_symlink: meta.is_symlink,
      }),
      Err(Error::Io(e)) => Err(e),
    }
  }

  async fn symlink_metadata(&self, path: &std::path::Path) -> io::Result<FileMetadata> {
    match self.0.symlink_metadata(path.assert_utf8()).await {
      Ok(meta) => Ok(FileMetadata {
        is_dir: meta.is_directory,
        is_file: meta.is_file,
        is_symlink: meta.is_symlink,
      }),
      Err(Error::Io(e)) => Err(e),
    }
  }

  async fn canonicalize(&self, path: &std::path::Path) -> io::Result<std::path::PathBuf> {
    match self.0.canonicalize(path.assert_utf8()).await {
      Ok(path) => Ok(path.into()),
      Err(Error::Io(e)) => Err(e),
    }
  }
}
