use std::{fs, path::Path};

use super::{
  cfg_async,
  sync::{ReadableFileSystem, WritableFileSystem},
  Error, Result,
};

pub struct NativeFileSystem;

impl WritableFileSystem for NativeFileSystem {
  fn create_dir(&self, dir: &Path) -> Result<()> {
    fs::create_dir(dir).map_err(Error::from)
  }

  fn create_dir_all(&self, dir: &Path) -> Result<()> {
    fs::create_dir_all(dir).map_err(Error::from)
  }

  fn write(&self, file: &Path, data: &[u8]) -> Result<()> {
    fs::write(file, data).map_err(Error::from)
  }
}

impl ReadableFileSystem for NativeFileSystem {
  fn read(&self, file: &Path) -> Result<Vec<u8>> {
    fs::read(file).map_err(Error::from)
  }
}

cfg_async! {
  use futures::future::BoxFuture;

  use crate::{AsyncReadableFileSystem, AsyncWritableFileSystem};
  pub struct AsyncNativeFileSystem;

  impl AsyncWritableFileSystem for AsyncNativeFileSystem {
    fn create_dir(&self, dir: &Path) -> BoxFuture<'_, Result<()>> {
      let dir = dir.to_string_lossy().to_string();
      let fut = async move { tokio::fs::create_dir(dir).await.map_err(Error::from) };
      Box::pin(fut)
    }

    fn create_dir_all(&self, dir: &Path) -> BoxFuture<'_, Result<()>> {
      let dir = dir.to_string_lossy().to_string();
      let fut = async move { tokio::fs::create_dir_all(dir).await.map_err(Error::from) };
      Box::pin(fut)
    }

    fn write(
      &self,
      file: &Path,
      data: &[u8],
    ) -> BoxFuture<'_, Result<()>> {
      let file = file.to_string_lossy().to_string();
      let data = data.to_vec();
      let fut = async move { tokio::fs::write(file, data).await.map_err(Error::from) };
      Box::pin(fut)
    }

    fn remove_file(&self, file: &Path) -> BoxFuture<'_, Result<()>> {
      let file = file.to_string_lossy().to_string();
      let fut = async move { tokio::fs::remove_file(file).await.map_err(Error::from) };
      Box::pin(fut)
    }

    fn remove_dir_all(&self, dir: &Path) -> BoxFuture<'_, Result<()>> {
      let dir = dir.to_string_lossy().to_string();
      let fut = async move { tokio::fs::remove_dir_all(dir).await.map_err(Error::from) };
      Box::pin(fut)
    }
  }

  impl AsyncReadableFileSystem for AsyncNativeFileSystem {
    fn read(&self, file: &Path) -> BoxFuture<'_, Result<Vec<u8>>> {
      let file = file.to_string_lossy().to_string();
      let fut = async move { tokio::fs::read(file).await.map_err(Error::from) };
      Box::pin(fut)
    }
  }
}
