use std::sync::Arc;

mod error;
pub use error::{BatchStorageFSError, StorageFSError, StorageFSOperation};
use rspack_fs::{FileMetadata, IntermediateFileSystem, ReadStream, WriteStream};
use rspack_paths::{Utf8Path, Utf8PathBuf};
use rustc_hash::FxHashSet as HashSet;

pub type StorageFSResult<T> = Result<T, StorageFSError>;

#[async_trait::async_trait]
pub trait StorageFS: std::fmt::Debug + Sync + Send {
  async fn exists(&self, path: &Utf8Path) -> StorageFSResult<bool>;
  async fn remove_dir(&self, path: &Utf8Path) -> StorageFSResult<()>;
  async fn ensure_dir(&self, path: &Utf8Path) -> StorageFSResult<()>;
  async fn write_file(&self, path: &Utf8Path) -> StorageFSResult<StorageWriter>;
  async fn read_file(&self, path: &Utf8Path) -> StorageFSResult<StorageReader>;
  async fn read_dir(&self, path: &Utf8Path) -> StorageFSResult<HashSet<String>>;
  async fn metadata(&self, path: &Utf8Path) -> StorageFSResult<FileMetadata>;
  async fn remove_file(&self, path: &Utf8Path) -> StorageFSResult<()>;
  async fn move_file(&self, from: &Utf8Path, to: &Utf8Path) -> StorageFSResult<()>;
}

#[derive(Debug)]
pub struct StorageWriter {
  path: Utf8PathBuf,
  stream: Box<dyn WriteStream>,
}

impl StorageWriter {
  pub async fn write_line(&mut self, line: &str) -> StorageFSResult<()> {
    self
      .stream
      .write_line(line)
      .await
      .map_err(|e| StorageFSError::from_fs_error(&self.path, StorageFSOperation::Write, e))
  }
  pub async fn write(&mut self, buf: &[u8]) -> StorageFSResult<usize> {
    self
      .stream
      .write(buf)
      .await
      .map_err(|e| StorageFSError::from_fs_error(&self.path, StorageFSOperation::Write, e))
  }
  pub async fn write_all(&mut self, buf: &[u8]) -> StorageFSResult<()> {
    self
      .stream
      .write_all(buf)
      .await
      .map_err(|e| StorageFSError::from_fs_error(&self.path, StorageFSOperation::Write, e))
  }
  pub async fn flush(&mut self) -> StorageFSResult<()> {
    self
      .stream
      .flush()
      .await
      .map_err(|e| StorageFSError::from_fs_error(&self.path, StorageFSOperation::Write, e))
  }
  pub async fn close(&mut self) -> StorageFSResult<()> {
    self
      .stream
      .close()
      .await
      .map_err(|e| StorageFSError::from_fs_error(&self.path, StorageFSOperation::Write, e))
  }
}

#[derive(Debug)]
pub struct StorageReader {
  path: Utf8PathBuf,
  stream: Box<dyn ReadStream>,
}

impl StorageReader {
  pub async fn read_line(&mut self) -> StorageFSResult<String> {
    self
      .stream
      .read_line()
      .await
      .map_err(|e| StorageFSError::from_fs_error(&self.path, StorageFSOperation::Read, e))
  }
  pub async fn read(&mut self, length: usize) -> StorageFSResult<Vec<u8>> {
    self
      .stream
      .read(length)
      .await
      .map_err(|e| StorageFSError::from_fs_error(&self.path, StorageFSOperation::Read, e))
  }
  pub async fn read_until(&mut self, byte: u8) -> StorageFSResult<Vec<u8>> {
    self
      .stream
      .read_until(byte)
      .await
      .map_err(|e| StorageFSError::from_fs_error(&self.path, StorageFSOperation::Read, e))
  }
  pub async fn read_to_end(&mut self) -> StorageFSResult<Vec<u8>> {
    self
      .stream
      .read_to_end()
      .await
      .map_err(|e| StorageFSError::from_fs_error(&self.path, StorageFSOperation::Read, e))
  }
  pub async fn skip(&mut self, offset: usize) -> StorageFSResult<()> {
    self
      .stream
      .skip(offset)
      .await
      .map_err(|e| StorageFSError::from_fs_error(&self.path, StorageFSOperation::Read, e))
  }
  pub async fn close(&mut self) -> StorageFSResult<()> {
    self
      .stream
      .close()
      .await
      .map_err(|e| StorageFSError::from_fs_error(&self.path, StorageFSOperation::Read, e))
  }
}

#[derive(Debug)]
pub struct StorageBridgeFS(pub Arc<dyn IntermediateFileSystem>);

#[async_trait::async_trait]
impl StorageFS for StorageBridgeFS {
  async fn exists(&self, path: &Utf8Path) -> StorageFSResult<bool> {
    match self.metadata(path).await {
      Ok(_) => Ok(true),
      Err(e) => {
        if e.is_not_found() {
          Ok(false)
        } else {
          Err(e)
        }
      }
    }
  }

  async fn remove_dir(&self, path: &Utf8Path) -> StorageFSResult<()> {
    if self.exists(path).await? {
      self
        .0
        .remove_dir_all(path)
        .await
        .map_err(|e| StorageFSError::from_fs_error(path, StorageFSOperation::Remove, e))?;
    }
    Ok(())
  }

  async fn ensure_dir(&self, path: &Utf8Path) -> StorageFSResult<()> {
    self
      .0
      .create_dir_all(path)
      .await
      .map_err(|e| StorageFSError::from_fs_error(path, StorageFSOperation::Dir, e))?;
    Ok(())
  }

  async fn write_file(&self, path: &Utf8Path) -> StorageFSResult<StorageWriter> {
    if self.exists(path).await? {
      self.remove_file(path).await?;
    }
    self
      .ensure_dir(path.parent().expect("should have parent"))
      .await?;

    let stream = self
      .0
      .create_write_stream(path)
      .await
      .map_err(|e| StorageFSError::from_fs_error(path, StorageFSOperation::Write, e))?;

    Ok(StorageWriter {
      path: path.to_path_buf(),
      stream,
    })
  }

  async fn read_file(&self, path: &Utf8Path) -> StorageFSResult<StorageReader> {
    let stream = self
      .0
      .create_read_stream(path)
      .await
      .map_err(|e| StorageFSError::from_fs_error(path, StorageFSOperation::Read, e))?;
    Ok(StorageReader {
      path: path.to_path_buf(),
      stream,
    })
  }

  async fn read_dir(&self, path: &Utf8Path) -> StorageFSResult<HashSet<String>> {
    let files = self
      .0
      .read_dir(path)
      .await
      .map_err(|e| StorageFSError::from_fs_error(path, StorageFSOperation::Read, e))?;
    Ok(files.into_iter().collect::<HashSet<_>>())
  }

  async fn metadata(&self, path: &Utf8Path) -> StorageFSResult<FileMetadata> {
    let res = self
      .0
      .stat(path)
      .await
      .map_err(|e| StorageFSError::from_fs_error(path, StorageFSOperation::Stat, e))?;
    Ok(res)
  }

  async fn remove_file(&self, path: &Utf8Path) -> StorageFSResult<()> {
    if self.exists(path).await? {
      self
        .0
        .remove_file(path)
        .await
        .map_err(|e| StorageFSError::from_fs_error(path, StorageFSOperation::Remove, e))?;
    }
    Ok(())
  }

  async fn move_file(&self, from: &Utf8Path, to: &Utf8Path) -> StorageFSResult<()> {
    if self.exists(from).await? {
      self
        .ensure_dir(to.parent().expect("should have parent"))
        .await?;
      self
        .0
        .rename(from, to)
        .await
        .map_err(|e| StorageFSError::from_fs_error(from, StorageFSOperation::Move, e))?;
    }
    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use std::sync::Arc;

  use rspack_fs::MemoryFileSystem;
  use rspack_paths::Utf8PathBuf;

  use super::{StorageBridgeFS, StorageFSResult};
  use crate::StorageFS;

  fn get_path(p: &str) -> Utf8PathBuf {
    Utf8PathBuf::from(p)
  }

  async fn test_create_dir(fs: &StorageBridgeFS) -> StorageFSResult<()> {
    fs.ensure_dir(&get_path("/parent/from")).await?;
    fs.ensure_dir(&get_path("/parent/to")).await?;

    assert!(fs.exists(&get_path("/parent/from")).await?);
    assert!(fs.exists(&get_path("/parent/to")).await?);

    assert!(fs.metadata(&get_path("/parent/from")).await?.is_directory);
    assert!(fs.metadata(&get_path("/parent/to")).await?.is_directory);

    Ok(())
  }

  async fn test_write_file(fs: &StorageBridgeFS) -> StorageFSResult<()> {
    let mut writer = fs.write_file(&get_path("/parent/from/file.txt")).await?;

    writer.write_line("hello").await?;
    writer.write(b" world").await?;
    writer.flush().await?;

    assert!(fs.exists(&get_path("/parent/from/file.txt")).await?);
    assert!(
      fs.metadata(&get_path("/parent/from/file.txt"))
        .await?
        .is_file
    );

    Ok(())
  }

  async fn test_read_file(fs: &StorageBridgeFS) -> StorageFSResult<()> {
    let mut reader = fs.read_file(&get_path("/parent/from/file.txt")).await?;

    assert_eq!(reader.read_line().await?, "hello");
    assert_eq!(reader.read(b" world".len()).await?, b" world");

    Ok(())
  }

  async fn test_move_file(fs: &StorageBridgeFS) -> StorageFSResult<()> {
    fs.move_file(
      &get_path("/parent/from/file.txt"),
      &get_path("/parent/to/file.txt"),
    )
    .await?;
    assert!(!fs.exists(&get_path("/parent/from/file.txt")).await?);
    assert!(fs.exists(&get_path("/parent/to/file.txt")).await?);
    assert!(fs.metadata(&get_path("/parent/to/file.txt")).await?.is_file);

    Ok(())
  }

  async fn test_remove_file(fs: &StorageBridgeFS) -> StorageFSResult<()> {
    fs.remove_file(&get_path("/parent/to/file.txt")).await?;
    assert!(!fs.exists(&get_path("/parent/to/file.txt")).await?);
    Ok(())
  }

  async fn test_remove_dir(fs: &StorageBridgeFS) -> StorageFSResult<()> {
    fs.remove_dir(&get_path("/parent/from")).await?;
    fs.remove_dir(&get_path("/parent/to")).await?;
    assert!(!fs.exists(&get_path("/parent/from")).await?);
    assert!(!fs.exists(&get_path("/parent/to")).await?);
    Ok(())
  }

  async fn test_error(fs: &StorageBridgeFS) -> StorageFSResult<()> {
    match fs.metadata(&get_path("/parent/from/not_exist.txt")).await {
      Ok(_) => panic!("should error"),
      Err(e) => assert_eq!(
        e.to_string(),
        r#"stat `/parent/from/not_exist.txt` failed due to `file not exist`"#
      ),
    };

    Ok(())
  }

  async fn test_memory_fs(fs: &StorageBridgeFS) -> StorageFSResult<()> {
    test_create_dir(fs).await?;
    test_write_file(fs).await?;
    test_read_file(fs).await?;
    test_move_file(fs).await?;
    test_remove_file(fs).await?;
    test_remove_dir(fs).await?;
    test_error(fs).await?;

    Ok(())
  }

  #[tokio::test]
  #[cfg_attr(miri, ignore)]
  async fn should_storage_bridge_fs_work() {
    let fs = StorageBridgeFS(Arc::new(MemoryFileSystem::default()));

    let _ = test_memory_fs(&fs).await.map_err(|e| {
      panic!("{}", e);
    });
  }
}
