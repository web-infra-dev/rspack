use std::{
  borrow::Cow,
  io::{BufRead, Cursor, ErrorKind, Read, Seek, Write},
  sync::Arc,
  time::{SystemTime, UNIX_EPOCH},
};

use vfs::{
  FileSystem as VfsFileSystem, MemoryFS as VfsMemoryFS, VfsError, VfsFileType, VfsMetadata,
  error::VfsErrorKind,
};

use super::{
  FileMetadata, FilePermissions, IntermediateFileSystem, IntermediateFileSystemExtras, ReadStream,
  ReadableFileSystem, WritableFileSystem, WriteStream,
};
use crate::{AbsPath, Error, IoResultToFsResultExt, Result};

/// In-memory filesystem backed by `vfs::MemoryFS` operating strictly on UTF-8 [`AbsPath`]s.
#[derive(Debug, Default, Clone)]
pub struct MemoryFileSystem {
  inner: Arc<VfsMemoryFS>,
}

impl MemoryFileSystem {
  pub fn new() -> Self {
    Self::default()
  }

  pub fn exists(&self, path: &AbsPath) -> bool {
    let vfs_path = abs_to_vfs_path(path);
    self.inner.exists(vfs_path.as_ref()).unwrap_or(false)
  }

  pub fn create_dir(&self, dir: &AbsPath) -> Result<()> {
    let vfs_path = abs_to_vfs_path(dir);
    if vfs_path.is_empty() {
      return Ok(());
    }
    match self.inner.metadata(vfs_path.as_ref()) {
      Ok(meta) if meta.file_type == VfsFileType::Directory => return Ok(()),
      Ok(_) => {
        return Err(Error::new(
          ErrorKind::AlreadyExists,
          format!("path exists and is not a directory: {dir}"),
        ));
      }
      Err(err) if matches!(err.kind(), VfsErrorKind::FileNotFound) => {}
      Err(err) => return Err(vfs_err_to_fs_err(err)),
    }
    self.ensure_parent_is_dir(vfs_path.as_ref())?;
    self
      .inner
      .create_dir(vfs_path.as_ref())
      .map_err(vfs_err_to_fs_err)
  }

  pub fn create_dir_all(&self, dir: &AbsPath) -> Result<()> {
    let vfs_path = abs_to_vfs_path(dir);
    if vfs_path.is_empty() {
      return Ok(());
    }
    if let Ok(meta) = self.inner.metadata(vfs_path.as_ref()) {
      return if meta.file_type == VfsFileType::Directory {
        Ok(())
      } else {
        Err(Error::new(
          ErrorKind::AlreadyExists,
          format!("path exists and is not a directory: {dir}"),
        ))
      };
    }
    let bytes = vfs_path.as_bytes();
    for (idx, &b) in bytes.iter().enumerate().skip(1) {
      if b == b'/' {
        self.ensure_dir_segment(&vfs_path[..idx])?;
      }
    }
    self.ensure_dir_segment(vfs_path.as_ref())
  }

  pub fn write(&self, file: &AbsPath, data: &[u8]) -> Result<()> {
    let vfs_path = abs_to_vfs_path(file);
    if vfs_path.is_empty() {
      return Err(Error::new(
        ErrorKind::IsADirectory,
        format!("cannot write to root directory: {file}"),
      ));
    }
    self.ensure_parent_is_dir(vfs_path.as_ref())?;
    match self.inner.metadata(vfs_path.as_ref()) {
      Ok(meta) if meta.file_type == VfsFileType::Directory => {
        return Err(Error::new(
          ErrorKind::IsADirectory,
          format!("path is a directory: {file}"),
        ));
      }
      Ok(_) => {}
      Err(err) if matches!(err.kind(), VfsErrorKind::FileNotFound) => {}
      Err(err) => return Err(vfs_err_to_fs_err(err)),
    }
    let mut handle = self
      .inner
      .create_file(vfs_path.as_ref())
      .map_err(vfs_err_to_fs_err)?;
    handle.write_all(data)?;
    handle.flush()?;
    Ok(())
  }

  fn ensure_parent_is_dir(&self, vfs_path: &str) -> Result<()> {
    let parent = vfs_path.rfind('/').map_or("", |idx| &vfs_path[..idx]);
    let meta = self.inner.metadata(parent).map_err(vfs_err_to_fs_err)?;
    if meta.file_type != VfsFileType::Directory {
      return Err(Error::new(
        ErrorKind::NotADirectory,
        format!("parent path is not a directory: {parent}"),
      ));
    }
    Ok(())
  }

  fn ensure_dir_segment(&self, seg: &str) -> Result<()> {
    match self.inner.metadata(seg) {
      Ok(meta) if meta.file_type == VfsFileType::Directory => Ok(()),
      Ok(_) => Err(Error::new(
        ErrorKind::AlreadyExists,
        format!("ancestor path is not a directory: {seg}"),
      )),
      Err(err) if matches!(err.kind(), VfsErrorKind::FileNotFound) => {
        match self.inner.create_dir(seg) {
          Ok(()) => Ok(()),
          Err(err) if matches!(err.kind(), VfsErrorKind::DirectoryExists) => Ok(()),
          Err(err) => Err(vfs_err_to_fs_err(err)),
        }
      }
      Err(err) => Err(vfs_err_to_fs_err(err)),
    }
  }

  pub fn remove_file(&self, file: &AbsPath) -> Result<()> {
    let vfs_path = abs_to_vfs_path(file);
    if vfs_path.is_empty() {
      return Err(Error::new(
        ErrorKind::IsADirectory,
        format!("cannot remove root directory as file: {file}"),
      ));
    }
    match self.inner.metadata(vfs_path.as_ref()) {
      Ok(meta) if meta.file_type == VfsFileType::Directory => {
        return Err(Error::new(
          ErrorKind::IsADirectory,
          format!("path is a directory: {file}"),
        ));
      }
      Ok(_) => {}
      Err(err) if matches!(err.kind(), VfsErrorKind::FileNotFound) => return Ok(()),
      Err(err) => return Err(vfs_err_to_fs_err(err)),
    }
    match self.inner.remove_file(vfs_path.as_ref()) {
      Ok(()) => Ok(()),
      Err(err) if matches!(err.kind(), VfsErrorKind::FileNotFound) => Ok(()),
      Err(err) => Err(vfs_err_to_fs_err(err)),
    }
  }

  pub fn remove_dir_all(&self, dir: &AbsPath) -> Result<()> {
    let vfs_path = abs_to_vfs_path(dir);
    match self.inner.metadata(vfs_path.as_ref()) {
      Ok(meta) if meta.file_type == VfsFileType::Directory => {}
      Ok(_) => {
        return Err(Error::new(
          ErrorKind::NotADirectory,
          format!("path is not a directory: {dir}"),
        ));
      }
      Err(err) if matches!(err.kind(), VfsErrorKind::FileNotFound) => return Ok(()),
      Err(err) => return Err(vfs_err_to_fs_err(err)),
    }
    self.remove_dir_contents_recursive(vfs_path.as_ref())?;
    if !vfs_path.is_empty() {
      match self.inner.remove_dir(vfs_path.as_ref()) {
        Ok(()) => {}
        Err(err) if matches!(err.kind(), VfsErrorKind::FileNotFound) => {}
        Err(err) => return Err(vfs_err_to_fs_err(err)),
      }
    }
    Ok(())
  }

  pub fn rename(&self, from: &AbsPath, to: &AbsPath) -> Result<()> {
    let from_vfs = abs_to_vfs_path(from);
    if from == to {
      self
        .inner
        .metadata(from_vfs.as_ref())
        .map_err(vfs_err_to_fs_err)?;
      return Ok(());
    }

    let from_meta = self
      .inner
      .metadata(from_vfs.as_ref())
      .map_err(vfs_err_to_fs_err)?;
    let to_vfs = abs_to_vfs_path(to);
    self.ensure_parent_is_dir(to_vfs.as_ref())?;

    if from_meta.file_type == VfsFileType::File {
      match self.inner.metadata(to_vfs.as_ref()) {
        Ok(meta) if meta.file_type == VfsFileType::Directory => {
          return Err(Error::new(
            ErrorKind::IsADirectory,
            format!("target path is a directory: {to}"),
          ));
        }
        Ok(_) => {}
        Err(err) if matches!(err.kind(), VfsErrorKind::FileNotFound) => {}
        Err(err) => return Err(vfs_err_to_fs_err(err)),
      }
      let data = self.read_sync(from)?;
      self.write(to, &data)?;
      self
        .inner
        .remove_file(from_vfs.as_ref())
        .map_err(vfs_err_to_fs_err)?;
      return Ok(());
    }

    if to.strip_prefix(from).is_some() {
      return Err(Error::new(
        ErrorKind::InvalidInput,
        format!("cannot rename directory `{from}` into itself `{to}`"),
      ));
    }
    match self.inner.metadata(to_vfs.as_ref()) {
      Ok(_) => {
        return Err(Error::new(
          ErrorKind::AlreadyExists,
          format!("target path already exists: {to}"),
        ));
      }
      Err(err) if matches!(err.kind(), VfsErrorKind::FileNotFound) => {}
      Err(err) => return Err(vfs_err_to_fs_err(err)),
    }

    self
      .inner
      .create_dir(to_vfs.as_ref())
      .map_err(vfs_err_to_fs_err)?;
    self.copy_dir_contents_recursive(from_vfs.as_ref(), to_vfs.as_ref())?;
    self.remove_dir_all(from)
  }

  fn remove_dir_contents_recursive(&self, vfs_path: &str) -> Result<()> {
    let entries: Vec<String> = self
      .inner
      .read_dir(vfs_path)
      .map_err(vfs_err_to_fs_err)?
      .collect();
    for entry in entries {
      let child = format!("{vfs_path}/{entry}");
      let meta = self.inner.metadata(&child).map_err(vfs_err_to_fs_err)?;
      if meta.file_type == VfsFileType::Directory {
        self.remove_dir_contents_recursive(&child)?;
        self.inner.remove_dir(&child).map_err(vfs_err_to_fs_err)?;
      } else {
        self.inner.remove_file(&child).map_err(vfs_err_to_fs_err)?;
      }
    }
    Ok(())
  }

  fn copy_dir_contents_recursive(&self, src_vfs: &str, dst_vfs: &str) -> Result<()> {
    let entries: Vec<String> = self
      .inner
      .read_dir(src_vfs)
      .map_err(vfs_err_to_fs_err)?
      .collect();
    for entry in entries {
      let src_child = format!("{src_vfs}/{entry}");
      let dst_child = format!("{dst_vfs}/{entry}");
      let meta = self.inner.metadata(&src_child).map_err(vfs_err_to_fs_err)?;
      if meta.file_type == VfsFileType::Directory {
        self
          .inner
          .create_dir(&dst_child)
          .map_err(vfs_err_to_fs_err)?;
        self.copy_dir_contents_recursive(&src_child, &dst_child)?;
      } else {
        let mut reader = self
          .inner
          .open_file(&src_child)
          .map_err(vfs_err_to_fs_err)?;
        let mut buf = Vec::new();
        reader.read_to_end(&mut buf)?;
        let mut writer = self
          .inner
          .create_file(&dst_child)
          .map_err(vfs_err_to_fs_err)?;
        writer.write_all(&buf)?;
        writer.flush()?;
      }
    }
    Ok(())
  }
}

#[async_trait::async_trait]
impl ReadableFileSystem for MemoryFileSystem {
  async fn read(&self, path: &AbsPath) -> Result<Vec<u8>> {
    self.read_sync(path)
  }

  fn read_sync(&self, path: &AbsPath) -> Result<Vec<u8>> {
    let vfs_path = abs_to_vfs_path(path);
    let mut file = self
      .inner
      .open_file(vfs_path.as_ref())
      .map_err(vfs_err_to_fs_err)?;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf)?;
    Ok(buf)
  }

  async fn metadata(&self, path: &AbsPath) -> Result<FileMetadata> {
    self.metadata_sync(path)
  }

  fn metadata_sync(&self, path: &AbsPath) -> Result<FileMetadata> {
    let vfs_path = abs_to_vfs_path(path);
    let meta = self
      .inner
      .metadata(vfs_path.as_ref())
      .map_err(vfs_err_to_fs_err)?;
    Ok(vfs_metadata_to_file_metadata(&meta))
  }

  async fn symlink_metadata(&self, path: &AbsPath) -> Result<FileMetadata> {
    self.metadata(path).await
  }

  async fn read_link(&self, _path: &AbsPath) -> Result<String> {
    Err(Error::new(
      ErrorKind::Unsupported,
      "read_link is not supported by MemoryFileSystem",
    ))
  }

  async fn canonicalize(&self, path: &AbsPath) -> Result<AbsPath> {
    self.canonicalize_sync(path)
  }

  fn canonicalize_sync(&self, path: &AbsPath) -> Result<AbsPath> {
    let vfs_path = abs_to_vfs_path(path);
    if self
      .inner
      .exists(vfs_path.as_ref())
      .map_err(vfs_err_to_fs_err)?
    {
      Ok(path.clone())
    } else {
      Err(Error::new(
        ErrorKind::NotFound,
        format!("path not found: {path}"),
      ))
    }
  }

  async fn read_dir(&self, dir: &AbsPath) -> Result<Vec<String>> {
    self.read_dir_sync(dir)
  }

  fn read_dir_sync(&self, dir: &AbsPath) -> Result<Vec<String>> {
    let vfs_path = abs_to_vfs_path(dir);
    let meta = self
      .inner
      .metadata(vfs_path.as_ref())
      .map_err(vfs_err_to_fs_err)?;
    if meta.file_type != VfsFileType::Directory {
      return Err(Error::new(
        ErrorKind::NotADirectory,
        format!("not a directory: {dir}"),
      ));
    }
    let iter = self
      .inner
      .read_dir(vfs_path.as_ref())
      .map_err(vfs_err_to_fs_err)?;
    Ok(iter.collect())
  }
}

#[async_trait::async_trait]
impl WritableFileSystem for MemoryFileSystem {
  async fn create_dir(&self, dir: &AbsPath) -> Result<()> {
    Self::create_dir(self, dir)
  }

  async fn create_dir_all(&self, dir: &AbsPath) -> Result<()> {
    Self::create_dir_all(self, dir)
  }

  async fn write(&self, file: &AbsPath, data: &[u8]) -> Result<()> {
    Self::write(self, file, data)
  }

  async fn remove_file(&self, file: &AbsPath) -> Result<()> {
    Self::remove_file(self, file)
  }

  async fn remove_dir_all(&self, dir: &AbsPath) -> Result<()> {
    Self::remove_dir_all(self, dir)
  }

  async fn read_dir(&self, dir: &AbsPath) -> Result<Vec<String>> {
    ReadableFileSystem::read_dir_sync(self, dir)
  }

  async fn read_file(&self, file: &AbsPath) -> Result<Vec<u8>> {
    ReadableFileSystem::read_sync(self, file)
  }

  async fn stat(&self, file: &AbsPath) -> Result<FileMetadata> {
    ReadableFileSystem::metadata_sync(self, file)
  }

  async fn set_permissions(&self, _path: &AbsPath, _perm: FilePermissions) -> Result<()> {
    Ok(())
  }
}

#[async_trait::async_trait]
impl IntermediateFileSystemExtras for MemoryFileSystem {
  async fn rename(&self, from: &AbsPath, to: &AbsPath) -> Result<()> {
    Self::rename(self, from, to)
  }

  async fn create_read_stream(&self, file: &AbsPath) -> Result<Box<dyn ReadStream>> {
    let contents = ReadableFileSystem::read_sync(self, file)?;
    Ok(Box::new(MemoryReadStream::new(contents)))
  }

  async fn create_write_stream(&self, file: &AbsPath) -> Result<Box<dyn WriteStream>> {
    Ok(Box::new(MemoryWriteStream::new(file.clone(), self.clone())))
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
    if buf.last().is_some_and(|b| *b == byte) {
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
  file: AbsPath,
  contents: Vec<u8>,
  fs: MemoryFileSystem,
}

impl MemoryWriteStream {
  pub fn new(file: AbsPath, fs: MemoryFileSystem) -> Self {
    Self {
      file,
      contents: vec![],
      fs,
    }
  }
}

#[async_trait::async_trait]
impl WriteStream for MemoryWriteStream {
  async fn write(&mut self, buf: &[u8]) -> Result<usize> {
    self.contents.extend_from_slice(buf);
    Ok(buf.len())
  }

  async fn write_all(&mut self, buf: &[u8]) -> Result<()> {
    self.contents.extend_from_slice(buf);
    Ok(())
  }

  async fn flush(&mut self) -> Result<()> {
    self.fs.write(&self.file, &self.contents)
  }

  async fn close(&mut self) -> Result<()> {
    self.flush().await
  }
}

fn abs_to_vfs_path(path: &AbsPath) -> Cow<'_, str> {
  let s = path.as_str();
  if cfg!(windows) {
    if let Some(unc) = s.strip_prefix(r"\\") {
      let mut out = String::with_capacity(s.len() + 4);
      out.push_str("/UNC/");
      for ch in unc.chars() {
        out.push(if ch == '\\' { '/' } else { ch });
      }
      if out.ends_with('/') {
        out.pop();
      }
      Cow::Owned(out)
    } else {
      let mut out = String::with_capacity(s.len() + 1);
      out.push('/');
      for ch in s.chars() {
        out.push(if ch == '\\' { '/' } else { ch });
      }
      if out.ends_with('/') {
        out.pop();
      }
      Cow::Owned(out)
    }
  } else if s == "/" {
    Cow::Borrowed("")
  } else {
    Cow::Borrowed(s)
  }
}

fn vfs_err_to_fs_err(err: VfsError) -> Error {
  let kind = match err.kind() {
    VfsErrorKind::FileNotFound => ErrorKind::NotFound,
    VfsErrorKind::FileExists | VfsErrorKind::DirectoryExists => ErrorKind::AlreadyExists,
    VfsErrorKind::NotSupported => ErrorKind::Unsupported,
    VfsErrorKind::IoError(io_err) => io_err.kind(),
    _ => ErrorKind::Other,
  };
  Error::new(kind, err.to_string())
}

fn system_time_to_ms(time: SystemTime) -> u64 {
  time
    .duration_since(UNIX_EPOCH)
    .map_or(0, |d| d.as_millis() as u64)
}

fn vfs_metadata_to_file_metadata(meta: &VfsMetadata) -> FileMetadata {
  let ctime_ms = meta.created.map_or(0, system_time_to_ms);
  let mtime_ms = meta.modified.map_or(ctime_ms, system_time_to_ms);
  let atime_ms = meta.accessed.map_or(mtime_ms, system_time_to_ms);
  FileMetadata {
    is_file: meta.file_type == VfsFileType::File,
    is_directory: meta.file_type == VfsFileType::Directory,
    is_symlink: false,
    atime_ms,
    mtime_ms,
    ctime_ms,
    size: meta.len,
  }
}
