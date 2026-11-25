use std::{io, path::PathBuf, sync::Arc};

use rspack_fs::{AbsPath, FsResultToIoResultExt, ReadableFileSystem};
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
    let abs = AbsPath::try_from_std_path(path).to_io_result()?;
    self.0.read(&abs).await.to_io_result()
  }

  async fn read_to_string(&self, path: &std::path::Path) -> std::io::Result<String> {
    let abs = AbsPath::try_from_std_path(path).to_io_result()?;
    self.0.read_to_string(&abs).await.to_io_result()
  }

  async fn metadata(&self, path: &std::path::Path) -> io::Result<FileMetadata> {
    let abs = AbsPath::try_from_std_path(path).to_io_result()?;
    let meta = self.0.metadata(&abs).await.to_io_result()?;
    Ok(FileMetadata {
      is_dir: meta.is_directory,
      is_file: meta.is_file,
      is_symlink: meta.is_symlink,
    })
  }

  async fn symlink_metadata(&self, path: &std::path::Path) -> io::Result<FileMetadata> {
    let abs = AbsPath::try_from_std_path(path).to_io_result()?;
    let meta = self.0.symlink_metadata(&abs).await.to_io_result()?;
    Ok(FileMetadata {
      is_dir: meta.is_directory,
      is_file: meta.is_file,
      is_symlink: meta.is_symlink,
    })
  }

  async fn canonicalize(&self, path: &std::path::Path) -> io::Result<PathBuf> {
    let abs = AbsPath::try_from_std_path(path).to_io_result()?;
    let canonical = self.0.canonicalize(&abs).await.to_io_result()?;
    Ok(PathBuf::from(canonical.as_str()))
  }
}
