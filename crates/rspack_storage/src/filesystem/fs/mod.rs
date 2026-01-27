use std::sync::Arc;

mod error;
mod reader;
pub mod transaction;
mod writer;

use error::FsResultToStorageFsResult;
pub use error::{BatchFSError, BatchFSResult, FSError, FSOperation, FSResult};
pub use reader::Reader;
use rspack_fs::{FileMetadata, IntermediateFileSystem};
use rspack_paths::{Utf8Path, Utf8PathBuf};
use rustc_hash::FxHashSet as HashSet;
use tokio::sync::{Mutex, RwLock};
pub use transaction::Transaction;
pub use writer::Writer;

#[derive(Debug, Clone)]
pub struct FileSystem {
  inner: Arc<dyn IntermediateFileSystem>,
  root: Utf8PathBuf,
  transaction: Arc<Mutex<Transaction>>,
  commit_lock: Arc<RwLock<()>>,
}

impl FileSystem {
  pub fn new(inner: Arc<dyn IntermediateFileSystem>, root: Utf8PathBuf) -> Self {
    let transaction = Transaction::new(root.clone(), inner.clone());
    Self {
      inner,
      root,
      transaction: Arc::new(Mutex::new(transaction)),
      commit_lock: Arc::new(RwLock::new(())),
    }
  }

  /// Get access to the inner filesystem (bypasses transaction/commit locks)
  ///
  /// This is used internally by the transaction system to avoid deadlocks
  pub(crate) fn inner(&self) -> &Arc<dyn IntermediateFileSystem> {
    &self.inner
  }

  pub async fn begin_transaction(&self) -> FSResult<()> {
    let mut tx_guard = self.transaction.lock().await;
    tx_guard.begin().await?;
    Ok(())
  }

  pub async fn commit_transaction(&self) -> FSResult<()> {
    // Acquire write lock to block all reads during commit
    let _write_lock = self.commit_lock.write().await;

    let mut tx_guard = self.transaction.lock().await;
    tx_guard.commit().await?;
    Ok(())
  }

  /// Get the temp root path for the active transaction
  pub fn transaction_temp_root(&self) -> Utf8PathBuf {
    self.root.join(".temp")
  }

  /// Add a file to the active transaction
  pub async fn transaction_add_file(
    &self,
    path: impl AsRef<Utf8Path>,
    content: &[u8],
  ) -> FSResult<()> {
    let mut tx_guard = self.transaction.lock().await;
    tx_guard.add_file(path, content).await?;
    Ok(())
  }

  /// Add a file that already exists in temp directory to the active transaction
  pub async fn transaction_add_file_from_temp(&self, path: impl AsRef<Utf8Path>) -> FSResult<()> {
    let mut tx_guard = self.transaction.lock().await;
    tx_guard.add_file_from_temp(path);
    Ok(())
  }

  /// Mark a file for removal in the active transaction
  pub async fn transaction_remove_file(&self, path: impl AsRef<Utf8Path>) -> FSResult<()> {
    let mut tx_guard = self.transaction.lock().await;
    tx_guard.remove_file(path);
    Ok(())
  }

  pub async fn exists(&self, path: &Utf8Path) -> FSResult<bool> {
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

  /// Internal exists that bypasses commit lock (for use by transaction system)
  pub(crate) async fn exists_internal(&self, path: &Utf8Path) -> FSResult<bool> {
    match self.metadata_internal(path).await {
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

  pub async fn remove_dir(&self, path: &Utf8Path) -> FSResult<()> {
    if self.exists(path).await? {
      self
        .inner
        .remove_dir_all(path)
        .await
        .to_storage_fs_result(path, FSOperation::Remove)?;
    }
    Ok(())
  }

  pub async fn ensure_dir(&self, path: &Utf8Path) -> FSResult<()> {
    self
      .inner
      .create_dir_all(path)
      .await
      .to_storage_fs_result(path, FSOperation::Dir)?;
    Ok(())
  }

  pub async fn write_file(&self, path: &Utf8Path) -> FSResult<Writer> {
    if self.exists(path).await? {
      self.remove_file(path).await?;
    }
    self
      .ensure_dir(path.parent().expect("should have parent"))
      .await?;

    let stream = self
      .inner
      .create_write_stream(path)
      .await
      .to_storage_fs_result(path, FSOperation::Write)?;

    Ok(Writer {
      path: path.to_path_buf(),
      stream,
    })
  }

  pub async fn read_file(&self, path: &Utf8Path) -> FSResult<Reader> {
    // Acquire read lock to wait for any commit
    let _read_lock = self.commit_lock.read().await;

    self.read_file_internal(path).await
  }

  /// Internal read_file that bypasses commit lock (for use by transaction system)
  pub(crate) async fn read_file_internal(&self, path: &Utf8Path) -> FSResult<Reader> {
    let stream = self
      .inner
      .create_read_stream(path)
      .await
      .to_storage_fs_result(path, FSOperation::Read)?;
    Ok(Reader {
      path: path.to_path_buf(),
      stream,
    })
  }

  pub async fn read_dir(&self, path: &Utf8Path) -> FSResult<HashSet<String>> {
    // Acquire read lock to wait for any commit
    let _read_lock = self.commit_lock.read().await;

    let files = self
      .inner
      .read_dir(path)
      .await
      .to_storage_fs_result(path, FSOperation::Read)?;
    Ok(files.into_iter().collect::<HashSet<_>>())
  }

  pub async fn metadata(&self, path: &Utf8Path) -> FSResult<FileMetadata> {
    // Acquire read lock to wait for any commit
    let _read_lock = self.commit_lock.read().await;

    self.metadata_internal(path).await
  }

  /// Internal metadata that bypasses commit lock (for use by transaction system)
  pub(crate) async fn metadata_internal(&self, path: &Utf8Path) -> FSResult<FileMetadata> {
    let res = self
      .inner
      .stat(path)
      .await
      .to_storage_fs_result(path, FSOperation::Stat)?;
    Ok(res)
  }

  pub async fn remove_file(&self, path: &Utf8Path) -> FSResult<()> {
    if self.exists(path).await? {
      self
        .inner
        .remove_file(path)
        .await
        .to_storage_fs_result(path, FSOperation::Remove)?;
    }
    Ok(())
  }

  pub async fn move_file(&self, from: &Utf8Path, to: &Utf8Path) -> FSResult<()> {
    if self.exists(from).await? {
      self
        .ensure_dir(to.parent().expect("should have parent"))
        .await?;
      self
        .inner
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

  use super::{FSResult, FileSystem};

  fn get_path(p: &str) -> Utf8PathBuf {
    Utf8PathBuf::from(p)
  }

  async fn test_create_dir(fs: &FileSystem) -> FSResult<()> {
    fs.ensure_dir(&get_path("/parent/from")).await?;
    fs.ensure_dir(&get_path("/parent/to")).await?;

    assert!(fs.exists(&get_path("/parent/from")).await?);
    assert!(fs.exists(&get_path("/parent/to")).await?);

    assert!(fs.metadata(&get_path("/parent/from")).await?.is_directory);
    assert!(fs.metadata(&get_path("/parent/to")).await?.is_directory);

    Ok(())
  }

  async fn test_write_file(fs: &FileSystem) -> FSResult<()> {
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

  async fn test_read_file(fs: &FileSystem) -> FSResult<()> {
    let mut reader = fs.read_file(&get_path("/parent/from/file.txt")).await?;

    assert_eq!(reader.read_line().await?, "hello");
    assert_eq!(reader.read(b" world".len()).await?, b" world");

    Ok(())
  }

  async fn test_move_file(fs: &FileSystem) -> FSResult<()> {
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

  async fn test_remove_file(fs: &FileSystem) -> FSResult<()> {
    fs.remove_file(&get_path("/parent/to/file.txt")).await?;
    assert!(!fs.exists(&get_path("/parent/to/file.txt")).await?);
    Ok(())
  }

  async fn test_remove_dir(fs: &FileSystem) -> FSResult<()> {
    fs.remove_dir(&get_path("/parent/from")).await?;
    fs.remove_dir(&get_path("/parent/to")).await?;
    assert!(!fs.exists(&get_path("/parent/from")).await?);
    assert!(!fs.exists(&get_path("/parent/to")).await?);
    Ok(())
  }

  async fn test_error(fs: &FileSystem) -> FSResult<()> {
    match fs.metadata(&get_path("/parent/from/not_exist.txt")).await {
      Ok(_) => panic!("should error"),
      Err(e) => assert_eq!(
        e.to_string(),
        r#"stat `/parent/from/not_exist.txt` failed due to `file not exist`"#
      ),
    };

    Ok(())
  }

  async fn test_memory_fs(fs: &FileSystem) -> FSResult<()> {
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
    let fs = FileSystem::new(Arc::new(MemoryFileSystem::default()));

    test_memory_fs(&fs).await?;
    Ok(())
  }
}
