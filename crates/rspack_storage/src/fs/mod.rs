use std::sync::Arc;

mod error;
use error::FsResultToStorageFsResult;
pub use error::{BatchFSError, BatchFSResult, FSError, FSOperation, FSResult};
use rspack_fs::{FileMetadata, IntermediateFileSystem, ReadStream, WriteStream};
use rspack_paths::{Utf8Path, Utf8PathBuf};
use rustc_hash::FxHashSet as HashSet;

#[async_trait::async_trait]
pub trait FileSystem: std::fmt::Debug + Sync + Send {
  async fn exists(&self, path: &Utf8Path) -> FSResult<bool>;
  async fn remove_dir(&self, path: &Utf8Path) -> FSResult<()>;
  async fn ensure_dir(&self, path: &Utf8Path) -> FSResult<()>;
  async fn write_file(&self, path: &Utf8Path) -> FSResult<Writer>;
  async fn read_file(&self, path: &Utf8Path) -> FSResult<Reader>;
  async fn read_dir(&self, path: &Utf8Path) -> FSResult<HashSet<String>>;
  async fn metadata(&self, path: &Utf8Path) -> FSResult<FileMetadata>;
  async fn remove_file(&self, path: &Utf8Path) -> FSResult<()>;
  async fn move_file(&self, from: &Utf8Path, to: &Utf8Path) -> FSResult<()>;
}

#[derive(Debug)]
pub struct Writer {
  path: Utf8PathBuf,
  stream: Box<dyn WriteStream>,
}

impl Writer {
  pub async fn write_line(&mut self, line: &str) -> FSResult<()> {
    self
      .stream
      .write_line(line)
      .await
      .to_storage_fs_result(&self.path, FSOperation::Write)
  }
  pub async fn write(&mut self, buf: &[u8]) -> FSResult<usize> {
    self
      .stream
      .write(buf)
      .await
      .to_storage_fs_result(&self.path, FSOperation::Write)
  }
  pub async fn write_all(&mut self, buf: &[u8]) -> FSResult<()> {
    self
      .stream
      .write_all(buf)
      .await
      .to_storage_fs_result(&self.path, FSOperation::Write)
  }
  pub async fn flush(&mut self) -> FSResult<()> {
    self
      .stream
      .flush()
      .await
      .to_storage_fs_result(&self.path, FSOperation::Write)
  }
  pub async fn close(&mut self) -> FSResult<()> {
    self
      .stream
      .close()
      .await
      .to_storage_fs_result(&self.path, FSOperation::Write)
  }
}

#[derive(Debug)]
pub struct Reader {
  path: Utf8PathBuf,
  stream: Box<dyn ReadStream>,
}

impl Reader {
  pub async fn read_line(&mut self) -> FSResult<String> {
    self
      .stream
      .read_line()
      .await
      .to_storage_fs_result(&self.path, FSOperation::Read)
  }
  pub async fn read(&mut self, length: usize) -> FSResult<Vec<u8>> {
    self
      .stream
      .read(length)
      .await
      .to_storage_fs_result(&self.path, FSOperation::Read)
  }
  pub async fn read_until(&mut self, byte: u8) -> FSResult<Vec<u8>> {
    self
      .stream
      .read_until(byte)
      .await
      .to_storage_fs_result(&self.path, FSOperation::Read)
  }
  pub async fn read_to_end(&mut self) -> FSResult<Vec<u8>> {
    self
      .stream
      .read_to_end()
      .await
      .to_storage_fs_result(&self.path, FSOperation::Read)
  }
  pub async fn skip(&mut self, offset: usize) -> FSResult<()> {
    self
      .stream
      .skip(offset)
      .await
      .to_storage_fs_result(&self.path, FSOperation::Read)
  }
  pub async fn close(&mut self) -> FSResult<()> {
    self
      .stream
      .close()
      .await
      .to_storage_fs_result(&self.path, FSOperation::Read)
  }
}

#[derive(Debug)]
pub struct BridgeFileSystem(pub Arc<dyn IntermediateFileSystem>);

#[async_trait::async_trait]
impl FileSystem for BridgeFileSystem {
  async fn exists(&self, path: &Utf8Path) -> FSResult<bool> {
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

  async fn remove_dir(&self, path: &Utf8Path) -> FSResult<()> {
    if self.exists(path).await? {
      self
        .0
        .remove_dir_all(path)
        .await
        .to_storage_fs_result(path, FSOperation::Remove)?;
    }
    Ok(())
  }

  async fn ensure_dir(&self, path: &Utf8Path) -> FSResult<()> {
    self
      .0
      .create_dir_all(path)
      .await
      .to_storage_fs_result(path, FSOperation::Dir)?;
    Ok(())
  }

  async fn write_file(&self, path: &Utf8Path) -> FSResult<Writer> {
    self
      .ensure_dir(path.parent().expect("should have parent"))
      .await?;

    let stream = self
      .0
      .create_write_stream(path)
      .await
      .to_storage_fs_result(path, FSOperation::Write)?;

    Ok(Writer {
      path: path.to_path_buf(),
      stream,
    })
  }

  async fn read_file(&self, path: &Utf8Path) -> FSResult<Reader> {
    let stream = self
      .0
      .create_read_stream(path)
      .await
      .to_storage_fs_result(path, FSOperation::Read)?;
    Ok(Reader {
      path: path.to_path_buf(),
      stream,
    })
  }

  async fn read_dir(&self, path: &Utf8Path) -> FSResult<HashSet<String>> {
    let files = self
      .0
      .read_dir(path)
      .await
      .to_storage_fs_result(path, FSOperation::Read)?;
    Ok(files.into_iter().collect::<HashSet<_>>())
  }

  async fn metadata(&self, path: &Utf8Path) -> FSResult<FileMetadata> {
    let res = self
      .0
      .stat(path)
      .await
      .to_storage_fs_result(path, FSOperation::Stat)?;
    Ok(res)
  }

  async fn remove_file(&self, path: &Utf8Path) -> FSResult<()> {
    if self.exists(path).await? {
      self
        .0
        .remove_file(path)
        .await
        .to_storage_fs_result(path, FSOperation::Remove)?;
    }
    Ok(())
  }

  async fn move_file(&self, from: &Utf8Path, to: &Utf8Path) -> FSResult<()> {
    if self.exists(from).await? {
      self
        .ensure_dir(to.parent().expect("should have parent"))
        .await?;
      self
        .0
        .rename(from, to)
        .await
        .to_storage_fs_result(from, FSOperation::Move)?;
    }
    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use std::sync::Arc;

  use rspack_fs::MemoryFileSystem;
  use rspack_paths::Utf8PathBuf;

  use super::{BridgeFileSystem, FSResult};
  use crate::FileSystem;

  fn get_path(p: &str) -> Utf8PathBuf {
    Utf8PathBuf::from(p)
  }

  async fn test_create_dir(fs: &BridgeFileSystem) -> FSResult<()> {
    fs.ensure_dir(&get_path("/parent/from")).await?;
    fs.ensure_dir(&get_path("/parent/to")).await?;

    assert!(fs.exists(&get_path("/parent/from")).await?);
    assert!(fs.exists(&get_path("/parent/to")).await?);

    assert!(fs.metadata(&get_path("/parent/from")).await?.is_directory);
    assert!(fs.metadata(&get_path("/parent/to")).await?.is_directory);

    Ok(())
  }

  async fn test_write_file(fs: &BridgeFileSystem) -> FSResult<()> {
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

  async fn test_read_file(fs: &BridgeFileSystem) -> FSResult<()> {
    let mut reader = fs.read_file(&get_path("/parent/from/file.txt")).await?;

    assert_eq!(reader.read_line().await?, "hello");
    assert_eq!(reader.read(b" world".len()).await?, b" world");

    Ok(())
  }

  async fn test_move_file(fs: &BridgeFileSystem) -> FSResult<()> {
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

  async fn test_remove_file(fs: &BridgeFileSystem) -> FSResult<()> {
    fs.remove_file(&get_path("/parent/to/file.txt")).await?;
    assert!(!fs.exists(&get_path("/parent/to/file.txt")).await?);
    Ok(())
  }

  async fn test_remove_dir(fs: &BridgeFileSystem) -> FSResult<()> {
    fs.remove_dir(&get_path("/parent/from")).await?;
    fs.remove_dir(&get_path("/parent/to")).await?;
    assert!(!fs.exists(&get_path("/parent/from")).await?);
    assert!(!fs.exists(&get_path("/parent/to")).await?);
    Ok(())
  }

  async fn test_error(fs: &BridgeFileSystem) -> FSResult<()> {
    match fs.metadata(&get_path("/parent/from/not_exist.txt")).await {
      Ok(_) => panic!("should error"),
      Err(e) => assert_eq!(
        e.to_string(),
        r#"stat `/parent/from/not_exist.txt` failed due to `file not exist`"#
      ),
    };

    Ok(())
  }

  async fn test_memory_fs(fs: &BridgeFileSystem) -> FSResult<()> {
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
  async fn should_storage_bridge_fs_work() -> FSResult<()> {
    let fs = BridgeFileSystem(Arc::new(MemoryFileSystem::default()));

    test_memory_fs(&fs).await?;
    Ok(())
  }
}
