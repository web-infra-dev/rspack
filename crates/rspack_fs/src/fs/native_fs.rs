use std::{
  fs::{self, File},
  io::{BufRead, BufReader, BufWriter, Read, Write},
  path::{Path, PathBuf},
};

use pnp::fs::{FileType, LruZipCache, VPath, VPathInfo, ZipCache};
#[cfg(not(target_family = "wasm"))]
use tokio::fs as tokio_fs;
use tracing::instrument;

use super::{
  FileMetadata, FilePermissions, IntermediateFileSystem, IntermediateFileSystemExtras, ReadStream,
  ReadableFileSystem, WritableFileSystem, WriteStream,
};
use crate::{AbsPath, Error, IoResultToFsResultExt, Result};

#[derive(Debug)]
struct NativeFileSystemOptions {
  pnp: bool,
}

/// Native OS read/write/intermediate filesystem operating strictly on UTF-8 [`AbsPath`]s.
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

impl From<FileType> for FileMetadata {
  fn from(value: FileType) -> Self {
    Self {
      is_directory: value == FileType::Directory,
      is_file: value == FileType::File,
      is_symlink: false,
      // Yarn PnP does not expose timestamps or file sizes.
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
  async fn read(&self, path: &AbsPath) -> Result<Vec<u8>> {
    self.read_sync(path)
  }

  #[instrument(skip(self), level = "debug")]
  fn read_sync(&self, path: &AbsPath) -> Result<Vec<u8>> {
    let std_path = path.as_std_path();
    if self.options.pnp {
      let buffer = match VPath::from(std_path)? {
        VPath::Zip(info) => self.pnp_lru.read(info.physical_base_path(), info.zip_path),
        VPath::Virtual(info) => fs::read(info.physical_base_path()),
        VPath::Native(native_path) => fs::read(native_path),
      };
      return buffer.to_fs_result();
    }
    fs::read(std_path).to_fs_result()
  }

  #[instrument(skip(self), level = "debug")]
  async fn metadata(&self, path: &AbsPath) -> Result<FileMetadata> {
    self.metadata_sync(path)
  }

  #[instrument(skip(self), level = "debug")]
  fn metadata_sync(&self, path: &AbsPath) -> Result<FileMetadata> {
    let std_path = path.as_std_path();
    if self.options.pnp {
      return match VPath::from(std_path)? {
        VPath::Zip(info) => self
          .pnp_lru
          .file_type(info.physical_base_path(), info.zip_path)
          .map(FileMetadata::from)
          .to_fs_result(),
        VPath::Virtual(info) => fs::metadata(info.physical_base_path())?.try_into(),
        VPath::Native(native_path) => fs::metadata(native_path)?.try_into(),
      };
    }
    fs::metadata(std_path)?.try_into()
  }

  #[instrument(skip(self), level = "debug")]
  async fn symlink_metadata(&self, path: &AbsPath) -> Result<FileMetadata> {
    fs::symlink_metadata(path.as_std_path())?.try_into()
  }

  #[instrument(skip(self), level = "debug")]
  async fn read_link(&self, path: &AbsPath) -> Result<String> {
    fs::read_link(path.as_std_path())?
      .into_os_string()
      .into_string()
      .map_err(|os| Error::NonUtf8Path(PathBuf::from(os)))
  }

  #[instrument(skip(self), level = "debug")]
  async fn canonicalize(&self, path: &AbsPath) -> Result<AbsPath> {
    self.canonicalize_sync(path)
  }

  #[instrument(skip(self), level = "debug")]
  fn canonicalize_sync(&self, path: &AbsPath) -> Result<AbsPath> {
    let std_path = path.as_std_path();
    let canon = if self.options.pnp {
      match VPath::from(std_path)? {
        VPath::Zip(info) => dunce::canonicalize(info.physical_base_path().join(info.zip_path))?,
        VPath::Virtual(info) => dunce::canonicalize(info.physical_base_path())?,
        VPath::Native(native_path) => dunce::canonicalize(native_path)?,
      }
    } else {
      dunce::canonicalize(std_path)?
    };
    AbsPath::try_from_std_path(&canon)
  }

  #[instrument(skip(self), level = "debug")]
  async fn read_dir(&self, dir: &AbsPath) -> Result<Vec<String>> {
    self.read_dir_sync(dir)
  }

  #[instrument(skip(self), level = "debug")]
  fn read_dir_sync(&self, dir: &AbsPath) -> Result<Vec<String>> {
    let mut res = vec![];
    let std_dir = dir.as_std_path();
    let dir_path = if self.options.pnp {
      match VPath::from(std_dir)? {
        VPath::Zip(info) => {
          let mut non_utf8 = None;
          self.pnp_lru.act(info.physical_base_path(), |zip| {
            for entry in zip.dirs.iter().chain(zip.files.keys()) {
              let pathbuf = PathBuf::from(entry);
              if let Some(file_name) = pathbuf.file_name() {
                let parent_path = pathbuf.parent().unwrap_or_else(|| Path::new("."));
                if Path::new(&info.zip_path) == parent_path {
                  match file_name.to_str() {
                    Some(name) => res.push(name.to_owned()),
                    None => {
                      non_utf8 = Some(std_dir.join(file_name));
                      break;
                    }
                  }
                }
              }
            }
          })?;
          if let Some(bad) = non_utf8 {
            return Err(Error::NonUtf8Path(bad));
          }
          return Ok(res);
        }
        VPath::Virtual(info) => info.physical_base_path(),
        VPath::Native(native_path) => native_path,
      }
    } else {
      std_dir.to_path_buf()
    };

    for entry in fs::read_dir(&dir_path)? {
      let entry = entry?;
      let name = entry
        .file_name()
        .into_string()
        .map_err(|os| Error::NonUtf8Path(dir_path.join(os)))?;
      res.push(name);
    }

    Ok(res)
  }

  #[instrument(skip(self), level = "debug")]
  async fn permissions(&self, path: &AbsPath) -> Result<Option<FilePermissions>> {
    let meta = fs::metadata(path.as_std_path())?;
    Ok(Some(FilePermissions::from_std(meta.permissions())))
  }
}

#[cfg(target_family = "wasm")]
#[async_trait::async_trait]
impl ReadableFileSystem for NativeFileSystem {
  #[instrument(skip(self), level = "debug")]
  async fn read(&self, path: &AbsPath) -> Result<Vec<u8>> {
    self.read_sync(path)
  }

  #[instrument(skip(self), level = "debug")]
  fn read_sync(&self, path: &AbsPath) -> Result<Vec<u8>> {
    fs::read(path.as_std_path()).to_fs_result()
  }

  #[instrument(skip(self), level = "debug")]
  async fn metadata(&self, path: &AbsPath) -> Result<FileMetadata> {
    self.metadata_sync(path)
  }

  #[instrument(skip(self), level = "debug")]
  fn metadata_sync(&self, path: &AbsPath) -> Result<FileMetadata> {
    fs::metadata(path.as_std_path())?.try_into()
  }

  #[instrument(skip(self), level = "debug")]
  async fn symlink_metadata(&self, path: &AbsPath) -> Result<FileMetadata> {
    fs::symlink_metadata(path.as_std_path())?.try_into()
  }

  #[instrument(skip(self), level = "debug")]
  async fn read_link(&self, path: &AbsPath) -> Result<String> {
    fs::read_link(path.as_std_path())?
      .into_os_string()
      .into_string()
      .map_err(|os| Error::NonUtf8Path(PathBuf::from(os)))
  }

  #[instrument(skip(self), level = "debug")]
  async fn canonicalize(&self, path: &AbsPath) -> Result<AbsPath> {
    self.canonicalize_sync(path)
  }

  #[instrument(skip(self), level = "debug")]
  fn canonicalize_sync(&self, path: &AbsPath) -> Result<AbsPath> {
    use std::path::Component;
    let mut path_buf = path.as_std_path().to_path_buf();
    while fs::symlink_metadata(&path_buf)?.is_symlink() {
      let link = fs::read_link(&path_buf)?;
      if link.to_str().is_none() {
        return Err(Error::NonUtf8Path(link));
      }
      path_buf.pop();
      for component in link.components() {
        match component {
          Component::ParentDir => {
            path_buf.pop();
          }
          Component::Normal(seg) => {
            let seg = seg
              .to_str()
              .ok_or_else(|| Error::NonUtf8Path(path_buf.join(seg)))?;
            path_buf.push(seg.trim_end_matches('\0'));
          }
          Component::RootDir => {
            path_buf = PathBuf::from("/");
          }
          Component::CurDir | Component::Prefix(_) => {}
        }
      }
    }
    AbsPath::try_from_std_path(&path_buf)
  }

  #[instrument(skip(self), level = "debug")]
  async fn read_dir(&self, dir: &AbsPath) -> Result<Vec<String>> {
    self.read_dir_sync(dir)
  }

  #[instrument(skip(self), level = "debug")]
  fn read_dir_sync(&self, dir: &AbsPath) -> Result<Vec<String>> {
    let std_dir = dir.as_std_path();
    let mut res = vec![];
    for entry in fs::read_dir(std_dir)? {
      let entry = entry?;
      let name = entry
        .file_name()
        .into_string()
        .map_err(|os| Error::NonUtf8Path(std_dir.join(os)))?;
      res.push(name);
    }
    Ok(res)
  }

  #[instrument(skip(self), level = "debug")]
  async fn permissions(&self, _path: &AbsPath) -> Result<Option<FilePermissions>> {
    Ok(None)
  }
}

#[cfg(not(target_family = "wasm"))]
#[async_trait::async_trait]
impl WritableFileSystem for NativeFileSystem {
  #[instrument(skip(self), level = "debug")]
  async fn create_dir(&self, dir: &AbsPath) -> Result<()> {
    fs::create_dir(dir.as_std_path()).to_fs_result()
  }

  #[instrument(skip(self), level = "debug")]
  async fn create_dir_all(&self, dir: &AbsPath) -> Result<()> {
    fs::create_dir_all(dir.as_std_path()).to_fs_result()
  }

  #[instrument(skip(self), level = "debug")]
  async fn write(&self, file: &AbsPath, data: &[u8]) -> Result<()> {
    fs::write(file.as_std_path(), data).to_fs_result()
  }

  #[instrument(skip(self), level = "debug")]
  async fn remove_file(&self, file: &AbsPath) -> Result<()> {
    tokio_fs::remove_file(file.as_std_path())
      .await
      .to_fs_result()
  }

  #[instrument(skip(self), level = "debug")]
  async fn remove_dir_all(&self, dir: &AbsPath) -> Result<()> {
    tokio_fs::remove_dir_all(dir.as_std_path())
      .await
      .to_fs_result()
  }

  #[instrument(skip(self), level = "debug")]
  async fn read_dir(&self, dir: &AbsPath) -> Result<Vec<String>> {
    let std_dir = dir.as_std_path();
    let mut reader = tokio_fs::read_dir(std_dir).await.to_fs_result()?;
    let mut res = vec![];
    while let Some(entry) = reader.next_entry().await.to_fs_result()? {
      let name = entry
        .file_name()
        .into_string()
        .map_err(|os| Error::NonUtf8Path(std_dir.join(os)))?;
      res.push(name);
    }
    Ok(res)
  }

  #[instrument(skip(self), level = "debug")]
  async fn read_file(&self, file: &AbsPath) -> Result<Vec<u8>> {
    tokio_fs::read(file.as_std_path()).await.to_fs_result()
  }

  #[instrument(skip(self), level = "debug")]
  async fn stat(&self, file: &AbsPath) -> Result<FileMetadata> {
    let metadata = tokio_fs::metadata(file.as_std_path())
      .await
      .to_fs_result()?;
    FileMetadata::try_from(metadata)
  }

  #[instrument(skip(self), level = "debug")]
  async fn set_permissions(&self, path: &AbsPath, perm: FilePermissions) -> Result<()> {
    if let Some(perm) = perm.into_std() {
      return tokio_fs::set_permissions(path.as_std_path(), perm)
        .await
        .to_fs_result();
    }
    Ok(())
  }
}

#[cfg(target_family = "wasm")]
#[async_trait::async_trait]
impl WritableFileSystem for NativeFileSystem {
  #[instrument(skip(self), level = "debug")]
  async fn create_dir(&self, dir: &AbsPath) -> Result<()> {
    fs::create_dir(dir.as_std_path()).to_fs_result()
  }

  #[instrument(skip(self), level = "debug")]
  async fn create_dir_all(&self, dir: &AbsPath) -> Result<()> {
    fs::create_dir_all(dir.as_std_path()).to_fs_result()
  }

  #[instrument(skip(self), level = "debug")]
  async fn write(&self, file: &AbsPath, data: &[u8]) -> Result<()> {
    fs::write(file.as_std_path(), data).to_fs_result()
  }

  #[instrument(skip(self), level = "debug")]
  async fn remove_file(&self, file: &AbsPath) -> Result<()> {
    fs::remove_file(file.as_std_path()).to_fs_result()
  }

  #[instrument(skip(self), level = "debug")]
  async fn remove_dir_all(&self, dir: &AbsPath) -> Result<()> {
    fs::remove_dir_all(dir.as_std_path()).to_fs_result()
  }

  #[instrument(skip(self), level = "debug")]
  async fn read_dir(&self, dir: &AbsPath) -> Result<Vec<String>> {
    let std_dir = dir.as_std_path();
    let mut res = vec![];
    for entry in fs::read_dir(std_dir)? {
      let entry = entry?;
      let name = entry
        .file_name()
        .into_string()
        .map_err(|os| Error::NonUtf8Path(std_dir.join(os)))?;
      res.push(name);
    }
    Ok(res)
  }

  #[instrument(skip(self), level = "debug")]
  async fn read_file(&self, file: &AbsPath) -> Result<Vec<u8>> {
    fs::read(file.as_std_path()).to_fs_result()
  }

  #[instrument(skip(self), level = "debug")]
  async fn stat(&self, file: &AbsPath) -> Result<FileMetadata> {
    let metadata = fs::metadata(file.as_std_path())?;
    FileMetadata::try_from(metadata)
  }

  #[instrument(skip(self), level = "debug")]
  async fn set_permissions(&self, _path: &AbsPath, _perm: FilePermissions) -> Result<()> {
    Ok(())
  }
}

#[async_trait::async_trait]
impl IntermediateFileSystemExtras for NativeFileSystem {
  #[instrument(skip(self), level = "debug")]
  async fn rename(&self, from: &AbsPath, to: &AbsPath) -> Result<()> {
    fs::rename(from.as_std_path(), to.as_std_path()).map_err(|error| {
      Error::Io(std::io::Error::new(
        error.kind(),
        format!("failed to rename file from `{from}` to `{to}`: {error}"),
      ))
    })
  }

  #[instrument(skip(self), level = "debug")]
  async fn create_read_stream(&self, file: &AbsPath) -> Result<Box<dyn ReadStream>> {
    let reader = NativeReadStream::try_new(file)?;
    Ok(Box::new(reader))
  }

  #[instrument(skip(self), level = "debug")]
  async fn create_write_stream(&self, file: &AbsPath) -> Result<Box<dyn WriteStream>> {
    let writer = NativeWriteStream::try_new(file)?;
    Ok(Box::new(writer))
  }
}

impl IntermediateFileSystem for NativeFileSystem {}

#[derive(Debug)]
pub struct NativeReadStream(BufReader<File>);

impl NativeReadStream {
  pub fn try_new(file: &AbsPath) -> Result<Self> {
    let file = File::open(file.as_std_path()).to_fs_result()?;
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
    if buf.last().is_some_and(|b| *b == byte) {
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
  pub fn try_new(file: &AbsPath) -> Result<Self> {
    let file = File::create_new(file.as_std_path()).to_fs_result()?;
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
    self.0.flush().to_fs_result()
  }
}
