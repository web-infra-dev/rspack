use std::sync::Arc;

use async_trait::async_trait;
use napi::{bindgen_prelude::Either3, Either};
use rspack_fs::{
  dunce, Error, FileMetadata, FileSystem, IntermediateFileSystemExtras, ReadStream,
  ReadableFileSystem, Result, WritableFileSystem, WriteStream,
};
use rspack_paths::{AssertUtf8, Utf8Path};

use crate::node::ThreadsafeNodeFS;

fn map_error_to_fs_error(e: rspack_error::Error) -> Error {
  Error::Io(std::io::Error::new(
    std::io::ErrorKind::Other,
    e.to_string(),
  ))
}

fn new_fs_error(msg: &str) -> Error {
  Error::Io(std::io::Error::new(std::io::ErrorKind::Other, msg))
}

pub struct NodeFileSystem(Arc<ThreadsafeNodeFS>);

impl std::fmt::Debug for NodeFileSystem {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("AsyncNodeWritableFileSystem").finish()
  }
}

impl FileSystem for NodeFileSystem {}

impl NodeFileSystem {
  pub fn new(tsfs: ThreadsafeNodeFS) -> napi::Result<Self> {
    Ok(Self(Arc::new(tsfs)))
  }
}
#[async_trait]
impl WritableFileSystem for NodeFileSystem {
  async fn create_dir(&self, dir: &Utf8Path) -> Result<()> {
    let dir = dir.as_str().to_string();
    self
      .0
      .mkdir
      .call_with_promise(dir)
      .await
      .map_err(map_error_to_fs_error)
  }

  async fn create_dir_all(&self, dir: &Utf8Path) -> Result<()> {
    let dir = dir.as_str().to_string();
    self
      .0
      .mkdirp
      .call_with_promise(dir)
      .await
      .map_err(map_error_to_fs_error)
      .map(|_| ())
  }

  async fn write(&self, file: &Utf8Path, data: &[u8]) -> Result<()> {
    let file = file.as_str().to_string();
    let data = data.to_vec();
    self
      .0
      .write_file
      .call_with_promise((file, data.into()))
      .await
      .map_err(map_error_to_fs_error)
  }

  async fn remove_file(&self, file: &Utf8Path) -> Result<()> {
    let file = file.as_str().to_string();
    self
      .0
      .remove_file
      .call_with_promise(file)
      .await
      .map_err(map_error_to_fs_error)
      .map(|_| ())
  }

  async fn remove_dir_all(&self, dir: &Utf8Path) -> Result<()> {
    let dir = dir.as_str().to_string();
    self
      .0
      .remove_dir_all
      .call_with_promise(dir)
      .await
      .map_err(map_error_to_fs_error)
      .map(|_| ())
  }

  // TODO: support read_dir options
  async fn read_dir(&self, dir: &Utf8Path) -> Result<Vec<String>> {
    let dir = dir.as_str().to_string();
    let res = self
      .0
      .read_dir
      .call_with_promise(dir)
      .await
      .map_err(map_error_to_fs_error)?;
    match res {
      Either::A(files) => Ok(files),
      Either::B(_) => Err(Error::Io(std::io::Error::new(
        std::io::ErrorKind::Other,
        "output file system call read dir failed",
      ))),
    }
  }

  // TODO: support read_file options
  async fn read_file(&self, file: &Utf8Path) -> Result<Vec<u8>> {
    let file = file.as_str().to_string();
    let res = self
      .0
      .read_file
      .call_with_promise(file)
      .await
      .map_err(map_error_to_fs_error)?;

    match res {
      Either3::A(data) => Ok(data.to_vec()),
      Either3::B(str) => Ok(str.into_bytes()),
      Either3::C(_) => Err(Error::Io(std::io::Error::new(
        std::io::ErrorKind::Other,
        "output file system call read file failed",
      ))),
    }
  }

  async fn stat(&self, file: &Utf8Path) -> Result<FileMetadata> {
    let file = file.as_str().to_string();
    let res = self
      .0
      .stat
      .call_with_promise(file)
      .await
      .map_err(map_error_to_fs_error)?;
    match res {
      Either::A(stat) => Ok(FileMetadata::from(stat)),
      Either::B(_) => Err(Error::Io(std::io::Error::new(
        std::io::ErrorKind::Other,
        "output file system call stat failed",
      ))),
    }
  }
}

#[async_trait]
impl IntermediateFileSystemExtras for NodeFileSystem {
  async fn rename(&self, from: &Utf8Path, to: &Utf8Path) -> Result<()> {
    let from = from.as_str().to_string();
    let to = to.as_str().to_string();
    self
      .0
      .rename
      .call_with_promise((from, to))
      .await
      .map_err(map_error_to_fs_error)
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

#[async_trait::async_trait]
impl ReadableFileSystem for NodeFileSystem {
  fn read(&self, path: &Utf8Path) -> Result<Vec<u8>> {
    let res = futures::executor::block_on(self.0.read_file.call_with_promise(path.to_string()))
      .map_err(map_error_to_fs_error)?;
    match res {
      Either3::A(buf) => Ok(buf.into()),
      Either3::B(s) => Ok(s.into()),
      Either3::C(_) => Err(new_fs_error("input file system call read failed")),
    }
  }

  fn metadata(&self, path: &Utf8Path) -> Result<FileMetadata> {
    let res = futures::executor::block_on(self.0.stat.call_with_promise(path.to_string()))
      .map_err(map_error_to_fs_error)?;
    match res {
      Either::A(stats) => Ok(stats.into()),
      Either::B(_) => Err(new_fs_error("input file system call metadata failed")),
    }
  }

  fn symlink_metadata(&self, path: &Utf8Path) -> Result<FileMetadata> {
    self.metadata(path)
  }

  fn canonicalize(&self, path: &Utf8Path) -> Result<rspack_paths::Utf8PathBuf> {
    let path = dunce::canonicalize(path)?;
    Ok(path.assert_utf8())
  }

  async fn async_read(&self, file: &Utf8Path) -> Result<Vec<u8>> {
    let res = self
      .0
      .read_file
      .call_with_promise(file.to_string())
      .await
      .map_err(map_error_to_fs_error)?;
    match res {
      Either3::A(buf) => Ok(buf.into()),
      Either3::B(s) => Ok(s.into()),
      Either3::C(_) => Err(new_fs_error("input file system call async_read failed")),
    }
  }
}

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
      .call_with_promise((file.as_str().to_string(), "r".to_string()))
      .await
      .map_err(map_error_to_fs_error)?;

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
  async fn read(&mut self, buf: &mut [u8]) -> Result<()> {
    let length = buf.len();
    let buffer = self
      .fs
      .read
      .call_with_promise((self.fd, length as u32, self.pos as u32))
      .await
      .map_err(map_error_to_fs_error)?;

    match buffer {
      Either::A(buffer) => {
        self.pos += buffer.len();
        buf.copy_from_slice(&buffer);
        Ok(())
      }
      Either::B(_) => Err(Error::Io(std::io::Error::new(
        std::io::ErrorKind::Other,
        "file system call read failed",
      ))),
    }
  }

  async fn read_until(&mut self, byte: u8, buf: &mut Vec<u8>) -> Result<usize> {
    let buffer = self
      .fs
      .read_until
      .call_with_promise((self.fd, byte, self.pos as u32))
      .await
      .map_err(map_error_to_fs_error)?;

    match buffer {
      Either::A(buffer) => {
        self.pos += buffer.len() + 1;
        buf.copy_from_slice(&buffer);
        Ok(buffer.len())
      }
      Either::B(_) => Err(Error::Io(std::io::Error::new(
        std::io::ErrorKind::Other,
        "file system call read until failed",
      ))),
    }
  }
  async fn read_to_end(&mut self, buf: &mut Vec<u8>) -> Result<usize> {
    let buffer = self
      .fs
      .read_to_end
      .call_with_promise((self.fd, self.pos as u32))
      .await
      .map_err(map_error_to_fs_error)?;

    match buffer {
      Either::A(buffer) => {
        self.pos += buffer.len();
        buf.copy_from_slice(&buffer);
        Ok(buffer.len())
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
    self
      .fs
      .close
      .call_with_promise(self.fd)
      .await
      .map_err(map_error_to_fs_error)
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
      .call_with_promise((file.as_str().to_string(), "w+".to_string()))
      .await
      .map_err(map_error_to_fs_error)?;

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
  async fn write(&mut self, buf: &[u8]) -> Result<usize> {
    let res = self
      .fs
      .write
      .call_with_promise((self.fd, buf.to_vec().into(), self.pos as u32))
      .await
      .map_err(map_error_to_fs_error)?;

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
      .call_with_promise((self.fd, buf.to_vec().into()))
      .await
      .map_err(map_error_to_fs_error)
  }
  async fn flush(&mut self) -> Result<()> {
    Ok(())
  }
  async fn close(&mut self) -> Result<()> {
    self
      .fs
      .close
      .call_with_promise(self.fd)
      .await
      .map_err(map_error_to_fs_error)
  }
}
