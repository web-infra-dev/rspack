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
  fn create_dir<P: AsRef<std::path::Path>>(&self, dir: P) -> BoxFuture<'_, rspack_fs::Result<()>> {
    let dir = dir.as_ref().to_string_lossy().to_string();
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

  fn create_dir_all<P: AsRef<std::path::Path>>(
    &self,
    dir: P,
  ) -> BoxFuture<'_, rspack_fs::Result<()>> {
    let dir = dir.as_ref().to_string_lossy().to_string();
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

  fn write<P: AsRef<std::path::Path>, D: AsRef<[u8]>>(
    &self,
    file: P,
    data: D,
  ) -> BoxFuture<'_, rspack_fs::Result<()>> {
    let file = file.as_ref().to_string_lossy().to_string();
    let data = data.as_ref().to_vec();
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

  fn remove_file<P: AsRef<std::path::Path>>(
    &self,
    file: P,
  ) -> BoxFuture<'_, rspack_fs::Result<()>> {
    let file = file.as_ref().to_string_lossy().to_string();
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

  fn remove_dir_all<P: AsRef<std::path::Path>>(
    &self,
    dir: P,
  ) -> BoxFuture<'_, rspack_fs::Result<()>> {
    let dir = dir.as_ref().to_string_lossy().to_string();
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
