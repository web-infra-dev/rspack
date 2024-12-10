use std::sync::Arc;

use async_trait::async_trait;
use futures::future::BoxFuture;
use napi::{bindgen_prelude::Either3, Either};
use rspack_fs::{
  Error, FileMetadata, IntermediateFileSystem, IntermediateFileSystemExtras, ReadStream, Result,
  WritableFileSystem, WriteStream,
};
use rspack_paths::Utf8Path;

use crate::node::ThreadsafeNodeFS;

pub struct NodeFileSystem(Arc<ThreadsafeNodeFS>);

impl std::fmt::Debug for NodeFileSystem {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("AsyncNodeWritableFileSystem").finish()
  }
}

impl NodeFileSystem {
  pub fn new(tsfs: ThreadsafeNodeFS) -> napi::Result<Self> {
    Ok(Self(Arc::new(tsfs)))
  }
}
#[async_trait]
impl WritableFileSystem for NodeFileSystem {
  async fn create_dir(&self, dir: &Utf8Path) -> Result<()> {
    let fut = async {
      let dir = dir.as_str().to_string();
      self.0.mkdir.call(dir).await.map_err(|e| {
        Error::Io(std::io::Error::new(
          std::io::ErrorKind::Other,
          e.to_string(),
        ))
      })
    };

    fut.await
  }

  async fn create_dir_all(&self, dir: &Utf8Path) -> Result<()> {
    let fut = async {
      let dir = dir.as_str().to_string();
      self
        .0
        .mkdirp
        .call(dir)
        .await
        .map_err(|e| {
          Error::Io(std::io::Error::new(
            std::io::ErrorKind::Other,
            e.to_string(),
          ))
        })
        .map(|_| ())
    };
    fut.await
  }

  async fn write(&self, file: &Utf8Path, data: &[u8]) -> Result<()> {
    let fut = async {
      let file = file.as_str().to_string();
      let data = data.to_vec();
      self
        .0
        .write_file
        .call((file, data.into()))
        .await
        .map_err(|e| {
          Error::Io(std::io::Error::new(
            std::io::ErrorKind::Other,
            e.to_string(),
          ))
        })
    };
    fut.await
  }

  fn remove_file<'a>(&'a self, file: &'a Utf8Path) -> BoxFuture<'a, Result<()>> {
    let fut = async {
      let file = file.as_str().to_string();
      self
        .0
        .remove_file
        .call(file)
        .await
        .map_err(|e| {
          Error::Io(std::io::Error::new(
            std::io::ErrorKind::Other,
            e.to_string(),
          ))
        })
        .map(|_| ())
    };
    Box::pin(fut)
  }

  fn remove_dir_all<'a>(&'a self, dir: &'a Utf8Path) -> BoxFuture<'a, Result<()>> {
    let fut = async {
      let dir = dir.as_str().to_string();
      self
        .0
        .remove_dir_all
        .call(dir)
        .await
        .map_err(|e| {
          Error::Io(std::io::Error::new(
            std::io::ErrorKind::Other,
            e.to_string(),
          ))
        })
        .map(|_| ())
    };
    Box::pin(fut)
  }

  // TODO: support read_dir options
  fn read_dir<'a>(&'a self, dir: &'a Utf8Path) -> BoxFuture<'a, Result<Vec<String>>> {
    let fut = async {
      let dir = dir.as_str().to_string();
      let res = self.0.read_dir.call(dir).await.map_err(|e| {
        Error::Io(std::io::Error::new(
          std::io::ErrorKind::Other,
          e.to_string(),
        ))
      })?;
      match res {
        Either::A(files) => Ok(files),
        Either::B(_) => Err(Error::Io(std::io::Error::new(
          std::io::ErrorKind::Other,
          "output file system call read dir failed",
        ))),
      }
    };
    Box::pin(fut)
  }

  // TODO: support read_file options
  fn read_file<'a>(&'a self, file: &'a Utf8Path) -> BoxFuture<'a, Result<Vec<u8>>> {
    let fut = async {
      let file = file.as_str().to_string();
      let res = self.0.read_file.call(file).await.map_err(|e| {
        Error::Io(std::io::Error::new(
          std::io::ErrorKind::Other,
          e.to_string(),
        ))
      })?;

      match res {
        Either3::A(data) => Ok(data.to_vec()),
        Either3::B(str) => Ok(str.into_bytes()),
        Either3::C(_) => Err(Error::Io(std::io::Error::new(
          std::io::ErrorKind::Other,
          "output file system call read file failed",
        ))),
      }
    };
    Box::pin(fut)
  }

  fn stat<'a>(&'a self, file: &'a Utf8Path) -> BoxFuture<'a, Result<FileMetadata>> {
    let fut = async {
      let file = file.as_str().to_string();
      let res = self.0.stat.call(file).await.map_err(|e| {
        Error::Io(std::io::Error::new(
          std::io::ErrorKind::Other,
          e.to_string(),
        ))
      })?;
      match res {
        Either::A(stat) => Ok(FileMetadata::from(stat)),
        Either::B(_) => Err(Error::Io(std::io::Error::new(
          std::io::ErrorKind::Other,
          "output file system call stat failed",
        ))),
      }
    };
    Box::pin(fut)
  }
}

#[async_trait]
impl IntermediateFileSystemExtras for NodeFileSystem {
  async fn rename(&self, from: &Utf8Path, to: &Utf8Path) -> Result<()> {
    let fut = async {
      let from = from.as_str().to_string();
      let to = to.as_str().to_string();
      self.0.rename.call((from, to)).await.map_err(|e| {
        Error::Io(std::io::Error::new(
          std::io::ErrorKind::Other,
          e.to_string(),
        ))
      })
    };
    fut.await
  }

  async fn create_read_stream(&self, file: &Utf8Path) -> Result<Box<dyn ReadStream>> {
    let reader = NodeReadStream::try_new(file, self.0.clone()).await?;
    Ok(Box::new(reader))
  }

  async fn create_write_stream(&self, file: &Utf8Path) -> Result<Box<dyn WriteStream>> {
    let writer = NodeWriteStream::try_new(file, self.0.clone()).await?;
    Ok(Box::new(writer))
  }
}

impl IntermediateFileSystem for NodeFileSystem {}

#[derive(Debug)]
pub struct NodeReadStream {
  fd: i32,
  pos: usize,
  fs: Arc<ThreadsafeNodeFS>,
}

impl NodeReadStream {
  pub async fn try_new(file: &Utf8Path, fs: Arc<ThreadsafeNodeFS>) -> Result<Self> {
    let res = fs
      .open
      .call((file.as_str().to_string(), "r".to_string()))
      .await
      .map_err(|e| {
        Error::Io(std::io::Error::new(
          std::io::ErrorKind::Other,
          e.to_string(),
        ))
      })?;

    match res {
      Either::A(fd) => Ok(Self { fd, pos: 0, fs }),
      Either::B(_) => Err(Error::Io(std::io::Error::new(
        std::io::ErrorKind::Other,
        "file system call read open failed",
      ))),
    }
  }
}

#[async_trait::async_trait]
impl ReadStream for NodeReadStream {
  async fn read(&mut self, length: usize) -> Result<Vec<u8>> {
    let buffer = self
      .fs
      .read
      .call((self.fd, length as u32, self.pos as u32))
      .await
      .map_err(|e| {
        Error::Io(std::io::Error::new(
          std::io::ErrorKind::Other,
          e.to_string(),
        ))
      })?;

    match buffer {
      Either::A(buffer) => {
        self.pos += buffer.len();
        Ok(buffer.to_vec())
      }
      Either::B(_) => Err(Error::Io(std::io::Error::new(
        std::io::ErrorKind::Other,
        "file system call read failed",
      ))),
    }
  }

  async fn read_until(&mut self, byte: u8) -> Result<Vec<u8>> {
    let buffer = self
      .fs
      .read_until
      .call((self.fd, byte, self.pos as u32))
      .await
      .map_err(|e| {
        Error::Io(std::io::Error::new(
          std::io::ErrorKind::Other,
          e.to_string(),
        ))
      })?;

    match buffer {
      Either::A(buffer) => {
        self.pos += buffer.len() + 1;
        Ok(buffer.to_vec())
      }
      Either::B(_) => Err(Error::Io(std::io::Error::new(
        std::io::ErrorKind::Other,
        "file system call read until failed",
      ))),
    }
  }
  async fn read_to_end(&mut self) -> Result<Vec<u8>> {
    let buffer = self
      .fs
      .read_to_end
      .call((self.fd, self.pos as u32))
      .await
      .map_err(|e| {
        Error::Io(std::io::Error::new(
          std::io::ErrorKind::Other,
          e.to_string(),
        ))
      })?;

    match buffer {
      Either::A(buffer) => {
        self.pos += buffer.len();
        Ok(buffer.to_vec())
      }
      Either::B(_) => Err(Error::Io(std::io::Error::new(
        std::io::ErrorKind::Other,
        "file system call read to end failed",
      ))),
    }
  }
  async fn skip(&mut self, offset: usize) -> Result<()> {
    self.pos += offset;
    Ok(())
  }
  async fn close(&mut self) -> Result<()> {
    self.fs.close.call(self.fd).await.map_err(|e| {
      Error::Io(std::io::Error::new(
        std::io::ErrorKind::Other,
        e.to_string(),
      ))
    })
  }
}

#[derive(Debug)]
pub struct NodeWriteStream {
  fd: i32,
  pos: usize,
  fs: Arc<ThreadsafeNodeFS>,
}

impl NodeWriteStream {
  pub async fn try_new(file: &Utf8Path, fs: Arc<ThreadsafeNodeFS>) -> Result<Self> {
    let res = fs
      .open
      .call((file.as_str().to_string(), "w+".to_string()))
      .await
      .map_err(|e| {
        Error::Io(std::io::Error::new(
          std::io::ErrorKind::Other,
          e.to_string(),
        ))
      })?;

    match res {
      Either::A(fd) => Ok(Self { fd, pos: 0, fs }),
      Either::B(_) => Err(Error::Io(std::io::Error::new(
        std::io::ErrorKind::Other,
        "file system call write open failed",
      ))),
    }
  }
}

#[async_trait::async_trait]
impl WriteStream for NodeWriteStream {
  async fn write_line(&mut self, line: &str) -> Result<()> {
    self.write(line.as_bytes()).await?;
    self.write(b"\n").await?;
    Ok(())
  }
  async fn write(&mut self, buf: &[u8]) -> Result<usize> {
    let res = self
      .fs
      .write
      .call((self.fd, buf.to_vec().into(), self.pos as u32))
      .await
      .map_err(|e| {
        Error::Io(std::io::Error::new(
          std::io::ErrorKind::Other,
          e.to_string(),
        ))
      })?;

    match res {
      Either::A(size) => {
        self.pos += size as usize;
        Ok(size as usize)
      }
      Either::B(_) => Err(Error::Io(std::io::Error::new(
        std::io::ErrorKind::Other,
        "file system call write failed",
      ))),
    }
  }
  async fn write_all(&mut self, buf: &[u8]) -> Result<()> {
    self
      .fs
      .write_all
      .call((self.fd, buf.to_vec().into()))
      .await
      .map_err(|e| {
        Error::Io(std::io::Error::new(
          std::io::ErrorKind::Other,
          e.to_string(),
        ))
      })
  }
  async fn flush(&mut self) -> Result<()> {
    Ok(())
  }
  async fn close(&mut self) -> Result<()> {
    self.fs.close.call(self.fd).await.map_err(|e| {
      Error::Io(std::io::Error::new(
        std::io::ErrorKind::Other,
        e.to_string(),
      ))
    })
  }
}
