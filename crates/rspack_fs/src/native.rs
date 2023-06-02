use std::{fs, path::Path};

use super::{
  cfg_async,
  sync::{ReadableFileSystem, WritableFileSystem},
  Error, Result,
};

pub struct NativeFileSystem;

impl WritableFileSystem for NativeFileSystem {
  fn create_dir<P: AsRef<Path>>(&self, dir: P) -> Result<()> {
    fs::create_dir(dir.as_ref()).map_err(Error::from)
  }

  fn create_dir_all<P: AsRef<std::path::Path>>(&self, dir: P) -> Result<()> {
    fs::create_dir_all(dir.as_ref()).map_err(Error::from)
  }

  fn write<P: AsRef<std::path::Path>, D: AsRef<[u8]>>(&self, file: P, data: D) -> Result<()> {
    fs::write(file.as_ref(), data.as_ref()).map_err(Error::from)
  }
}

impl ReadableFileSystem for NativeFileSystem {
  fn read<P: AsRef<Path>>(&self, file: P) -> Result<Vec<u8>> {
    fs::read(file.as_ref()).map_err(Error::from)
  }
}

cfg_async! {
  use futures::future::BoxFuture;

  use crate::{AsyncReadableFileSystem, AsyncWritableFileSystem};
  pub struct AsyncNativeFileSystem;

  impl AsyncWritableFileSystem for AsyncNativeFileSystem {
    fn create_dir<P: AsRef<Path>>(&self, dir: P) -> BoxFuture<'_, Result<()>> {
      let dir = dir.as_ref().to_string_lossy().to_string();
      let fut = async move { tokio::fs::create_dir(dir).await.map_err(Error::from) };
      Box::pin(fut)
    }

    fn create_dir_all<P: AsRef<std::path::Path>>(&self, dir: P) -> BoxFuture<'_, Result<()>> {
      let dir = dir.as_ref().to_string_lossy().to_string();
      let fut = async move { tokio::fs::create_dir_all(dir).await.map_err(Error::from) };
      Box::pin(fut)
    }

    fn write<P: AsRef<std::path::Path>, D: AsRef<[u8]>>(
      &self,
      file: P,
      data: D,
    ) -> BoxFuture<'_, Result<()>> {
      let file = file.as_ref().to_string_lossy().to_string();
      let data = data.as_ref().to_vec();
      let fut = async move { tokio::fs::write(file, data).await.map_err(Error::from) };
      Box::pin(fut)
    }

    fn remove_file<P: AsRef<Path>>(&self, file: P) -> BoxFuture<'_, Result<()>> {
      let file = file.as_ref().to_string_lossy().to_string();
      let fut = async move { tokio::fs::remove_file(file).await.map_err(Error::from) };
      Box::pin(fut)
    }

    fn remove_dir_all<P: AsRef<Path>>(&self, dir: P) -> BoxFuture<'_, Result<()>> {
      let dir = dir.as_ref().to_string_lossy().to_string();
      let fut = async move { tokio::fs::remove_dir_all(dir).await.map_err(Error::from) };
      Box::pin(fut)
    }
  }

  impl AsyncReadableFileSystem for AsyncNativeFileSystem {
    fn read<P: AsRef<Path>>(&self, file: P) -> BoxFuture<'_, Result<Vec<u8>>> {
      let file = file.as_ref().to_string_lossy().to_string();
      let fut = async move { tokio::fs::read(file).await.map_err(Error::from) };
      Box::pin(fut)
    }
  }
}
