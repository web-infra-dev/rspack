use futures::future::BoxFuture;
use rspack_fs::r#async::AsyncWritableFileSystem;
use rspack_paths::Utf8Path;

use crate::node::ThreadsafeNodeFS;

pub struct AsyncNodeWritableFileSystem(ThreadsafeNodeFS);

impl std::fmt::Debug for AsyncNodeWritableFileSystem {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("AsyncNodeWritableFileSystem").finish()
  }
}

impl AsyncNodeWritableFileSystem {
  pub fn new(tsfs: ThreadsafeNodeFS) -> napi::Result<Self> {
    Ok(Self(tsfs))
  }
}

impl AsyncWritableFileSystem for AsyncNodeWritableFileSystem {
  fn create_dir<'a>(&'a self, dir: &'a Utf8Path) -> BoxFuture<'a, rspack_fs::Result<()>> {
    let fut = async {
      let dir = dir.as_str().to_string();
      self.0.mkdir.call(dir).await.map_err(|e| {
        rspack_fs::Error::Io(std::io::Error::new(
          std::io::ErrorKind::Other,
          e.to_string(),
        ))
      })
    };

    Box::pin(fut)
  }

  fn create_dir_all<'a>(&'a self, dir: &'a Utf8Path) -> BoxFuture<'a, rspack_fs::Result<()>> {
    let fut = async {
      let dir = dir.as_str().to_string();
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

  fn write<'a>(
    &'a self,
    file: &'a Utf8Path,
    data: &'a [u8],
  ) -> BoxFuture<'a, rspack_fs::Result<()>> {
    let fut = async {
      let file = file.as_str().to_string();
      let data = data.to_vec();
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

  fn remove_file<'a>(&'a self, file: &'a Utf8Path) -> BoxFuture<'a, rspack_fs::Result<()>> {
    let fut = async {
      let file = file.as_str().to_string();
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

  fn remove_dir_all<'a>(&'a self, dir: &'a Utf8Path) -> BoxFuture<'a, rspack_fs::Result<()>> {
    let fut = async {
      let dir = dir.as_str().to_string();
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
