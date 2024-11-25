use std::fs;

use futures::future::BoxFuture;
use rspack_paths::{AssertUtf8, Utf8Path, Utf8PathBuf};

use crate::{
  AsyncReadableFileSystem, AsyncWritableFileSystem, Error, FileMetadata, FileSystem, Result,
  SyncReadableFileSystem, SyncWritableFileSystem, WritableFileSystem,
};

#[derive(Debug)]
pub struct NativeFileSystem;
impl FileSystem for NativeFileSystem {}
impl WritableFileSystem for NativeFileSystem {}

impl SyncWritableFileSystem for NativeFileSystem {
  fn create_dir(&self, dir: &Utf8Path) -> Result<()> {
    fs::create_dir(dir).map_err(Error::from)
  }

  fn create_dir_all(&self, dir: &Utf8Path) -> Result<()> {
    fs::create_dir_all(dir).map_err(Error::from)
  }

  fn write(&self, file: &Utf8Path, data: &[u8]) -> Result<()> {
    fs::write(file, data).map_err(Error::from)
  }
}

impl SyncReadableFileSystem for NativeFileSystem {
  fn read(&self, path: &Utf8Path) -> Result<Vec<u8>> {
    fs::read(path).map_err(Error::from)
  }

  fn metadata(&self, path: &Utf8Path) -> Result<FileMetadata> {
    let meta = fs::metadata(path)?;
    meta.try_into()
  }

  fn symlink_metadata(&self, path: &Utf8Path) -> Result<FileMetadata> {
    let meta = fs::symlink_metadata(path)?;
    meta.try_into()
  }

  fn canonicalize(&self, path: &Utf8Path) -> Result<Utf8PathBuf> {
    let path = dunce::canonicalize(path)?;
    Ok(path.assert_utf8())
  }
}

impl AsyncWritableFileSystem for NativeFileSystem {
  fn create_dir<'a>(&'a self, dir: &'a Utf8Path) -> BoxFuture<'a, Result<()>> {
    let dir = dir.to_path_buf();
    let fut = async move { tokio::fs::create_dir(dir).await.map_err(Error::from) };
    Box::pin(fut)
  }

  fn create_dir_all<'a>(&'a self, dir: &'a Utf8Path) -> BoxFuture<'a, Result<()>> {
    let fut = async move { tokio::fs::create_dir_all(dir).await.map_err(Error::from) };
    Box::pin(fut)
  }

  fn write<'a>(&'a self, file: &'a Utf8Path, data: &'a [u8]) -> BoxFuture<'a, Result<()>> {
    let fut = async move { tokio::fs::write(file, data).await.map_err(Error::from) };
    Box::pin(fut)
  }

  fn remove_file<'a>(&'a self, file: &'a Utf8Path) -> BoxFuture<'a, Result<()>> {
    let fut = async move { tokio::fs::remove_file(file).await.map_err(Error::from) };
    Box::pin(fut)
  }

  fn remove_dir_all<'a>(&'a self, dir: &'a Utf8Path) -> BoxFuture<'a, Result<()>> {
    let dir = dir.to_path_buf();
    let fut = async move { tokio::fs::remove_dir_all(dir).await.map_err(Error::from) };
    Box::pin(fut)
  }

  fn read_dir<'a>(&'a self, dir: &'a Utf8Path) -> BoxFuture<'a, Result<Vec<String>>> {
    let dir = dir.to_path_buf();
    let fut = async move {
      let mut reader = tokio::fs::read_dir(dir).await.map_err(Error::from)?;
      let mut res = vec![];
      while let Some(entry) = reader.next_entry().await.map_err(Error::from)? {
        res.push(entry.file_name().to_string_lossy().to_string());
      }
      Ok(res)
    };
    Box::pin(fut)
  }

  fn read_file<'a>(&'a self, file: &'a Utf8Path) -> BoxFuture<'a, Result<Vec<u8>>> {
    let fut = async move { tokio::fs::read(file).await.map_err(Error::from) };
    Box::pin(fut)
  }

  fn stat<'a>(&'a self, file: &'a Utf8Path) -> BoxFuture<'a, Result<FileMetadata>> {
    let fut = async move {
      let metadata = tokio::fs::metadata(file).await.map_err(Error::from)?;
      FileMetadata::try_from(metadata)
    };
    Box::pin(fut)
  }
}

impl AsyncReadableFileSystem for NativeFileSystem {
  fn async_read<'a>(&'a self, file: &'a Utf8Path) -> BoxFuture<'a, Result<Vec<u8>>> {
    let fut = async move { tokio::fs::read(file).await.map_err(Error::from) };
    Box::pin(fut)
  }
}
