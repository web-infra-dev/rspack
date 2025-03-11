use std::{
  fs::{self, File},
  io::{BufRead, BufReader, BufWriter, Read, Write},
};

use pnp::fs::{FileType, LruZipCache, VPath, VPathInfo, ZipCache};
use rspack_paths::{AssertUtf8, Utf8Path, Utf8PathBuf};

use crate::{
  Error, FileMetadata, IntermediateFileSystem, IntermediateFileSystemExtras, ReadStream,
  ReadableFileSystem, Result, WritableFileSystem, WriteStream,
};
#[derive(Debug)]
struct NativeFileSystemOptions {
  // enable Yarn PnP feature
  pnp: bool,
}
#[derive(Debug)]
pub struct NativeFileSystem {
  options: NativeFileSystemOptions,
  pnp_lru: LruZipCache<Vec<u8>>,
}
impl NativeFileSystem {
  pub fn new(pnp: bool) -> Self {
    Self {
      options: NativeFileSystemOptions { pnp },
      pnp_lru: LruZipCache::new(50, pnp::fs::open_zip_via_read_p),
    }
  }
}

#[cfg(not(target_family = "wasm"))]
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

#[cfg(target_family = "wasm")]
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
    fs::remove_file(file).map_err(Error::from)
  }

  async fn remove_dir_all(&self, dir: &Utf8Path) -> Result<()> {
    let dir = dir.to_path_buf();
    fs::remove_dir_all(dir).map_err(Error::from)
  }

  async fn read_dir(&self, dir: &Utf8Path) -> Result<Vec<String>> {
    let dir = dir.to_path_buf();
    let mut res = vec![];
    let reader = fs::read_dir(dir).map_err(Error::from)?;
    for entry in reader {
      let entry = entry.map_err(Error::from)?;
      res.push(entry.file_name().to_string_lossy().to_string());
    }
    Ok(res)
  }

  async fn read_file(&self, file: &Utf8Path) -> Result<Vec<u8>> {
    fs::read(file).map_err(Error::from)
  }

  async fn stat(&self, file: &Utf8Path) -> Result<FileMetadata> {
    let metadata = fs::metadata(file).map_err(Error::from)?;
    FileMetadata::try_from(metadata)
  }
}

impl From<FileType> for FileMetadata {
  fn from(value: FileType) -> Self {
    FileMetadata {
      is_directory: value == FileType::Directory,
      is_file: value == FileType::File,
      is_symlink: false,
      // yarn pnp don'ts have following info
      atime_ms: 0,
      mtime_ms: 0,
      ctime_ms: 0,
      size: 0,
    }
  }
}

#[cfg(not(target_family = "wasm"))]
#[async_trait::async_trait]
impl ReadableFileSystem for NativeFileSystem {
  fn read(&self, path: &Utf8Path) -> Result<Vec<u8>> {
    if self.options.pnp {
      let path = path.as_std_path();
      let buffer = match VPath::from(path)? {
        VPath::Zip(info) => self.pnp_lru.read(info.physical_base_path(), info.zip_path),
        VPath::Virtual(info) => std::fs::read(info.physical_base_path()),
        VPath::Native(path) => std::fs::read(&path),
      };
      return buffer.map_err(Error::from);
    }
    fs::read(path).map_err(Error::from)
  }

  fn metadata(&self, path: &Utf8Path) -> Result<FileMetadata> {
    if self.options.pnp {
      let path = path.as_std_path();
      return match VPath::from(path)? {
        VPath::Zip(info) => self
          .pnp_lru
          .file_type(info.physical_base_path(), info.zip_path)
          .map(FileMetadata::from)
          .map_err(Error::from),

        VPath::Virtual(info) => {
          let meta = fs::metadata(info.physical_base_path())?;
          FileMetadata::try_from(meta)
        }
        VPath::Native(path) => {
          let meta = fs::metadata(path)?;
          FileMetadata::try_from(meta)
        }
      };
    }
    let meta = fs::metadata(path)?;
    meta.try_into()
  }

  fn symlink_metadata(&self, path: &Utf8Path) -> Result<FileMetadata> {
    let meta = fs::symlink_metadata(path)?;
    meta.try_into()
  }

  fn canonicalize(&self, path: &Utf8Path) -> Result<Utf8PathBuf> {
    if self.options.pnp {
      let path = path.as_std_path();
      let path = match VPath::from(path)? {
        VPath::Zip(info) => dunce::canonicalize(info.physical_base_path().join(info.zip_path)),
        VPath::Virtual(info) => dunce::canonicalize(info.physical_base_path()),
        VPath::Native(path) => dunce::canonicalize(path),
      };
      return path.map(|x| x.assert_utf8()).map_err(Error::from);
    }
    let path = dunce::canonicalize(path)?;
    Ok(path.assert_utf8())
  }

  async fn async_read(&self, file: &Utf8Path) -> Result<Vec<u8>> {
    tokio::fs::read(file).await.map_err(Error::from)
  }

  fn read_dir(&self, dir: &Utf8Path) -> Result<Vec<String>> {
    let mut res = vec![];
    for entry in fs::read_dir(dir)? {
      let entry = entry?;
      res.push(entry.file_name().to_string_lossy().to_string());
    }
    Ok(res)
  }
}

#[cfg(target_family = "wasm")]
#[async_trait::async_trait]
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

  async fn async_read(&self, file: &Utf8Path) -> Result<Vec<u8>> {
    fs::read(file).map_err(Error::from)
  }

  fn read_dir(&self, dir: &Utf8Path) -> Result<Vec<String>> {
    let mut res = vec![];
    for entry in fs::read_dir(dir)? {
      let entry = entry?;
      res.push(entry.file_name().to_string_lossy().to_string());
    }
    Ok(res)
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

impl IntermediateFileSystem for NativeFileSystem {}

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
  async fn read(&mut self, length: usize) -> Result<Vec<u8>> {
    let mut buf = vec![0u8; length];
    self.0.read_exact(&mut buf).map_err(Error::from)?;
    Ok(buf)
  }

  async fn read_until(&mut self, byte: u8) -> Result<Vec<u8>> {
    let mut buf = vec![];
    self.0.read_until(byte, &mut buf).map_err(Error::from)?;
    if buf.last().is_some_and(|b| b == &byte) {
      buf.pop();
    }
    Ok(buf)
  }
  async fn read_to_end(&mut self) -> Result<Vec<u8>> {
    let mut buf = vec![];
    self.0.read_to_end(&mut buf).map_err(Error::from)?;
    Ok(buf)
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
    let file = File::create_new(file).map_err(Error::from)?;
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
