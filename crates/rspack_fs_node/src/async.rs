use futures::future::BoxFuture;
use napi::Env;
use rspack_fs::r#async::AsyncWritableFileSystem;
use rspack_napi_shared::threadsafe_function::ThreadsafeFunctionCallMode;

use crate::node::{ThreadsafeFunctionRef, ThreadsafeNodeFS, TryIntoThreadsafeFunctionRef};

pub struct AsyncNodeWritableFileSystem {
  fs_ts: ThreadsafeFunctionRef,
}

impl AsyncNodeWritableFileSystem {
  pub fn new(env: Env, fs_ts: ThreadsafeNodeFS) -> napi::Result<Self> {
    let fs_ts = fs_ts.try_into_tsfn_ref(&env)?;
    Ok(Self { fs_ts })
  }
}

impl AsyncWritableFileSystem for AsyncNodeWritableFileSystem {
  fn create_dir<P: AsRef<std::path::Path>>(&self, dir: P) -> BoxFuture<'_, rspack_fs::Result<()>> {
    let dir = dir.as_ref().to_string_lossy().to_string();
    let fut = async move {
      self
        .fs_ts
        .mkdir
        .call(dir, ThreadsafeFunctionCallMode::NonBlocking)
        .expect("Failed to call tsfn")
        .await
        .expect("Failed to poll")
        .map_err(|e| {
          rspack_fs::Error::Io(std::io::Error::new(
            std::io::ErrorKind::Other,
            e.to_string(),
          ))
        })
    };

    Box::pin(fut)
  }

  fn create_dir_all<P: AsRef<std::path::Path>>(
    &self,
    dir: P,
  ) -> BoxFuture<'_, rspack_fs::Result<()>> {
    let dir = dir.as_ref().to_string_lossy().to_string();
    let fut = async move {
      self
        .fs_ts
        .mkdirp
        .call(dir, ThreadsafeFunctionCallMode::NonBlocking)
        .expect("Failed to call tsfn")
        .await
        .expect("Failed to poll")
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

  fn write<P: AsRef<std::path::Path>, D: AsRef<[u8]>>(
    &self,
    file: P,
    data: D,
  ) -> BoxFuture<'_, rspack_fs::Result<()>> {
    let file = file.as_ref().to_string_lossy().to_string();
    let data = data.as_ref().to_vec();
    let fut = async move {
      self
        .fs_ts
        .write_file
        .call((file, data), ThreadsafeFunctionCallMode::NonBlocking)
        .expect("Failed to call tsfn")
        .await
        .expect("Failed to poll")
        .map_err(|e| {
          rspack_fs::Error::Io(std::io::Error::new(
            std::io::ErrorKind::Other,
            e.to_string(),
          ))
        })
    };
    Box::pin(fut)
  }

  fn remove_file<P: AsRef<std::path::Path>>(
    &self,
    file: P,
  ) -> BoxFuture<'_, rspack_fs::Result<()>> {
    let file = file.as_ref().to_string_lossy().to_string();
    let fut = async move {
      self
        .fs_ts
        .remove_file
        .call(file, ThreadsafeFunctionCallMode::NonBlocking)
        .expect("Failed to call tsfn")
        .await
        .expect("Failed to poll")
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

  fn remove_dir_all<P: AsRef<std::path::Path>>(
    &self,
    dir: P,
  ) -> BoxFuture<'_, rspack_fs::Result<()>> {
    let dir = dir.as_ref().to_string_lossy().to_string();
    let fut = async move {
      self
        .fs_ts
        .remove_dir_all
        .call(dir, ThreadsafeFunctionCallMode::NonBlocking)
        .expect("Failed to call tsfn")
        .await
        .expect("Failed to poll")
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
