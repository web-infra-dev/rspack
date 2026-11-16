use std::{
  borrow::Cow,
  io::{BufRead, Cursor, ErrorKind, Read, Seek, Write},
  sync::{Arc, Mutex},
  time::{SystemTime, UNIX_EPOCH},
};

use rspack_paths::{AssertUtf8, Utf8Component, Utf8Path, Utf8PathBuf, Utf8Prefix};
use vfs::{FileSystem, MemoryFS, VfsError, VfsFileType, error::VfsErrorKind};

use crate::{
  Error, FileMetadata, IntermediateFileSystem, IntermediateFileSystemExtras, IoResultToFsResultExt,
  ReadStream, ReadableFileSystem, Result, WritableFileSystem, WriteStream,
  file_metadata::FilePermissions,
};

fn vfs_error(path: &Utf8Path, error: VfsError) -> Error {
  let kind = match error.kind() {
    VfsErrorKind::FileNotFound => ErrorKind::NotFound,
    VfsErrorKind::DirectoryExists | VfsErrorKind::FileExists => ErrorKind::AlreadyExists,
    VfsErrorKind::InvalidPath => ErrorKind::InvalidInput,
    VfsErrorKind::NotSupported => ErrorKind::Unsupported,
    VfsErrorKind::IoError(error) => error.kind(),
    _ => ErrorKind::Other,
  };
  Error::Io(std::io::Error::new(
    kind,
    format!("{path}: {}", error.kind()),
  ))
}

// Keep native roots separate, including Windows drives/UNC roots and relative paths.
// The backend's own empty root is private, so public roots can be created and removed.
fn vfs_path(path: &Utf8Path) -> String {
  let root = path.ancestors().last().expect("path has an ancestor");
  let mut result = String::from("/root");
  for component in root.components() {
    // Prefix equality normalizes drive letters and UNC separators. Use the parsed
    // prefix so equivalent native paths address the same backend entry.
    let component = match component {
      Utf8Component::Prefix(prefix) => Cow::Owned(match prefix.kind() {
        Utf8Prefix::Disk(drive) => format!("{}:", char::from(drive)),
        Utf8Prefix::VerbatimDisk(drive) => format!("\\\\?\\{}:", char::from(drive)),
        Utf8Prefix::UNC(server, share) => format!("\\\\{server}\\{share}"),
        Utf8Prefix::VerbatimUNC(server, share) => format!("\\\\?\\UNC\\{server}\\{share}"),
        Utf8Prefix::DeviceNS(device) => format!("\\\\.\\{device}"),
        Utf8Prefix::Verbatim(prefix) => format!("\\\\?\\{prefix}"),
      }),
      Utf8Component::RootDir => Cow::Borrowed("/"),
      _ => Cow::Borrowed(component.as_str()),
    };
    for ch in component.chars() {
      match ch {
        '%' => result.push_str("%25"),
        '/' => result.push_str("%2F"),
        '\\' => result.push_str("%5C"),
        _ => result.push(ch),
      }
    }
  }
  for component in path
    .strip_prefix(root)
    .expect("path starts with its root")
    .iter()
  {
    result.push('/');
    result.push_str(component);
  }
  result
}

fn time_ms(time: Option<SystemTime>) -> u64 {
  time
    .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
    .map_or(0, |time| time.as_millis() as u64)
}

#[derive(Debug, Default, Clone)]
pub struct MemoryFileSystem {
  // Serialize compound mutations so readers cannot see a partially copied rename.
  fs: Arc<Mutex<MemoryFS>>,
}

impl MemoryFileSystem {
  fn metadata_inner(fs: &MemoryFS, path: &Utf8Path) -> Result<FileMetadata> {
    let metadata = fs
      .metadata(&vfs_path(path))
      .map_err(|err| vfs_error(path, err))?;
    Ok(FileMetadata {
      is_file: metadata.file_type == VfsFileType::File,
      is_directory: metadata.file_type == VfsFileType::Directory,
      is_symlink: false,
      atime_ms: time_ms(metadata.accessed),
      mtime_ms: time_ms(metadata.modified),
      ctime_ms: time_ms(metadata.created),
      size: metadata.len,
    })
  }

  fn existing_metadata(fs: &MemoryFS, path: &Utf8Path) -> Result<Option<FileMetadata>> {
    match Self::metadata_inner(fs, path) {
      Ok(metadata) => Ok(Some(metadata)),
      Err(Error::Io(error)) if error.kind() == ErrorKind::NotFound => Ok(None),
      Err(error) => Err(error),
    }
  }

  fn ensure_dir(fs: &MemoryFS, path: &Utf8Path) -> Result<()> {
    if !Self::metadata_inner(fs, path)?.is_directory {
      return Err(Error::new(
        ErrorKind::NotADirectory,
        &format!("{path}: not a directory"),
      ));
    }
    Ok(())
  }

  fn create_dir_inner(fs: &MemoryFS, path: &Utf8Path, recursive: bool) -> Result<()> {
    if let Some(metadata) = Self::existing_metadata(fs, path)? {
      if metadata.is_directory {
        return Ok(());
      }
      return Err(Error::new(
        ErrorKind::NotADirectory,
        &format!("{path}: not a directory"),
      ));
    }
    if let Some(parent) = path.parent() {
      if recursive {
        Self::create_dir_inner(fs, parent, true)?;
      } else {
        Self::ensure_dir(fs, parent)?;
      }
    }
    fs.create_dir(&vfs_path(path))
      .map_err(|err| vfs_error(path, err))
  }

  /// Create a directory and any missing ancestors synchronously.
  /// Returns an error if an existing ancestor is a file.
  pub fn create_dir_all_sync(&self, path: &Utf8Path) -> Result<()> {
    let fs = self.fs.lock().expect("should get lock");
    Self::create_dir_inner(&fs, path, true)
  }

  fn write_contents(fs: &MemoryFS, path: &Utf8Path, data: &[u8]) -> Result<()> {
    let mut writer = fs
      .create_file(&vfs_path(path))
      .map_err(|err| vfs_error(path, err))?;
    writer.write_all(data).to_fs_result()?;
    // MemoryFS commits the bytes and updates its timestamps when the writer drops.
    drop(writer);
    Ok(())
  }

  /// Write a file synchronously using the backend's native timestamps.
  /// Returns an error if the parent is missing or the path is a directory.
  pub fn write_sync(&self, path: &Utf8Path, data: &[u8]) -> Result<()> {
    let fs = self.fs.lock().expect("should get lock");
    let previous = Self::existing_metadata(&fs, path)?;
    if previous
      .as_ref()
      .is_some_and(|metadata| metadata.is_directory)
    {
      return Err(Error::new(
        ErrorKind::IsADirectory,
        &format!("{path}: is a directory"),
      ));
    }
    let parent = path
      .parent()
      .ok_or_else(|| Error::new(ErrorKind::InvalidInput, "file has no parent directory"))?;
    Self::ensure_dir(&fs, parent)?;
    Self::write_contents(&fs, path, data)
  }

  fn read_inner(fs: &MemoryFS, path: &Utf8Path) -> Result<Vec<u8>> {
    let metadata = Self::metadata_inner(fs, path)?;
    if !metadata.is_file {
      return Err(Error::new(
        ErrorKind::IsADirectory,
        &format!("{path}: is a directory"),
      ));
    }
    let backend_path = vfs_path(path);
    let mut reader = fs
      .open_file(&backend_path)
      .map_err(|err| vfs_error(path, err))?;
    let mut contents = Vec::new();
    reader.read_to_end(&mut contents).to_fs_result()?;
    Ok(contents)
  }

  fn read_dir_inner(fs: &MemoryFS, path: &Utf8Path) -> Result<Vec<String>> {
    Self::ensure_dir(fs, path)?;
    Ok(
      fs.read_dir(&vfs_path(path))
        .map_err(|err| vfs_error(path, err))?
        .collect(),
    )
  }

  fn remove_inner(fs: &MemoryFS, path: &Utf8Path, directory: bool) -> Result<()> {
    let Some(metadata) = Self::existing_metadata(fs, path)? else {
      return Ok(());
    };
    if directory != metadata.is_directory {
      let kind = if directory {
        ErrorKind::NotADirectory
      } else {
        ErrorKind::IsADirectory
      };
      return Err(Error::new(kind, &format!("{path}: unexpected file type")));
    }
    if directory {
      for name in Self::read_dir_inner(fs, path)? {
        let child = path.join(name);
        let child_metadata = Self::metadata_inner(fs, &child)?;
        Self::remove_inner(fs, &child, child_metadata.is_directory)?;
      }
      fs.remove_dir(&vfs_path(path))
        .map_err(|err| vfs_error(path, err))?;
    } else {
      fs.remove_file(&vfs_path(path))
        .map_err(|err| vfs_error(path, err))?;
    }
    Ok(())
  }

  fn copy_inner(fs: &MemoryFS, from: &Utf8Path, to: &Utf8Path) -> Result<()> {
    let metadata = Self::metadata_inner(fs, from)?;
    if metadata.is_directory {
      fs.create_dir(&vfs_path(to))
        .map_err(|err| vfs_error(to, err))?;
      for name in Self::read_dir_inner(fs, from)? {
        Self::copy_inner(fs, &from.join(&name), &to.join(&name))?;
      }
    } else {
      Self::write_contents(fs, to, &Self::read_inner(fs, from)?)?;
    }
    Ok(())
  }

  fn _rename(&self, from: &Utf8Path, to: &Utf8Path) -> Result<()> {
    if from == to {
      return Ok(());
    }
    let fs = self.fs.lock().expect("should get lock");
    let parent = to.parent().ok_or_else(|| {
      Error::new(
        ErrorKind::NotFound,
        "target parent directory does not exist",
      )
    })?;
    Self::ensure_dir(&fs, parent)?;
    let metadata = Self::metadata_inner(&fs, from)?;
    if metadata.is_directory && to.starts_with(from) {
      return Err(Error::new(
        ErrorKind::InvalidInput,
        "cannot rename a directory into itself",
      ));
    }
    if let Some(target) = Self::existing_metadata(&fs, to)? {
      if metadata.is_directory {
        return Err(Error::new(
          ErrorKind::AlreadyExists,
          &format!("{to}: target already exists"),
        ));
      }
      if target.is_directory {
        return Err(Error::new(
          ErrorKind::IsADirectory,
          &format!("{to}: is a directory"),
        ));
      }
    }
    // MemoryFS has no rename operation. Copy/remove under one lock so readers
    // cannot observe a partially moved tree; the backend owns the new timestamps.
    Self::copy_inner(&fs, from, to)?;
    Self::remove_inner(&fs, from, metadata.is_directory)
  }
}
#[async_trait::async_trait]
impl WritableFileSystem for MemoryFileSystem {
  async fn create_dir(&self, dir: &Utf8Path) -> Result<()> {
    let fs = self.fs.lock().expect("should get lock");
    Self::create_dir_inner(&fs, dir, false)
  }

  async fn create_dir_all(&self, dir: &Utf8Path) -> Result<()> {
    self.create_dir_all_sync(dir)
  }

  async fn write(&self, file: &Utf8Path, data: &[u8]) -> Result<()> {
    self.write_sync(file, data)
  }

  async fn remove_file(&self, file: &Utf8Path) -> Result<()> {
    let fs = self.fs.lock().expect("should get lock");
    Self::remove_inner(&fs, file, false)
  }

  async fn remove_dir_all(&self, dir: &Utf8Path) -> Result<()> {
    let fs = self.fs.lock().expect("should get lock");
    Self::remove_inner(&fs, dir, true)
  }

  async fn read_dir(&self, dir: &Utf8Path) -> Result<Vec<String>> {
    self.read_dir_sync(dir)
  }

  async fn read_file(&self, file: &Utf8Path) -> Result<Vec<u8>> {
    ReadableFileSystem::read(self, file).await
  }

  async fn stat(&self, file: &Utf8Path) -> Result<FileMetadata> {
    ReadableFileSystem::metadata(self, file).await
  }

  async fn set_permissions(&self, _path: &Utf8Path, _perm: FilePermissions) -> Result<()> {
    Ok(())
  }
}

#[async_trait::async_trait]
impl ReadableFileSystem for MemoryFileSystem {
  async fn read(&self, path: &Utf8Path) -> Result<Vec<u8>> {
    self.read_sync(path)
  }

  fn read_sync(&self, path: &Utf8Path) -> Result<Vec<u8>> {
    let fs = self.fs.lock().expect("should get lock");
    Self::read_inner(&fs, path)
  }

  async fn metadata(&self, path: &Utf8Path) -> Result<FileMetadata> {
    self.metadata_sync(path)
  }

  fn metadata_sync(&self, path: &Utf8Path) -> Result<FileMetadata> {
    let fs = self.fs.lock().expect("should get lock");
    Self::metadata_inner(&fs, path)
  }

  async fn symlink_metadata(&self, path: &Utf8Path) -> Result<FileMetadata> {
    self.metadata(path).await
  }

  async fn read_link(&self, _path: &Utf8Path) -> Result<Utf8PathBuf> {
    Err(Error::new(
      std::io::ErrorKind::Unsupported,
      "read_link is not supported by this filesystem",
    ))
  }

  async fn canonicalize(&self, path: &Utf8Path) -> Result<Utf8PathBuf> {
    let path = dunce::canonicalize(path)?;
    Ok(path.assert_utf8())
  }

  async fn read_dir(&self, dir: &Utf8Path) -> Result<Vec<String>> {
    self.read_dir_sync(dir)
  }

  fn read_dir_sync(&self, dir: &Utf8Path) -> Result<Vec<String>> {
    let fs = self.fs.lock().expect("should get lock");
    Self::read_dir_inner(&fs, dir)
  }

  async fn permissions(&self, _path: &Utf8Path) -> Result<Option<FilePermissions>> {
    Ok(None)
  }
}

#[async_trait::async_trait]
impl IntermediateFileSystemExtras for MemoryFileSystem {
  async fn rename(&self, from: &Utf8Path, to: &Utf8Path) -> Result<()> {
    self._rename(from, to)?;
    Ok(())
  }

  async fn create_read_stream(&self, file: &Utf8Path) -> Result<Box<dyn ReadStream>> {
    let contents = self.read(file).await?;
    let reader = MemoryReadStream::new(contents);
    Ok(Box::new(reader))
  }

  async fn create_write_stream(&self, file: &Utf8Path) -> Result<Box<dyn WriteStream>> {
    let writer = MemoryWriteStream::new(file, self.clone());
    Ok(Box::new(writer))
  }
}

impl IntermediateFileSystem for MemoryFileSystem {}

#[derive(Debug)]
pub struct MemoryReadStream(Cursor<Vec<u8>>);

impl MemoryReadStream {
  pub fn new(contents: Vec<u8>) -> Self {
    Self(Cursor::new(contents))
  }
}

#[async_trait::async_trait]
impl ReadStream for MemoryReadStream {
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
pub struct MemoryWriteStream {
  file: Utf8PathBuf,
  contents: Vec<u8>,
  fs: MemoryFileSystem,
}

impl MemoryWriteStream {
  pub fn new(file: &Utf8Path, fs: MemoryFileSystem) -> Self {
    Self {
      file: file.to_path_buf(),
      contents: vec![],
      fs,
    }
  }
}

#[async_trait::async_trait]
impl WriteStream for MemoryWriteStream {
  async fn write(&mut self, buf: &[u8]) -> Result<usize> {
    self.contents.extend(buf);
    Ok(buf.len())
  }
  async fn write_all(&mut self, buf: &[u8]) -> Result<()> {
    self.contents = buf.to_vec();
    Ok(())
  }
  async fn flush(&mut self) -> Result<()> {
    self.fs.write(&self.file, &self.contents).await?;
    Ok(())
  }
  async fn close(&mut self) -> Result<()> {
    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use rspack_paths::Utf8Path;

  use super::{MemoryFileSystem, ReadableFileSystem, WritableFileSystem};
  #[tokio::test]
  async fn async_fs_test() {
    let fs = MemoryFileSystem::default();
    let file_content = "1".as_bytes();
    // init fs
    WritableFileSystem::create_dir_all(&fs, Utf8Path::new("/a/b/c"))
      .await
      .unwrap();
    WritableFileSystem::write(&fs, Utf8Path::new("/a/file1"), file_content)
      .await
      .unwrap();
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    // test create_dir
    assert!(
      WritableFileSystem::create_dir(&fs, Utf8Path::new("/a/b/c/d/e"))
        .await
        .is_err()
    );
    assert!(
      WritableFileSystem::create_dir(&fs, Utf8Path::new("/a/b/c/d"))
        .await
        .is_ok()
    );
    assert!(
      WritableFileSystem::create_dir(&fs, Utf8Path::new("/a/b/c/d/e"))
        .await
        .is_ok()
    );
    assert!(
      WritableFileSystem::create_dir(&fs, Utf8Path::new("/a/file1/c/d"))
        .await
        .is_err()
    );
    assert!(
      WritableFileSystem::create_dir(&fs, Utf8Path::new("/a/file1/c"))
        .await
        .is_err()
    );

    // test create_dir_all
    assert!(
      WritableFileSystem::create_dir_all(&fs, Utf8Path::new("/a1/b1/c1"))
        .await
        .is_ok()
    );
    assert!(
      WritableFileSystem::create_dir_all(&fs, Utf8Path::new("/a/file1/c/d"))
        .await
        .is_err()
    );
    assert!(
      WritableFileSystem::create_dir_all(&fs, Utf8Path::new("/a/file1/c"))
        .await
        .is_err()
    );

    // test write
    assert!(
      WritableFileSystem::write(&fs, Utf8Path::new("/a/temp/file2"), file_content)
        .await
        .is_err()
    );
    assert!(
      WritableFileSystem::write(&fs, Utf8Path::new("/a/file2"), file_content)
        .await
        .is_ok()
    );
    assert!(
      WritableFileSystem::write(&fs, Utf8Path::new("/a/file1/file2"), file_content)
        .await
        .is_err()
    );

    // read
    assert!(
      ReadableFileSystem::read(&fs, Utf8Path::new("/a/temp/file2"))
        .await
        .is_err()
    );
    assert!(
      ReadableFileSystem::read(&fs, Utf8Path::new("/a/file1/file2"))
        .await
        .is_err()
    );
    assert_eq!(
      ReadableFileSystem::read(&fs, Utf8Path::new("/a/file1"))
        .await
        .unwrap(),
      file_content
    );
    assert_eq!(
      ReadableFileSystem::read(&fs, Utf8Path::new("/a/file2"))
        .await
        .unwrap(),
      file_content
    );

    // stat
    assert!(
      WritableFileSystem::stat(&fs, Utf8Path::new("/a/file1/c/d"))
        .await
        .is_err()
    );
    let file1_meta = WritableFileSystem::stat(&fs, Utf8Path::new("/a/file1"))
      .await
      .unwrap();
    let file2_meta = WritableFileSystem::stat(&fs, Utf8Path::new("/a/file2"))
      .await
      .unwrap();
    assert!(file1_meta.is_file);
    assert!(file2_meta.is_file);
    assert!(file1_meta.ctime_ms < file2_meta.ctime_ms);
    let dir_meta = WritableFileSystem::stat(&fs, Utf8Path::new("/a/b"))
      .await
      .unwrap();
    assert!(dir_meta.is_directory);
    assert!(dir_meta.ctime_ms < file2_meta.ctime_ms);

    // read dir
    assert!(
      WritableFileSystem::read_dir(&fs, Utf8Path::new("/a2/b2/c2"))
        .await
        .is_err(),
    );
    let children = WritableFileSystem::read_dir(&fs, Utf8Path::new("/a"))
      .await
      .unwrap();
    assert_eq!(children.len(), 3);
    assert!(children.contains(&String::from("b")));
    assert!(children.contains(&String::from("file1")));
    assert!(children.contains(&String::from("file2")));

    // remove file
    assert!(
      WritableFileSystem::remove_file(&fs, Utf8Path::new("/a/b/c"))
        .await
        .is_err(),
    );
    assert!(
      WritableFileSystem::remove_file(&fs, Utf8Path::new("/a/file3"))
        .await
        .is_ok(),
    );
    assert!(
      WritableFileSystem::remove_file(&fs, Utf8Path::new("/a/file2"))
        .await
        .is_ok(),
    );
    assert!(
      WritableFileSystem::stat(&fs, Utf8Path::new("/a/file2"))
        .await
        .is_err(),
    );

    // remove dir
    assert!(
      WritableFileSystem::remove_dir_all(&fs, Utf8Path::new("/a3/b3/c3"))
        .await
        .is_ok(),
    );
    assert!(
      WritableFileSystem::remove_dir_all(&fs, Utf8Path::new("/a/file1"))
        .await
        .is_err(),
    );
    assert!(
      WritableFileSystem::remove_dir_all(&fs, Utf8Path::new("/a"))
        .await
        .is_ok(),
    );
    assert!(
      WritableFileSystem::stat(&fs, Utf8Path::new("/a/file1"))
        .await
        .is_err(),
    );
  }
}
