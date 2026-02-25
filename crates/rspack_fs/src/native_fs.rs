use std::{
  fs::{self, File},
  io::{BufRead, BufReader, BufWriter, Read, Write},
  path::{Path, PathBuf},
};

use pnp::fs::{FileType, LruZipCache, VPath, VPathInfo, ZipCache};
use rspack_paths::{AssertUtf8, Utf8Path, Utf8PathBuf};
use tracing::instrument;

use crate::{
  Error, FileMetadata, FilePermissions, IntermediateFileSystem, IntermediateFileSystemExtras,
  IoResultToFsResultExt, ReadStream, ReadableFileSystem, Result, WritableFileSystem, WriteStream,
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
  #[instrument(skip(self), level = "debug")]
  async fn create_dir(&self, dir: &Utf8Path) -> Result<()> {
    fs::create_dir(dir).to_fs_result()
  }
  #[instrument(skip(self), level = "debug")]
  async fn create_dir_all(&self, dir: &Utf8Path) -> Result<()> {
    fs::create_dir_all(dir).to_fs_result()
  }
  #[instrument(skip(self), level = "debug")]
  async fn write(&self, file: &Utf8Path, data: &[u8]) -> Result<()> {
    fs::write(file, data).to_fs_result()
  }
  #[instrument(skip(self), level = "debug")]
  async fn remove_file(&self, file: &Utf8Path) -> Result<()> {
    tokio::fs::remove_file(file).await.to_fs_result()
  }
  #[instrument(skip(self), level = "debug")]
  async fn remove_dir_all(&self, dir: &Utf8Path) -> Result<()> {
    let dir = dir.to_path_buf();
    tokio::fs::remove_dir_all(dir).await.to_fs_result()
  }
  #[instrument(skip(self), level = "debug")]
  async fn read_dir(&self, dir: &Utf8Path) -> Result<Vec<String>> {
    let dir = dir.to_path_buf();
    let mut reader = tokio::fs::read_dir(dir).await.to_fs_result()?;
    let mut res = vec![];
    while let Some(entry) = reader.next_entry().await.to_fs_result()? {
      res.push(entry.file_name().to_string_lossy().to_string());
    }
    Ok(res)
  }
  #[instrument(skip(self), level = "debug")]
  async fn read_file(&self, file: &Utf8Path) -> Result<Vec<u8>> {
    tokio::fs::read(file).await.to_fs_result()
  }
  #[instrument(skip(self), level = "debug")]
  async fn stat(&self, file: &Utf8Path) -> Result<FileMetadata> {
    let metadata = tokio::fs::metadata(file).await.to_fs_result()?;
    FileMetadata::try_from(metadata)
  }
  #[instrument(skip(self), level = "debug")]
  async fn set_permissions(&self, path: &Utf8Path, perm: FilePermissions) -> Result<()> {
    if let Some(perm) = perm.into_std() {
      return tokio::fs::set_permissions(path, perm).await.to_fs_result();
    }
    Ok(())
  }
}

#[cfg(target_family = "wasm")]
#[async_trait::async_trait]
impl WritableFileSystem for NativeFileSystem {
  #[instrument(skip(self), level = "debug")]
  async fn create_dir(&self, dir: &Utf8Path) -> Result<()> {
    fs::create_dir(dir).to_fs_result()
  }
  #[instrument(skip(self), level = "debug")]
  async fn create_dir_all(&self, dir: &Utf8Path) -> Result<()> {
    fs::create_dir_all(dir).to_fs_result()
  }
  #[instrument(skip(self), level = "debug")]
  async fn write(&self, file: &Utf8Path, data: &[u8]) -> Result<()> {
    fs::write(file, data).to_fs_result()
  }
  #[instrument(skip(self), level = "debug")]
  async fn remove_file(&self, file: &Utf8Path) -> Result<()> {
    fs::remove_file(file).to_fs_result()
  }
  #[instrument(skip(self), level = "debug")]
  async fn remove_dir_all(&self, dir: &Utf8Path) -> Result<()> {
    let dir = dir.to_path_buf();
    fs::remove_dir_all(dir).to_fs_result()
  }
  #[instrument(skip(self), level = "debug")]
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
  #[instrument(skip(self), level = "debug")]
  async fn read_file(&self, file: &Utf8Path) -> Result<Vec<u8>> {
    fs::read(file).to_fs_result()
  }
  #[instrument(skip(self), level = "debug")]
  async fn stat(&self, file: &Utf8Path) -> Result<FileMetadata> {
    let metadata = fs::metadata(file).to_fs_result()?;
    FileMetadata::try_from(metadata)
  }
  #[instrument(skip(self), level = "debug")]
  async fn set_permissions(&self, _path: &Utf8Path, perm: FilePermissions) -> Result<()> {
    Ok(())
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
  #[instrument(skip(self), level = "debug")]
  async fn read(&self, path: &Utf8Path) -> Result<Vec<u8>> {
    if self.options.pnp {
      let path = path.as_std_path();
      let buffer = match VPath::from(path)? {
        VPath::Zip(info) => self.pnp_lru.read(info.physical_base_path(), info.zip_path),
        VPath::Virtual(info) => fs::read(info.physical_base_path()),
        VPath::Native(path) => fs::read(&path),
      };
      return buffer.map_err(Error::from);
    }

    fs::read(path).map_err(Error::from)
  }
  #[instrument(skip(self), level = "debug")]
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
  #[instrument(skip(self), level = "debug")]
  async fn metadata(&self, path: &Utf8Path) -> Result<FileMetadata> {
    self.metadata_sync(path)
  }
  #[instrument(skip(self), level = "debug")]
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
  #[instrument(skip(self), level = "debug")]
  async fn symlink_metadata(&self, path: &Utf8Path) -> Result<FileMetadata> {
    let meta = fs::symlink_metadata(path)?;
    meta.try_into()
  }
  #[instrument(skip(self), level = "debug")]
  async fn canonicalize(&self, path: &Utf8Path) -> Result<Utf8PathBuf> {
    if self.options.pnp {
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
  }
  #[instrument(skip(self), level = "debug")]
  async fn read_dir(&self, dir: &Utf8Path) -> Result<Vec<String>> {
    self.read_dir_sync(dir)
  }
  #[instrument(skip(self), level = "debug")]
  fn read_dir_sync(&self, dir: &Utf8Path) -> Result<Vec<String>> {
    let mut res = vec![];
    let dir = if self.options.pnp {
      let path = dir.as_std_path();
      match VPath::from(path)? {
        VPath::Zip(info) => {
          self.pnp_lru.act(info.physical_base_path(), |zip| {
            for path in zip.dirs.iter().chain(zip.files.keys()) {
              let pathbuf = PathBuf::from(path);
              if let Some(file_name) = pathbuf.file_name() {
                let parent_path = pathbuf.parent().unwrap_or_else(|| Path::new("."));
                if Path::new(&info.zip_path) == parent_path {
                  res.push(file_name.to_string_lossy().to_string());
                }
              }
            }
          })?;

          return Ok(res);
        }
        VPath::Virtual(info) => info.physical_base_path(),
        VPath::Native(path) => path,
      }
    } else {
      dir.into()
    };

    for entry in fs::read_dir(dir)? {
      let entry = entry?;
      res.push(entry.file_name().to_string_lossy().to_string());
    }

    Ok(res)
  }
  #[instrument(skip(self), level = "debug")]
  async fn permissions(&self, path: &Utf8Path) -> Result<Option<FilePermissions>> {
    let meta = tokio::fs::metadata(path).await.to_fs_result()?;
    Ok(Some(FilePermissions::from_std(meta.permissions())))
  }
}

#[cfg(target_family = "wasm")]
#[async_trait::async_trait]
impl ReadableFileSystem for NativeFileSystem {
  #[instrument(skip(self), level = "debug")]
  async fn read(&self, path: &Utf8Path) -> Result<Vec<u8>> {
    self.read_sync(path)
  }
  #[instrument(skip(self), level = "debug")]
  fn read_sync(&self, path: &Utf8Path) -> Result<Vec<u8>> {
    fs::read(path).to_fs_result()
  }
  #[instrument(skip(self), level = "debug")]
  async fn metadata(&self, path: &Utf8Path) -> Result<FileMetadata> {
    let meta = fs::metadata(path)?;
    meta.try_into()
  }
  #[instrument(skip(self), level = "debug")]
  fn metadata_sync(&self, path: &Utf8Path) -> Result<FileMetadata> {
    let meta = fs::metadata(path)?;
    meta.try_into()
  }
  #[instrument(skip(self), level = "debug")]
  async fn symlink_metadata(&self, path: &Utf8Path) -> Result<FileMetadata> {
    let meta = fs::symlink_metadata(path)?;
    meta.try_into()
  }
  #[instrument(skip(self), level = "debug")]
  async fn canonicalize(&self, path: &Utf8Path) -> Result<Utf8PathBuf> {
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
  #[instrument(skip(self), level = "debug")]
  async fn read_dir(&self, dir: &Utf8Path) -> Result<Vec<String>> {
    self.read_dir_sync(dir)
  }
  #[instrument(skip(self), level = "debug")]
  fn read_dir_sync(&self, dir: &Utf8Path) -> Result<Vec<String>> {
    let mut res = vec![];
    for entry in fs::read_dir(dir)? {
      let entry = entry?;
      res.push(entry.file_name().to_string_lossy().to_string());
    }
    Ok(res)
  }
  #[instrument(skip(self), level = "debug")]
  async fn permissions(&self, path: &Utf8Path) -> Result<Option<FilePermissions>> {
    Ok(None)
  }
}

#[async_trait::async_trait]
impl IntermediateFileSystemExtras for NativeFileSystem {
  #[instrument(skip(self), level = "debug")]
  async fn rename(&self, from: &Utf8Path, to: &Utf8Path) -> Result<()> {
    fs::rename(from, to).to_fs_result()
  }
  #[instrument(skip(self), level = "debug")]
  async fn create_read_stream(&self, file: &Utf8Path) -> Result<Box<dyn ReadStream>> {
    let reader = NativeReadStream::try_new(file)?;
    Ok(Box::new(reader))
  }
  #[instrument(skip(self), level = "debug")]
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
  #[instrument(skip(self), level = "debug")]
  async fn read(&mut self, length: usize) -> Result<Vec<u8>> {
    let mut buf = vec![0u8; length];
    self.0.read_exact(&mut buf).to_fs_result()?;
    Ok(buf)
  }
  #[instrument(skip(self), level = "debug")]
  async fn read_until(&mut self, byte: u8) -> Result<Vec<u8>> {
    let mut buf = vec![];
    self.0.read_until(byte, &mut buf).to_fs_result()?;
    if buf.last().is_some_and(|b| b == &byte) {
      buf.pop();
    }
    Ok(buf)
  }
  #[instrument(skip(self), level = "debug")]
  async fn read_to_end(&mut self) -> Result<Vec<u8>> {
    let mut buf = vec![];
    self.0.read_to_end(&mut buf).to_fs_result()?;
    Ok(buf)
  }
  #[instrument(skip(self), level = "debug")]
  async fn skip(&mut self, offset: usize) -> Result<()> {
    self.0.seek_relative(offset as i64).to_fs_result()
  }
  #[instrument(skip(self), level = "debug")]
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
  #[instrument(skip(self), level = "debug")]
  async fn write(&mut self, buf: &[u8]) -> Result<usize> {
    self.0.write(buf).to_fs_result()
  }
  #[instrument(skip(self), level = "debug")]
  async fn write_all(&mut self, buf: &[u8]) -> Result<()> {
    self.0.write_all(buf).to_fs_result()
  }
  #[instrument(skip(self), level = "debug")]
  async fn flush(&mut self) -> Result<()> {
    self.0.flush().to_fs_result()
  }
  #[instrument(skip(self), level = "debug")]
  async fn close(&mut self) -> Result<()> {
    Ok(())
  }
}
