use std::{
  fs::{self, File},
  io::{BufRead, BufReader, BufWriter, Read, Write},
  sync::Arc,
};

use pnp::fs::{FileType, LruZipCache, VPath, VPathInfo, ZipCache};
use rspack_paths::{AssertUtf8, Utf8Path, Utf8PathBuf};
use tokio::task::spawn_blocking;

use crate::{
  Error, FileMetadata, IntermediateFileSystem, IntermediateFileSystemExtras, IoResultToFsResultExt,
  ReadStream, ReadableFileSystem, Result, WritableFileSystem, WriteStream,
};
#[derive(Debug, Clone)]
struct NativeFileSystemOptions {
  // enable Yarn PnP feature
  pnp: bool,
}
#[derive(Debug, Clone)]
pub struct NativeFileSystem {
  options: NativeFileSystemOptions,
  pnp_lru: Arc<LruZipCache<Vec<u8>>>,
}
impl NativeFileSystem {
  pub fn new(pnp: bool) -> Self {
    Self {
      options: NativeFileSystemOptions { pnp },
      pnp_lru: Arc::new(LruZipCache::new(50, pnp::fs::open_zip_via_read_p)),
    }
  }
}

#[cfg(not(target_family = "wasm"))]
#[async_trait::async_trait]
impl WritableFileSystem for NativeFileSystem {
  async fn create_dir(&self, dir: &Utf8Path) -> Result<()> {
    fs::create_dir(dir).to_fs_result()
  }

  async fn create_dir_all(&self, dir: &Utf8Path) -> Result<()> {
    fs::create_dir_all(dir).to_fs_result()
  }

  async fn write(&self, file: &Utf8Path, data: &[u8]) -> Result<()> {
    fs::write(file, data).to_fs_result()
  }

  async fn remove_file(&self, file: &Utf8Path) -> Result<()> {
    tokio::fs::remove_file(file).await.to_fs_result()
  }

  async fn remove_dir_all(&self, dir: &Utf8Path) -> Result<()> {
    let dir = dir.to_path_buf();
    tokio::fs::remove_dir_all(dir).await.to_fs_result()
  }

  async fn read_dir(&self, dir: &Utf8Path) -> Result<Vec<String>> {
    let dir = dir.to_path_buf();
    let mut reader = tokio::fs::read_dir(dir).await.to_fs_result()?;
    let mut res = vec![];
    while let Some(entry) = reader.next_entry().await.to_fs_result()? {
      res.push(entry.file_name().to_string_lossy().to_string());
    }
    Ok(res)
  }

  async fn read_file(&self, file: &Utf8Path) -> Result<Vec<u8>> {
    tokio::fs::read(file).await.to_fs_result()
  }

  async fn stat(&self, file: &Utf8Path) -> Result<FileMetadata> {
    let metadata = tokio::fs::metadata(file).await.to_fs_result()?;
    FileMetadata::try_from(metadata)
  }
}

#[cfg(target_family = "wasm")]
#[async_trait::async_trait]
impl WritableFileSystem for NativeFileSystem {
  async fn create_dir(&self, dir: &Utf8Path) -> Result<()> {
    fs::create_dir(dir).to_fs_result()
  }

  async fn create_dir_all(&self, dir: &Utf8Path) -> Result<()> {
    fs::create_dir_all(dir).to_fs_result()
  }

  async fn write(&self, file: &Utf8Path, data: &[u8]) -> Result<()> {
    fs::write(file, data).to_fs_result()
  }

  async fn remove_file(&self, file: &Utf8Path) -> Result<()> {
    fs::remove_file(file).to_fs_result()
  }

  async fn remove_dir_all(&self, dir: &Utf8Path) -> Result<()> {
    let dir = dir.to_path_buf();
    fs::remove_dir_all(dir).to_fs_result()
  }

  async fn read_dir(&self, dir: &Utf8Path) -> Result<Vec<String>> {
    let dir = dir.to_path_buf();
    let mut res = vec![];
    let reader = fs::read_dir(dir).to_fs_result()?;
    for entry in reader {
      let entry = entry.to_fs_result()?;
      res.push(entry.file_name().to_string_lossy().to_string());
    }
    Ok(res)
  }

  async fn read_file(&self, file: &Utf8Path) -> Result<Vec<u8>> {
    fs::read(file).to_fs_result()
  }

  async fn stat(&self, file: &Utf8Path) -> Result<FileMetadata> {
    let metadata = fs::metadata(file).to_fs_result()?;
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
  async fn read(&self, path: &Utf8Path) -> Result<Vec<u8>> {
    let _self = self.clone();
    let path = path.to_owned();
    match spawn_blocking(move || _self.read_sync(&path)).await {
      Ok(res) => res,
      Err(_) => Err(Error::new(
        std::io::ErrorKind::Other,
        "spawn_blocking failed",
      )),
    }
  }

  fn read_sync(&self, path: &Utf8Path) -> Result<Vec<u8>> {
    if self.options.pnp {
      let path = path.as_std_path();
      let buffer = match VPath::from(path)? {
        VPath::Zip(info) => self.pnp_lru.read(info.physical_base_path(), info.zip_path),
        VPath::Virtual(info) => std::fs::read(info.physical_base_path()),
        VPath::Native(path) => std::fs::read(&path),
      };
      return buffer.to_fs_result();
    }
    fs::read(path).to_fs_result()
  }

  async fn metadata(&self, path: &Utf8Path) -> Result<FileMetadata> {
    let _self = self.clone();
    let path = path.to_owned();
    match spawn_blocking(move || _self.metadata_sync(&path)).await {
      Ok(res) => res,
      Err(_) => Err(Error::new(
        std::io::ErrorKind::Other,
        "spawn_blocking failed",
      )),
    }
  }

  fn metadata_sync(&self, path: &Utf8Path) -> Result<FileMetadata> {
    if self.options.pnp {
      let path = path.as_std_path();
      return match VPath::from(path)? {
        VPath::Zip(info) => self
          .pnp_lru
          .file_type(info.physical_base_path(), info.zip_path)
          .map(FileMetadata::from)
          .to_fs_result(),

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

  async fn symlink_metadata(&self, path: &Utf8Path) -> Result<FileMetadata> {
    let meta = tokio::fs::symlink_metadata(path).await.to_fs_result()?;
    meta.try_into()
  }

  async fn canonicalize(&self, path: &Utf8Path) -> Result<Utf8PathBuf> {
    let path = path.to_owned();
    let pnp = self.options.pnp;
    let sync_canonicalize = move || {
      if pnp {
        let path = path.as_std_path();
        let path = match VPath::from(path)? {
          VPath::Zip(info) => dunce::canonicalize(info.physical_base_path().join(info.zip_path)),
          VPath::Virtual(info) => dunce::canonicalize(info.physical_base_path()),
          VPath::Native(path) => dunce::canonicalize(path),
        };
        return path.map(|x| x.assert_utf8()).to_fs_result();
      }
      let path = dunce::canonicalize(path)?;
      Ok(path.assert_utf8())
    };

    match spawn_blocking(sync_canonicalize).await {
      Ok(res) => res,
      Err(_) => Err(Error::new(
        std::io::ErrorKind::Other,
        "spawn_blocking failed",
      )),
    }
  }

  async fn read_dir(&self, dir: &Utf8Path) -> Result<Vec<String>> {
    let mut res = vec![];
    let mut read_dir = tokio::fs::read_dir(dir).await.to_fs_result()?;
    while let Some(entry) = read_dir.next_entry().await? {
      res.push(entry.file_name().to_string_lossy().to_string());
    }
    Ok(res)
  }
  fn read_dir_sync(&self, dir: &Utf8Path) -> Result<Vec<String>> {
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
    fs::read(path).to_fs_result()
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
    // Comes from rspack_resolver
    use std::path::Component;
    let mut path_buf = path.to_path_buf();
    loop {
      let link = fs::read_link(&path_buf)?;
      path_buf.pop();
      for component in link.components() {
        match component {
          Component::ParentDir => {
            path_buf.pop();
          }
          Component::Normal(seg) => {
            path_buf.push(seg.to_string_lossy().trim_end_matches('\0'));
          }
          Component::RootDir => {
            path_buf = Utf8PathBuf::from("/");
          }
          Component::CurDir | Component::Prefix(_) => {}
        }
      }
      if !fs::symlink_metadata(&path_buf)?.is_symlink() {
        break;
      }
    }
    Ok(path_buf)
  }

  async fn async_read(&self, file: &Utf8Path) -> Result<Vec<u8>> {
    fs::read(file).to_fs_result()
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
    fs::rename(from, to).to_fs_result()
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
    let file = File::open(file).to_fs_result()?;
    Ok(Self(BufReader::new(file)))
  }
}

#[async_trait::async_trait]
impl ReadStream for NativeReadStream {
  async fn read(&mut self, length: usize) -> Result<Vec<u8>> {
    let mut buf = vec![0u8; length];
    self.0.read_exact(&mut buf).to_fs_result()?;
    Ok(buf)
  }

  async fn read_until(&mut self, byte: u8) -> Result<Vec<u8>> {
    let mut buf = vec![];
    self.0.read_until(byte, &mut buf).to_fs_result()?;
    if buf.last().is_some_and(|b| b == &byte) {
      buf.pop();
    }
    Ok(buf)
  }
  async fn read_to_end(&mut self) -> Result<Vec<u8>> {
    let mut buf = vec![];
    self.0.read_to_end(&mut buf).to_fs_result()?;
    Ok(buf)
  }
  async fn skip(&mut self, offset: usize) -> Result<()> {
    self.0.seek_relative(offset as i64).to_fs_result()
  }
  async fn close(&mut self) -> Result<()> {
    Ok(())
  }
}

#[derive(Debug)]
pub struct NativeWriteStream(BufWriter<File>);

impl NativeWriteStream {
  pub fn try_new(file: &Utf8Path) -> Result<Self> {
    let file = File::create_new(file).to_fs_result()?;
    Ok(Self(BufWriter::new(file)))
  }
}

#[async_trait::async_trait]
impl WriteStream for NativeWriteStream {
  async fn write(&mut self, buf: &[u8]) -> Result<usize> {
    self.0.write(buf).to_fs_result()
  }
  async fn write_all(&mut self, buf: &[u8]) -> Result<()> {
    self.0.write_all(buf).to_fs_result()
  }
  async fn flush(&mut self) -> Result<()> {
    self.0.flush().to_fs_result()
  }
  async fn close(&mut self) -> Result<()> {
    Ok(())
  }
}
