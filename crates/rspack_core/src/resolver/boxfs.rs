use std::sync::Arc;

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
    self.0.read_to_string(path)
  }

  fn metadata(&self, path: &std::path::Path) -> std::io::Result<FileMetadata> {
    self.0.metadata(path)
  }

  fn symlink_metadata(&self, path: &std::path::Path) -> std::io::Result<FileMetadata> {
    self.0.symlink_metadata(path)
  }

  fn canonicalize(&self, path: &std::path::Path) -> std::io::Result<std::path::PathBuf> {
    self.0.canonicalize(path)
  }
}
