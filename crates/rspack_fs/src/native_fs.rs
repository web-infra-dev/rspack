#[cfg(not(target_family = "wasm"))]
use std::{fmt, io, sync::Arc};
use std::{
  fs::{self, File},
  io::{BufRead, BufReader, BufWriter, Read, Write},
  path::{Path, PathBuf},
};

#[cfg(not(target_family = "wasm"))]
use cow_utils::CowUtils;
#[cfg(not(target_family = "wasm"))]
use once_cell::sync::OnceCell;
#[cfg(not(target_family = "wasm"))]
use parking_lot::Mutex;
use pnp::fs::{FileType, LruZipCache, VPath, VPathInfo, ZipCache};
use rspack_paths::{AssertUtf8, Utf8Path, Utf8PathBuf};
#[cfg(not(target_family = "wasm"))]
use rspack_util::{fx_hash::FxHashMap, node_path::NodePath};
#[cfg(not(target_family = "wasm"))]
use tokio::fs as tokio_fs;
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
  #[cfg(not(target_family = "wasm"))]
  pnp_zip_attributes: ZipAttributesCache,
}
impl NativeFileSystem {
  pub fn new(pnp: bool) -> Self {
    Self {
      options: NativeFileSystemOptions { pnp },
      pnp_lru: LruZipCache::new(50, pnp::fs::open_zip_via_read_p),
      #[cfg(not(target_family = "wasm"))]
      pnp_zip_attributes: ZipAttributesCache::default(),
    }
  }

  #[cfg(not(target_family = "wasm"))]
  fn read_metadata(&self, path: &Utf8Path, include_zip_attributes: bool) -> Result<FileMetadata> {
    if self.options.pnp {
      match VPath::from(path.as_std_path())? {
        VPath::Zip(info) => {
          let archive = info.physical_base_path();
          let mut metadata = FileMetadata::from(self.pnp_lru.file_type(&archive, &info.zip_path)?);
          if include_zip_attributes
            && metadata.is_file
            && let Some(attributes) = self.pnp_zip_attributes.get(&archive, &info.zip_path)?
          {
            metadata.mode = cfg!(unix).then_some(attributes.mode);
            // Yarn's ZipFS reports the entry's modification time for both fields.
            if let Some(mtime) = attributes.mtime_ms {
              metadata.atime_ms = Some(mtime);
              metadata.mtime_ms = mtime;
            }
          }
          return Ok(metadata);
        }
        VPath::Virtual(info) => return fs::metadata(info.physical_base_path())?.try_into(),
        VPath::Native(path) => return fs::metadata(path)?.try_into(),
      }
    }
    fs::metadata(path)?.try_into()
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
    tokio_fs::remove_file(file).await.to_fs_result()
  }
  #[instrument(skip(self), level = "debug")]
  async fn remove_dir_all(&self, dir: &Utf8Path) -> Result<()> {
    let dir = dir.to_path_buf();
    tokio_fs::remove_dir_all(dir).await.to_fs_result()
  }
  #[instrument(skip(self), level = "debug")]
  async fn read_dir(&self, dir: &Utf8Path) -> Result<Vec<String>> {
    let dir = dir.to_path_buf();
    let mut reader = tokio_fs::read_dir(dir).await.to_fs_result()?;
    let mut res = vec![];
    while let Some(entry) = reader.next_entry().await.to_fs_result()? {
      res.push(entry.file_name().to_string_lossy().to_string());
    }
    Ok(res)
  }
  #[instrument(skip(self), level = "debug")]
  async fn read_file(&self, file: &Utf8Path) -> Result<Vec<u8>> {
    tokio_fs::read(file).await.to_fs_result()
  }
  #[instrument(skip(self), level = "debug")]
  async fn stat(&self, file: &Utf8Path) -> Result<FileMetadata> {
    let metadata = tokio_fs::metadata(file).await.to_fs_result()?;
    FileMetadata::try_from(metadata)
  }
  #[instrument(skip(self), level = "debug")]
  async fn set_permissions(&self, path: &Utf8Path, perm: FilePermissions) -> Result<()> {
    if let Some(perm) = perm.into_std() {
      return tokio_fs::set_permissions(path, perm).await.to_fs_result();
    }
    Ok(())
  }

  #[instrument(skip(self), level = "debug")]
  async fn set_times(&self, path: &Utf8Path, atime_ms: u64, mtime_ms: u64) -> Result<()> {
    let to_file_time = |ms: u64| {
      filetime::FileTime::from_unix_time((ms / 1000) as i64, ((ms % 1000) * 1_000_000) as u32)
    };
    filetime::set_file_times(path, to_file_time(atime_ms), to_file_time(mtime_ms)).to_fs_result()
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
      mode: None,
      // The PnP reader only reports the file type.
      atime_ms: None,
      mtime_ms: 0,
      ctime_ms: 0,
      size: 0,
    }
  }
}

#[cfg(not(target_family = "wasm"))]
#[derive(Clone, Copy)]
struct ZipFileAttributes {
  mode: u16,
  mtime_ms: Option<u64>,
}

#[cfg(not(target_family = "wasm"))]
type ZipAttributes = FxHashMap<String, ZipFileAttributes>;

#[cfg(not(target_family = "wasm"))]
#[derive(Default)]
struct ZipAttributesCache(Mutex<Vec<(PathBuf, Arc<OnceCell<ZipAttributes>>)>>);

#[cfg(not(target_family = "wasm"))]
impl fmt::Debug for ZipAttributesCache {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("ZipAttributesCache").finish_non_exhaustive()
  }
}

#[cfg(not(target_family = "wasm"))]
impl ZipAttributesCache {
  fn get(&self, path: &Path, entry: &str) -> io::Result<Option<ZipFileAttributes>> {
    const CAPACITY: usize = 50;
    let attributes = {
      let mut archives = self.0.lock();
      let cached = match archives.iter().position(|(archive, _)| archive == path) {
        Some(index) => archives.remove(index),
        None => {
          if archives.len() == CAPACITY {
            archives.remove(0);
          }
          (path.to_owned(), Arc::new(OnceCell::new()))
        }
      };
      let attributes = Arc::clone(&cached.1);
      archives.push(cached);
      attributes
    };

    // Hold the cache lock only for the lookup, and share the archive read with
    // concurrent callers copying files from the same archive.
    let attributes = attributes.get_or_try_init(|| read_zip_file_attributes(path))?;
    Ok(attributes.get(entry).copied())
  }
}

#[cfg(not(target_family = "wasm"))]
fn zip_attributes_error(error: rawzip::Error) -> io::Error {
  match error.into_kind() {
    rawzip::ErrorKind::IO(error) => error,
    _ => io::Error::new(
      io::ErrorKind::InvalidData,
      "could not read ZIP file attributes",
    ),
  }
}

#[cfg(not(target_family = "wasm"))]
fn read_zip_file_attributes(path: &Path) -> io::Result<ZipAttributes> {
  let mut buffer = vec![0; rawzip::MAX_CENTRAL_DIRECTORY_RECORD_SIZE];
  let archive =
    rawzip::ZipArchive::from_file(File::open(path)?, &mut buffer).map_err(zip_attributes_error)?;
  let mut records = archive.entries(&mut buffer);
  let mut attributes = FxHashMap::default();
  while let Some(entry) = records.next_entry().map_err(zip_attributes_error)? {
    let entry_path = entry.file_path();
    let Ok(name) = std::str::from_utf8(entry_path.as_ref()) else {
      continue;
    };
    if name.ends_with('/') {
      continue;
    }
    let name = name.cow_replace('\\', "/");
    let name = Utf8Path::new(&name).node_normalize_posix().into_string();

    // Match Yarn's ZipFS, which uses the DOS modification time and Unix mode.
    let dos_time = entry.last_modified_dos();
    let mtime_ms = rawzip::time::UtcDateTime::from_components(
      dos_time.year(),
      dos_time.month(),
      dos_time.day(),
      dos_time.hour(),
      dos_time.minute(),
      dos_time.second(),
      0,
    )
    .and_then(|time| u64::try_from(time.to_unix()).ok())
    .map(|time| time * 1000);
    let mode = if entry.version_made_by().creator_system() == rawzip::CreatorSystem::UNIX {
      ((entry.external_attributes() >> 16) & 0o777) as u16
    } else {
      0o644
    };
    attributes.insert(name, ZipFileAttributes { mode, mtime_ms });
  }
  Ok(attributes)
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
        VPath::Virtual(info) => fs::read(info.physical_base_path()),
        VPath::Native(path) => fs::read(&path),
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
    self.read_metadata(path, false)
  }

  async fn metadata_with_file_attributes(&self, path: &Utf8Path) -> Result<FileMetadata> {
    self.read_metadata(path, true)
  }

  #[instrument(skip(self), level = "debug")]
  async fn symlink_metadata(&self, path: &Utf8Path) -> Result<FileMetadata> {
    let meta = fs::symlink_metadata(path)?;
    meta.try_into()
  }
  #[instrument(skip(self), level = "debug")]
  async fn read_link(&self, path: &Utf8Path) -> Result<Utf8PathBuf> {
    Ok(fs::read_link(path)?.assert_utf8())
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
    let meta = tokio_fs::metadata(path).await.to_fs_result()?;
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
  async fn read_link(&self, path: &Utf8Path) -> Result<Utf8PathBuf> {
    Ok(fs::read_link(path)?.assert_utf8())
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
    fs::rename(from, to).map_err(|error| {
      Error::Io(std::io::Error::new(
        error.kind(),
        format!("failed to rename file from `{from}` to `{to}`: {error}"),
      ))
    })
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

#[cfg(all(test, not(target_family = "wasm")))]
mod tests {
  use rspack_paths::AssertUtf8;

  use super::{IntermediateFileSystemExtras, NativeFileSystem};

  #[tokio::test]
  async fn test_rename_error_includes_source_and_destination_paths() {
    let temp = tempfile::tempdir().expect("should create temp directory");
    let source = temp.path().join("source").assert_utf8();
    let destination = temp.path().join("destination").assert_utf8();
    std::fs::create_dir(&source).expect("should create source directory");
    std::fs::write(source.join("content"), "source")
      .expect("should make source directory non-empty");
    std::fs::create_dir(&destination).expect("should create destination directory");
    std::fs::write(destination.join("blocker"), "blocker")
      .expect("should make destination directory non-empty");

    let error = NativeFileSystem::new(false)
      .rename(&source, &destination)
      .await
      .expect_err("rename should fail when destination is a non-empty directory");
    let message = error.to_string();

    assert!(message.contains(source.as_str()));
    assert!(message.contains(destination.as_str()));
  }
}
