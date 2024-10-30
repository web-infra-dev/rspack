use std::{
  fs::{self, Metadata},
  io,
  path::{Path, PathBuf},
};

use rspack_paths::Utf8Path;

use super::{
  sync::{ReadableFileSystem, WritableFileSystem},
  Error, Result,
};

#[derive(Debug)]
pub struct NativeFileSystem;

impl WritableFileSystem for NativeFileSystem {
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

impl ReadableFileSystem for NativeFileSystem {
  fn read(&self, path: &Path) -> io::Result<Vec<u8>> {
    fs::read(path)
  }

  fn metadata(&self, path: &Path) -> io::Result<Metadata> {
    fs::metadata(path)
  }

  fn symlink_metadata(&self, path: &Path) -> io::Result<Metadata> {
    fs::symlink_metadata(path)
  }

  fn canonicalize(&self, path: &Path) -> io::Result<PathBuf> {
    dunce::canonicalize(path)
  }
}

use futures::future::BoxFuture;

use crate::{r#async::FileStat, AsyncReadableFileSystem, AsyncWritableFileSystem};

#[derive(Debug)]
pub struct AsyncNativeFileSystem;

impl AsyncWritableFileSystem for AsyncNativeFileSystem {
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
      loop {
        if let Some(entry) = reader.next_entry().await.map_err(Error::from)? {
          res.push(entry.file_name().to_string_lossy().to_string());
        } else {
          break;
        }
      }
      Ok(res)
    };
    Box::pin(fut)
  }

  fn read_file<'a>(&'a self, file: &'a Utf8Path) -> BoxFuture<'a, Result<Vec<u8>>> {
    let fut = async move { tokio::fs::read(file).await.map_err(Error::from) };
    Box::pin(fut)
  }

  fn stat<'a>(&'a self, file: &'a Utf8Path) -> BoxFuture<'a, Result<crate::r#async::FileStat>> {
    let fut = async move {
      let metadata = tokio::fs::metadata(file).await.map_err(Error::from)?;
      FileStat::try_from(metadata)
    };
    Box::pin(fut)
  }
}

impl AsyncReadableFileSystem for AsyncNativeFileSystem {
  fn read<'a>(&'a self, file: &'a Utf8Path) -> BoxFuture<'a, Result<Vec<u8>>> {
    let fut = async move { tokio::fs::read(file).await.map_err(Error::from) };
    Box::pin(fut)
  }
}

impl TryFrom<Metadata> for FileStat {
  fn try_from(metadata: Metadata) -> Result<Self> {
    let mtime_ms = metadata
      .modified()
      .map_err(Error::from)?
      .duration_since(std::time::UNIX_EPOCH)
      .expect("mtime is before unix epoch")
      .as_millis() as u64;
    let ctime_ms = metadata
      .created()
      .map_err(Error::from)?
      .duration_since(std::time::UNIX_EPOCH)
      .expect("ctime is before unix epoch")
      .as_millis() as u64;
    let atime_ms = metadata
      .accessed()
      .map_err(Error::from)?
      .duration_since(std::time::UNIX_EPOCH)
      .expect("atime is before unix epoch")
      .as_millis() as u64;
    Ok(Self {
      is_directory: metadata.is_dir(),
      is_file: metadata.is_file(),
      size: metadata.len(),
      mtime_ms,
      ctime_ms,
      atime_ms,
    })
  }

  type Error = Error;
}
