use std::fmt::{Debug, Formatter};
use std::path::Path;

use futures::future::BoxFuture;
use napi::Env;
use rspack_fs::{AsyncReadableFileSystem, AsyncWritableFileSystem};
use rspack_napi_shared::threadsafe_function::ThreadsafeFunctionCallMode;

use crate::node::{
  ThreadsafeFunctionRef, ThreadsafeNodeFS, ThreadsafeNodeInputFS, ThreadsafeNodeInputFSRef,
  TryIntoThreadsafeFunctionRef,
};

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

pub struct AsyncNodeReadableFileSystem {
  fs_ts: ThreadsafeNodeInputFSRef,
}

impl AsyncNodeReadableFileSystem {
  pub fn new(env: Env, fs_ts: ThreadsafeNodeInputFS) -> Self {
    let fs_ts = fs_ts
      .try_into_tsfn_ref(&env)
      .expect("failed to convert js function into rust");
    Self { fs_ts }
  }
}

impl Debug for AsyncNodeReadableFileSystem {
  fn fmt(&self, _f: &mut Formatter<'_>) -> std::fmt::Result {
    todo!()
  }
}

impl AsyncReadableFileSystem for AsyncNodeReadableFileSystem {
  fn read(&self, file: &dyn AsRef<Path>) -> BoxFuture<'_, rspack_fs::Result<Vec<u8>>> {
    let file = file.as_ref().to_string_lossy().to_string();
    let fut = async move {
      self
        .fs_ts
        .read_file
        .call(file, ThreadsafeFunctionCallMode::NonBlocking)
        .expect("failed to call tsfn")
        .await
        .expect("failed to poll")
        .map(Vec::from)
        .map_err(|e| {
          rspack_fs::Error::Io(std::io::Error::new(
            std::io::ErrorKind::Other,
            e.to_string(),
          ))
        })
    };
    Box::pin(fut)
  }

  fn metadata(
    &self,
    file: &dyn AsRef<Path>,
  ) -> BoxFuture<'_, rspack_fs::Result<rspack_fs::FSMetadata>> {
    let file = file.as_ref().to_string_lossy().to_string();
    let fut = async move {
      self
        .fs_ts
        .stat
        .call(file, ThreadsafeFunctionCallMode::NonBlocking)
        .expect("failed to call tsfn")
        .await
        .expect("failed to poll")
        .map(|a| a.into())
        .map_err(|e| {
          rspack_fs::Error::Io(std::io::Error::new(
            std::io::ErrorKind::Other,
            e.to_string(),
          ))
        })
    };
    Box::pin(fut)
  }

  fn symlink_metadata(
    &self,
    file: &dyn AsRef<Path>,
  ) -> BoxFuture<'_, rspack_fs::Result<rspack_fs::FSMetadata>> {
    let file = file.as_ref().to_string_lossy().to_string();
    let fut = async move {
      self
        .fs_ts
        .lstat
        .call(file, ThreadsafeFunctionCallMode::NonBlocking)
        .expect("failed to call tsfn")
        .await
        .expect("failed to poll")
        .map(|a| a.into())
        .map_err(|e| {
          rspack_fs::Error::Io(std::io::Error::new(
            std::io::ErrorKind::Other,
            e.to_string(),
          ))
        })
    };
    Box::pin(fut)
  }
}
