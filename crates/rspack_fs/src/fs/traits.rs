use std::fmt::Debug;

use super::metadata::{FileMetadata, FilePermissions};
use crate::{AbsPath, Error, Result};

/// Low-level read-only filesystem interface operating strictly on UTF-8 [`AbsPath`]s.
#[async_trait::async_trait]
pub trait ReadableFileSystem: Debug + Send + Sync {
  async fn read(&self, path: &AbsPath) -> Result<Vec<u8>>;
  fn read_sync(&self, path: &AbsPath) -> Result<Vec<u8>>;

  async fn read_to_string(&self, path: &AbsPath) -> Result<String> {
    let data = self.read(path).await?;
    String::from_utf8(data).map_err(|_| {
      Error::new(
        std::io::ErrorKind::InvalidData,
        "stream did not contain valid UTF-8",
      )
    })
  }

  fn read_to_string_sync(&self, path: &AbsPath) -> Result<String> {
    let data = self.read_sync(path)?;
    String::from_utf8(data).map_err(|_| {
      Error::new(
        std::io::ErrorKind::InvalidData,
        "stream did not contain valid UTF-8",
      )
    })
  }

  fn is_virtual(&self, _path: &AbsPath) -> bool {
    false
  }

  async fn metadata(&self, path: &AbsPath) -> Result<FileMetadata>;
  fn metadata_sync(&self, path: &AbsPath) -> Result<FileMetadata>;

  async fn symlink_metadata(&self, path: &AbsPath) -> Result<FileMetadata>;

  async fn read_link(&self, path: &AbsPath) -> Result<String>;

  async fn canonicalize(&self, path: &AbsPath) -> Result<AbsPath>;
  fn canonicalize_sync(&self, path: &AbsPath) -> Result<AbsPath>;

  async fn read_dir(&self, dir: &AbsPath) -> Result<Vec<String>>;
  fn read_dir_sync(&self, dir: &AbsPath) -> Result<Vec<String>>;

  async fn permissions(&self, _path: &AbsPath) -> Result<Option<FilePermissions>> {
    Ok(None)
  }
}

/// Low-level writable filesystem interface operating strictly on UTF-8 [`AbsPath`]s.
#[async_trait::async_trait]
pub trait WritableFileSystem: Debug + Send + Sync {
  /// Creates a new, empty directory at the provided path.
  async fn create_dir(&self, dir: &AbsPath) -> Result<()>;

  /// Recursively create a directory and all of its parent components if they are missing.
  async fn create_dir_all(&self, dir: &AbsPath) -> Result<()>;

  /// Write a slice as the entire contents of a file, creating or replacing it.
  async fn write(&self, file: &AbsPath, data: &[u8]) -> Result<()>;

  /// Removes a file from the filesystem.
  async fn remove_file(&self, file: &AbsPath) -> Result<()>;

  /// Removes a directory at this path after removing all of its contents.
  async fn remove_dir_all(&self, dir: &AbsPath) -> Result<()>;

  /// Returns a list of entries in a directory.
  async fn read_dir(&self, dir: &AbsPath) -> Result<Vec<String>>;

  /// Read the entire contents of a file into a bytes vector.
  async fn read_file(&self, file: &AbsPath) -> Result<Vec<u8>>;

  async fn stat(&self, file: &AbsPath) -> Result<FileMetadata>;

  async fn set_permissions(&self, _path: &AbsPath, _perm: FilePermissions) -> Result<()>;
}

pub trait IntermediateFileSystem:
  WritableFileSystem + IntermediateFileSystemExtras + Debug + Send + Sync
{
}

#[async_trait::async_trait]
pub trait IntermediateFileSystemExtras: Debug + Send + Sync {
  async fn rename(&self, from: &AbsPath, to: &AbsPath) -> Result<()>;
  async fn create_read_stream(&self, file: &AbsPath) -> Result<Box<dyn ReadStream>>;
  async fn create_write_stream(&self, file: &AbsPath) -> Result<Box<dyn WriteStream>>;
}

#[async_trait::async_trait]
pub trait ReadStream: Debug + Sync + Send {
  async fn read_line(&mut self) -> Result<String> {
    match String::from_utf8(self.read_until(b'\n').await?) {
      Ok(s) => Ok(s),
      Err(_) => Err(Error::Io(std::io::Error::other("invalid utf8 line"))),
    }
  }
  async fn read(&mut self, length: usize) -> Result<Vec<u8>>;
  async fn read_until(&mut self, byte: u8) -> Result<Vec<u8>>;
  async fn read_to_end(&mut self) -> Result<Vec<u8>>;
  async fn skip(&mut self, offset: usize) -> Result<()>;
  async fn close(&mut self) -> Result<()>;
}

#[async_trait::async_trait]
pub trait WriteStream: Debug + Sync + Send {
  async fn write_line(&mut self, line: &str) -> Result<()> {
    self.write(line.as_bytes()).await?;
    self.write(b"\n").await?;
    Ok(())
  }
  async fn write(&mut self, buf: &[u8]) -> Result<usize>;
  async fn write_all(&mut self, buf: &[u8]) -> Result<()>;
  async fn flush(&mut self) -> Result<()>;
  async fn close(&mut self) -> Result<()>;
}
