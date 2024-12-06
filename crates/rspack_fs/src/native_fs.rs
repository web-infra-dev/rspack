use std::{
  fs::{self, File},
  io::{BufRead, BufReader, BufWriter, Read, Write},
};

use futures::future::BoxFuture;
use rspack_paths::{AssertUtf8, Utf8Path, Utf8PathBuf};

use crate::{
  Error, FileMetadata, FileSystem, IntermediateFileSystemExtras, ReadStream, ReadableFileSystem,
  Result, WritableFileSystem, WriteStream,
};

#[derive(Debug)]
pub struct NativeFileSystem;
impl FileSystem for NativeFileSystem {}
#[async_trait::async_trait]
impl WritableFileSystem for NativeFileSystem {
  async fn create_dir(&self, dir: &Utf8Path) -> Result<()> {
    fs::create_dir(dir).map_err(Error::from)
  }

  async fn create_dir_all(&self, dir: &Utf8Path) -> Result<()> {
    fs::create_dir_all(dir).map_err(Error::from)
  }

  async fn write(&self, file: &Utf8Path, data: &[u8]) -> Result<()> {
    fs::write(file, data).map_err(Error::from)
  }

  async fn remove_file(&self, file: &Utf8Path) -> Result<()> {
    tokio::fs::remove_file(file).await.map_err(Error::from)
  }

  async fn remove_dir_all(&self, dir: &Utf8Path) -> Result<()> {
    let dir = dir.to_path_buf();
    tokio::fs::remove_dir_all(dir).await.map_err(Error::from)
  }

  async fn read_dir(&self, dir: &Utf8Path) -> Result<Vec<String>> {
    let dir = dir.to_path_buf();
    let mut reader = tokio::fs::read_dir(dir).await.map_err(Error::from)?;
    let mut res = vec![];
    while let Some(entry) = reader.next_entry().await.map_err(Error::from)? {
      res.push(entry.file_name().to_string_lossy().to_string());
    }
    Ok(res)
  }

  async fn read_file(&self, file: &Utf8Path) -> Result<Vec<u8>> {
    tokio::fs::read(file).await.map_err(Error::from)
  }

  async fn stat(&self, file: &Utf8Path) -> Result<FileMetadata> {
    let metadata = tokio::fs::metadata(file).await.map_err(Error::from)?;
    FileMetadata::try_from(metadata)
  }
}

impl ReadableFileSystem for NativeFileSystem {
  fn read(&self, path: &Utf8Path) -> Result<Vec<u8>> {
    fs::read(path).map_err(Error::from)
  }

  fn metadata(&self, path: &Utf8Path) -> Result<FileMetadata> {
    let meta = fs::metadata(path)?;
    meta.try_into()
  }

  fn symlink_metadata(&self, path: &Utf8Path) -> Result<FileMetadata> {
    let meta = fs::symlink_metadata(path)?;
    meta.try_into()
  }

  fn canonicalize(&self, path: &Utf8Path) -> Result<Utf8PathBuf> {
    let path = dunce::canonicalize(path)?;
    Ok(path.assert_utf8())
  }

  fn async_read<'a>(&'a self, file: &'a Utf8Path) -> BoxFuture<'a, Result<Vec<u8>>> {
    let fut = async move { tokio::fs::read(file).await.map_err(Error::from) };
    Box::pin(fut)
  }
}

#[async_trait::async_trait]
impl IntermediateFileSystemExtras for NativeFileSystem {
  async fn rename(&self, from: &Utf8Path, to: &Utf8Path) -> Result<()> {
    fs::rename(from, to).map_err(Error::from)
  }

  async fn create_read_stream(&self, file: &Utf8Path) -> Result<Box<dyn ReadStream>> {
    let reader = NativeReadStream::try_new(file)?;
    Ok(Box::new(reader))
  }

  async fn create_write_stream(&self, file: &Utf8Path) -> Result<Box<dyn WriteStream>> {
    let writer = NativeWriteStream::try_new(file)?;
    Ok(Box::new(writer))
  }
}

#[derive(Debug)]
pub struct NativeReadStream(BufReader<File>);

impl NativeReadStream {
  pub fn try_new(file: &Utf8Path) -> Result<Self> {
    let file = File::open(file).map_err(Error::from)?;
    Ok(Self(BufReader::new(file)))
  }
}

#[async_trait::async_trait]
impl ReadStream for NativeReadStream {
  async fn read(&mut self, buf: &mut [u8]) -> Result<()> {
    self.0.read_exact(buf).map_err(Error::from)
  }

  async fn read_until(&mut self, byte: u8, buf: &mut Vec<u8>) -> Result<usize> {
    self.0.read_until(byte, buf).map_err(Error::from)
  }
  async fn read_to_end(&mut self, buf: &mut Vec<u8>) -> Result<usize> {
    self.0.read_to_end(buf).map_err(Error::from)
  }
  async fn skip(&mut self, offset: usize) -> Result<()> {
    self.0.seek_relative(offset as i64).map_err(Error::from)
  }
  async fn close(&mut self) -> Result<()> {
    Ok(())
  }
}

#[derive(Debug)]
pub struct NativeWriteStream(BufWriter<File>);

impl NativeWriteStream {
  pub fn try_new(file: &Utf8Path) -> Result<Self> {
    let file = File::open(file).map_err(Error::from)?;
    Ok(Self(BufWriter::new(file)))
  }
}

#[async_trait::async_trait]
impl WriteStream for NativeWriteStream {
  async fn write(&mut self, buf: &[u8]) -> Result<usize> {
    self.0.write(buf).map_err(Error::from)
  }
  async fn write_all(&mut self, buf: &[u8]) -> Result<()> {
    self.0.write_all(buf).map_err(Error::from)
  }
  async fn flush(&mut self) -> Result<()> {
    self.0.flush().map_err(Error::from)
  }
  async fn close(&mut self) -> Result<()> {
    Ok(())
  }
}
