use std::{io, sync::Arc};

use rspack_fs::{Error, FileSystem};
use rspack_paths::AssertUtf8;
use rspack_resolver::{FileMetadata, FileSystem as ResolverFileSystem};

#[derive(Clone)]
pub struct BoxFS(Arc<dyn FileSystem>);

impl BoxFS {
  pub fn new(fs: Arc<dyn FileSystem>) -> Self {
    Self(fs)
  }
}
impl ResolverFileSystem for BoxFS {
  fn read_to_string(&self, path: &std::path::Path) -> std::io::Result<String> {
    match self.0.read(path.assert_utf8()) {
      Ok(x) => String::from_utf8(x).map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err)),
      Err(Error::Io(e)) => Err(e),
    }
  }

  fn metadata(&self, path: &std::path::Path) -> std::io::Result<FileMetadata> {
    match self.0.metadata(path.assert_utf8()) {
      Ok(meta) => Ok(FileMetadata {
        is_dir: meta.is_directory,
        is_file: meta.is_file,
        is_symlink: meta.is_symlink,
      }),
      Err(Error::Io(e)) => Err(e),
    }
  }

  fn symlink_metadata(&self, path: &std::path::Path) -> std::io::Result<FileMetadata> {
    match self.0.symlink_metadata(path.assert_utf8()) {
      Ok(meta) => Ok(FileMetadata {
        is_dir: meta.is_directory,
        is_file: meta.is_file,
        is_symlink: meta.is_symlink,
      }),
      Err(Error::Io(e)) => Err(e),
    }
  }

  fn canonicalize(&self, path: &std::path::Path) -> std::io::Result<std::path::PathBuf> {
    match self.0.canonicalize(path.assert_utf8()) {
      Ok(path) => Ok(path.into()),
      Err(Error::Io(e)) => Err(e),
    }
  }
}
