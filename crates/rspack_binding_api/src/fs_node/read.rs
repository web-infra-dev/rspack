use async_trait::async_trait;
use napi::{Either, bindgen_prelude::Either3};
use rspack_fs::{
  AbsPath, Error, FileMetadata, FilePermissions, ReadableFileSystem, Result,
  RspackResultToFsResultExt,
};
use tracing::instrument;

use super::NodeFileSystem;

#[async_trait]
impl ReadableFileSystem for NodeFileSystem {
  #[instrument(skip(self), level = "debug")]
  async fn read(&self, path: &AbsPath) -> Result<Vec<u8>> {
    self
      .0
      .read_file
      .call_with_promise(path.as_str().to_string())
      .await
      .to_fs_result()
      // TODO: simplify the return value?
      .map(|result| match result {
        Either3::A(buf) => buf.into(),
        Either3::B(str) => str.into(),
        Either3::C(_) => vec![],
      })
  }

  #[instrument(skip(self), level = "debug")]
  fn read_sync(&self, path: &AbsPath) -> Result<Vec<u8>> {
    #[allow(clippy::disallowed_methods)]
    rspack_napi::runtime::block_on(self.read(path))
  }

  #[instrument(skip(self), level = "debug")]
  async fn metadata(&self, path: &AbsPath) -> Result<FileMetadata> {
    let res = self
      .0
      .stat
      .call_with_promise(path.as_str().to_string())
      .await
      .to_fs_result()?;
    match res {
      Either::A(stats) => Ok(stats.into()),
      Either::B(_) => Err(Error::new(
        std::io::ErrorKind::Other,
        "input file system call stat failed",
      )),
    }
  }

  #[instrument(skip(self), level = "debug")]
  fn metadata_sync(&self, path: &AbsPath) -> Result<FileMetadata> {
    #[allow(clippy::disallowed_methods)]
    rspack_napi::runtime::block_on(self.metadata(path))
  }

  #[instrument(skip(self), level = "debug")]
  async fn symlink_metadata(&self, path: &AbsPath) -> Result<FileMetadata> {
    let res = self
      .0
      .lstat
      .call_with_promise(path.as_str().to_string())
      .await
      .to_fs_result()?;
    match res {
      Either::A(stats) => Ok(stats.into()),
      Either::B(_) => Err(Error::new(
        std::io::ErrorKind::Other,
        "input file system call lstat failed",
      )),
    }
  }

  #[instrument(skip(self), level = "debug")]
  async fn read_link(&self, path: &AbsPath) -> Result<String> {
    let res = self
      .0
      .readlink
      .call_with_promise(path.as_str().to_string())
      .await
      .to_fs_result()?;
    match res {
      Either::A(target) => Ok(target),
      Either::B(_) => Err(Error::new(
        std::io::ErrorKind::Other,
        "input file system call readlink failed",
      )),
    }
  }

  #[instrument(skip(self), level = "debug")]
  async fn canonicalize(&self, path: &AbsPath) -> Result<AbsPath> {
    let res = self
      .0
      .realpath
      .call_with_promise(path.as_str().to_string())
      .await
      .to_fs_result()?;
    match res {
      Either::A(str) => AbsPath::new(&str),
      Either::B(_) => Err(Error::new(
        std::io::ErrorKind::Other,
        "input file system call realpath failed",
      )),
    }
  }

  #[instrument(skip(self), level = "debug")]
  fn canonicalize_sync(&self, path: &AbsPath) -> Result<AbsPath> {
    #[allow(clippy::disallowed_methods)]
    rspack_napi::runtime::block_on(self.canonicalize(path))
  }

  #[instrument(skip(self), level = "debug")]
  async fn read_dir(&self, dir: &AbsPath) -> Result<Vec<String>> {
    let res = self
      .0
      .read_dir
      .call_with_promise(dir.as_str().to_string())
      .await
      .to_fs_result()?;
    match res {
      Either::A(list) => Ok(list),
      Either::B(_) => Err(Error::new(
        std::io::ErrorKind::Other,
        "input file system call read_dir failed",
      )),
    }
  }

  #[instrument(skip(self), level = "debug")]
  fn read_dir_sync(&self, dir: &AbsPath) -> Result<Vec<String>> {
    #[allow(clippy::disallowed_methods)]
    rspack_napi::runtime::block_on(ReadableFileSystem::read_dir(self, dir))
  }

  #[instrument(skip(self), level = "debug")]
  async fn permissions(&self, path: &AbsPath) -> Result<Option<FilePermissions>> {
    let res = self
      .0
      .stat
      .call_with_promise(path.as_str().to_string())
      .await
      .to_fs_result()?;
    match res {
      Either::A(stats) => Ok(Some(FilePermissions::from_mode(stats.mode))),
      Either::B(_) => Err(Error::new(
        std::io::ErrorKind::Other,
        "input file system call stat failed",
      )),
    }
  }
}
