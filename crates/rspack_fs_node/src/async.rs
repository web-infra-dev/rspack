use std::path::Path;

use futures::future::BoxFuture;
use rspack_fs::r#async::AsyncWritableFileSystem;

use crate::node::ThreadsafeNodeFS;

pub struct AsyncNodeWritableFileSystem(ThreadsafeNodeFS);

impl AsyncNodeWritableFileSystem {
  pub fn new(tsfs: ThreadsafeNodeFS) -> napi::Result<Self> {
    Ok(Self(tsfs))
  }
}

impl AsyncWritableFileSystem for AsyncNodeWritableFileSystem {
  fn create_dir(&self, dir: &Path) -> BoxFuture<'_, rspack_fs::Result<()>> {
    let dir = dir.to_string_lossy().to_string();
    let fut = async move {
      self.0.mkdir.call(dir).await.map_err(|e| {
        rspack_fs::Error::Io(std::io::Error::new(
          std::io::ErrorKind::Other,
          e.to_string(),
        ))
      })
    };

    Box::pin(fut)
  }

  fn create_dir_all(&self, dir: &Path) -> BoxFuture<'_, rspack_fs::Result<()>> {
    let dir = dir.to_string_lossy().to_string();
    let fut = async move {
      self
        .0
        .mkdirp
        .call(dir)
        .await
        .map_err(|e| {
          rspack_fs::Error::Io(std::io::Error::new(
            std::io::ErrorKind::Other,
            e.to_string(),
          ))
        })
        .map(|_| ())
    };
    Box::pin(fut)
  }

  fn write(&self, file: &Path, data: &[u8]) -> BoxFuture<'_, rspack_fs::Result<()>> {
    let file = file.to_string_lossy().to_string();
    let data = data.to_vec();
    let fut = async move {
      self
        .0
        .write_file
        .call((file, data.into()))
        .await
        .map_err(|e| {
          rspack_fs::Error::Io(std::io::Error::new(
            std::io::ErrorKind::Other,
            e.to_string(),
          ))
        })
    };
    Box::pin(fut)
  }

  fn remove_file(&self, file: &Path) -> BoxFuture<'_, rspack_fs::Result<()>> {
    let file = file.to_string_lossy().to_string();
    let fut = async move {
      self
        .0
        .remove_file
        .call(file)
        .await
        .map_err(|e| {
          rspack_fs::Error::Io(std::io::Error::new(
            std::io::ErrorKind::Other,
            e.to_string(),
          ))
        })
        .map(|_| ())
    };
    Box::pin(fut)
  }

  fn remove_dir_all(&self, dir: &Path) -> BoxFuture<'_, rspack_fs::Result<()>> {
    let dir = dir.to_string_lossy().to_string();
    let fut = async move {
      self
        .0
        .remove_dir_all
        .call(dir)
        .await
        .map_err(|e| {
          rspack_fs::Error::Io(std::io::Error::new(
            std::io::ErrorKind::Other,
            e.to_string(),
          ))
        })
        .map(|_| ())
    };
    Box::pin(fut)
  }
}
