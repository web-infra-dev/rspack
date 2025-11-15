use std::sync::Arc;

use async_trait::async_trait;
use napi::{
  Either,
  bindgen_prelude::{Either3, block_on},
};
use rspack_fs::{
  Error, FileMetadata, FilePermissions, IntermediateFileSystem, IntermediateFileSystemExtras,
  ReadStream, ReadableFileSystem, Result, RspackResultToFsResultExt, WritableFileSystem,
  WriteStream,
};
use rspack_paths::{Utf8Path, Utf8PathBuf};
use tracing::instrument;

use super::node::ThreadsafeNodeFS;

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
    let dir = dir.as_str().to_string();
    self.0.mkdir.call_with_promise(dir).await.to_fs_result()
  }

  async fn create_dir_all(&self, dir: &Utf8Path) -> Result<()> {
    let dir = dir.as_str().to_string();
    self
      .0
      .mkdirp
      .call_with_promise(dir)
      .await
      .to_fs_result()
      .map(|_| ())
  }

  async fn write(&self, file: &Utf8Path, data: &[u8]) -> Result<()> {
    let file = file.as_str().to_string();
    let data = data.to_vec();
    self
      .0
      .write_file
      .call_with_promise((file, data.into()).into())
      .await
      .to_fs_result()
  }

  async fn remove_file(&self, file: &Utf8Path) -> Result<()> {
    let file = file.as_str().to_string();
    self
      .0
      .remove_file
      .call_with_promise(file)
      .await
      .to_fs_result()
      .map(|_| ())
  }

  async fn remove_dir_all(&self, dir: &Utf8Path) -> Result<()> {
    let dir = dir.as_str().to_string();
    self
      .0
      .remove_dir_all
      .call_with_promise(dir)
      .await
      .to_fs_result()
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
      .to_fs_result()?;
    match res {
      Either::A(files) => Ok(files),
      Either::B(_) => Err(Error::new(
        std::io::ErrorKind::Other,
        "output file system call read dir failed:",
      )),
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
      .to_fs_result()?;

    match res {
      Either3::A(data) => Ok(data.to_vec()),
      Either3::B(str) => Ok(str.into_bytes()),
      Either3::C(_) => Err(Error::new(
        std::io::ErrorKind::Other,
        "output file system call read file failed:",
      )),
    }
  }

  async fn stat(&self, file: &Utf8Path) -> Result<FileMetadata> {
    let file = file.as_str().to_string();
    let res = self.0.stat.call_with_promise(file).await.to_fs_result()?;
    match res {
      Either::A(stat) => Ok(FileMetadata::from(stat)),
      Either::B(_) => Err(Error::new(
        std::io::ErrorKind::Other,
        "output file system call stat failed:",
      )),
    }
  }

  async fn set_permissions(&self, path: &Utf8Path, perm: FilePermissions) -> Result<()> {
    if let Some(mode) = perm.into_mode()
      && let Some(chmod) = &self.0.chmod
    {
      let file = path.as_str().to_string();
      return chmod
        .call_with_promise((file, mode).into())
        .await
        .to_fs_result();
    }
    Ok(())
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
      .call_with_promise((from, to).into())
      .await
      .to_fs_result()
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

#[async_trait]
impl ReadableFileSystem for NodeFileSystem {
  #[instrument(skip(self), level = "debug")]
  async fn read(&self, path: &Utf8Path) -> Result<Vec<u8>> {
    self
      .0
      .read_file
      .call_with_promise(path.as_str().to_string())
      .await
      .to_fs_result()
      // TODO: simplify the return value?
      .map(|result| match result {
        Either3::A(buf) => buf.into(),
        Either3::B(str) => str.into(),
        Either3::C(_) => vec![],
      })
  }
  #[instrument(skip(self), level = "debug")]
  fn read_sync(&self, path: &Utf8Path) -> Result<Vec<u8>> {
    block_on(self.read(path))
  }

  #[instrument(skip(self), level = "debug")]
  async fn metadata(&self, path: &Utf8Path) -> Result<FileMetadata> {
    let res = self
      .0
      .stat
      .call_with_promise(path.as_str().to_string())
      .await
      .to_fs_result()?;
    match res {
      Either::A(stats) => Ok(stats.into()),
      Either::B(_) => Err(Error::new(
        std::io::ErrorKind::Other,
        "input file system call stat failed",
      )),
    }
  }

  #[instrument(skip(self), level = "debug")]
  fn metadata_sync(&self, path: &Utf8Path) -> Result<FileMetadata> {
    block_on(self.metadata(path))
  }

  #[instrument(skip(self), level = "debug")]
  async fn symlink_metadata(&self, path: &Utf8Path) -> Result<FileMetadata> {
    let res = self
      .0
      .lstat
      .call_with_promise(path.as_str().to_string())
      .await
      .to_fs_result()?;
    match res {
      Either::A(stats) => Ok(stats.into()),
      Either::B(_) => Err(Error::new(
        std::io::ErrorKind::Other,
        "input file system call lstat failed",
      )),
    }
  }

  #[instrument(skip(self), level = "debug")]
  async fn canonicalize(&self, path: &Utf8Path) -> Result<Utf8PathBuf> {
    let res = self
      .0
      .realpath
      .call_with_promise(path.as_str().to_string())
      .await
      .to_fs_result()?;
    match res {
      Either::A(str) => Ok(Utf8PathBuf::from(str)),
      Either::B(_) => Err(Error::new(
        std::io::ErrorKind::Other,
        "input file system call realpath failed",
      )),
    }
  }

  #[instrument(skip(self), level = "debug")]
  async fn read_dir(&self, dir: &Utf8Path) -> Result<Vec<String>> {
    let res = self
      .0
      .read_dir
      .call_with_promise(dir.as_str().to_string())
      .await
      .to_fs_result()?;
    match res {
      Either::A(list) => Ok(list),
      Either::B(_) => Err(Error::new(
        std::io::ErrorKind::Other,
        "input file system call read_dir failed",
      )),
    }
  }
  #[instrument(skip(self), level = "debug")]
  fn read_dir_sync(&self, dir: &Utf8Path) -> Result<Vec<String>> {
    block_on(ReadableFileSystem::read_dir(self, dir))
  }
  #[instrument(skip(self), level = "debug")]
  async fn permissions(&self, path: &Utf8Path) -> Result<Option<FilePermissions>> {
    let res = self
      .0
      .stat
      .call_with_promise(path.as_str().to_string())
      .await
      .to_fs_result()?;
    match res {
      Either::A(stats) => Ok(Some(FilePermissions::from_mode(stats.mode))),
      Either::B(_) => Err(Error::new(
        std::io::ErrorKind::Other,
        "input file system call stat failed",
      )),
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
      .call_with_promise((file.as_str().to_string(), "r".to_string()).into())
      .await
      .to_fs_result()?;

    match res {
      Either::A(fd) => Ok(Self { fd, pos: 0, fs }),
      Either::B(_) => Err(Error::new(
        std::io::ErrorKind::Other,
        "file system call read open failed:",
      )),
    }
  }
}

#[async_trait]
impl ReadStream for NodeReadStream {
  async fn read(&mut self, length: usize) -> Result<Vec<u8>> {
    let buffer = self
      .fs
      .read
      .call_with_promise((self.fd, length as u32, self.pos as u32).into())
      .await
      .to_fs_result()?;

    match buffer {
      Either::A(buffer) => {
        self.pos += buffer.len();
        Ok(buffer.to_vec())
      }
      Either::B(_) => Err(Error::new(
        std::io::ErrorKind::Other,
        "file system call read failed:",
      )),
    }
  }

  async fn read_until(&mut self, byte: u8) -> Result<Vec<u8>> {
    let buffer = self
      .fs
      .read_until
      .call_with_promise((self.fd, byte, self.pos as u32).into())
      .await
      .to_fs_result()?;

    match buffer {
      Either::A(buffer) => {
        self.pos += buffer.len() + 1;
        Ok(buffer.to_vec())
      }
      Either::B(_) => Err(Error::new(
        std::io::ErrorKind::Other,
        "file system call read until failed:",
      )),
    }
  }
  async fn read_to_end(&mut self) -> Result<Vec<u8>> {
    let buffer = self
      .fs
      .read_to_end
      .call_with_promise((self.fd, self.pos as u32).into())
      .await
      .to_fs_result()?;

    match buffer {
      Either::A(buffer) => {
        self.pos += buffer.len();
        Ok(buffer.to_vec())
      }
      Either::B(_) => Err(Error::new(
        std::io::ErrorKind::Other,
        "file system call read to end failed:",
      )),
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
      .to_fs_result()
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
      .call_with_promise((file.as_str().to_string(), "w+".to_string()).into())
      .await
      .to_fs_result()?;

    match res {
      Either::A(fd) => Ok(Self { fd, pos: 0, fs }),
      Either::B(_) => Err(Error::new(
        std::io::ErrorKind::Other,
        "file system call write open failed:",
      )),
    }
  }
}

#[async_trait]
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
      .call_with_promise((self.fd, buf.to_vec().into(), self.pos as u32).into())
      .await
      .to_fs_result()?;

    match res {
      Either::A(size) => {
        self.pos += size as usize;
        Ok(size as usize)
      }
      Either::B(_) => Err(Error::new(
        std::io::ErrorKind::Other,
        "file system call write failed:",
      )),
    }
  }
  async fn write_all(&mut self, buf: &[u8]) -> Result<()> {
    self
      .fs
      .write_all
      .call_with_promise((self.fd, buf.to_vec().into()).into())
      .await
      .to_fs_result()
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
      .to_fs_result()
  }
}
