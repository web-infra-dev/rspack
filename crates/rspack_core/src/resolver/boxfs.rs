use std::{io, sync::Arc};

use rspack_fs::ReadableFileSystem;
use rspack_resolver::{FileMetadata, FileSystem as ResolverFileSystem};

#[derive(Clone)]
pub struct BoxFS(Arc<dyn ReadableFileSystem>);

impl BoxFS {
  pub fn new(fs: Arc<dyn ReadableFileSystem>) -> Self {
    Self(fs)
  }
}
impl ResolverFileSystem for BoxFS {
  fn read_to_string(&self, path: &std::path::Path) -> std::io::Result<String> {
    self.0.read(path).and_then(|x| {
      String::from_utf8(x).map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))
    })
  }

  fn metadata(&self, path: &std::path::Path) -> std::io::Result<FileMetadata> {
    self.0.metadata(path).map(FileMetadata::from)
  }

  fn symlink_metadata(&self, path: &std::path::Path) -> std::io::Result<FileMetadata> {
    self.0.symlink_metadata(path).map(FileMetadata::from)
  }

  fn canonicalize(&self, path: &std::path::Path) -> std::io::Result<std::path::PathBuf> {
    self.0.canonicalize(path)
  }
}
