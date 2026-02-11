// Lock file structures for transaction management

use std::sync::Arc;

use futures::AsyncWriteExt;
use rspack_fs::IntermediateFileSystem;
use rspack_paths::Utf8PathBuf;

use crate::fs::{FSError, FSOperation, FSResult, Reader, Writer, error::FsResultToStorageFsResult};

mod commit;
mod state;

pub use commit::CommitLock;
pub use state::StateLock;

const STATE_LOCK_FILE: &str = "state.lock";
const COMMIT_LOCK_FILE: &str = "commit.lock";

/// Helper for reading and writing lock files
///
/// Provides convenient methods to read/write state.lock and commit.lock
/// in a specific directory.
#[derive(Debug, Clone)]
pub struct LockHelper {
  root_dir: Utf8PathBuf,
  fs: Arc<dyn IntermediateFileSystem>,
}

impl LockHelper {
  /// Create a new lock helper for the given directory
  pub fn new(root_dir: Utf8PathBuf, fs: Arc<dyn IntermediateFileSystem>) -> Self {
    Self { root_dir, fs }
  }

  // StateLock methods

  /// Read state.lock from the directory
  ///
  /// Returns None if the file doesn't exist or is invalid.
  pub async fn state_lock(&self) -> FSResult<Option<StateLock>> {
    let lock_path = self.root_dir.join(STATE_LOCK_FILE);

    // Check if file exists
    // TODO use readfile error to check
    match self.fs.stat(&lock_path).await {
      Ok(_) => {}
      Err(_) => return Ok(None),
    }

    // Read file
    let buf = self
      .fs
      .read_file(&lock_path)
      .await
      .to_storage_fs_result(&lock_path, FSOperation::Read)?;
    let content = String::from_utf8_lossy(&buf);
    Ok(StateLock::from_string(&content))
  }

  /// Update state.lock in the directory
  ///
  /// - If `lock` is Some, write/overwrite the state.lock file
  /// - If `lock` is None, remove the state.lock file
  pub async fn update_state_lock(&self, lock: Option<&StateLock>) -> FSResult<()> {
    let lock_path = self.root_dir.join(STATE_LOCK_FILE);

    match lock {
      Some(lock) => {
        // Write file
        self
          .fs
          .write(&lock_path, lock.to_string().as_bytes())
          .await
          .to_storage_fs_result(&lock_path, FSOperation::Write)?;
      }
      None => {
        // Remove the file
        self.fs.remove_file(&lock_path).await;
      }
    }

    Ok(())
  }

  // CommitLock methods

  /// Read commit.lock from the directory
  ///
  /// Returns None if the file doesn't exist or is invalid.
  pub async fn commit_lock(&self) -> FSResult<Option<CommitLock>> {
    let lock_path = self.root_dir.join(COMMIT_LOCK_FILE);

    // Check if file exists
    match self.fs.stat(&lock_path).await {
      Ok(_) => {}
      Err(_) => return Ok(None),
    }

    // Read file
    let buf = self
      .fs
      .read_file(&lock_path)
      .await
      .to_storage_fs_result(&lock_path, FSOperation::Read)?;
    let content = String::from_utf8_lossy(&buf);
    Ok(CommitLock::from_string(&content))
  }

  /// Update commit.lock in the directory
  ///
  /// - If `lock` is Some, write/overwrite the commit.lock file
  /// - If `lock` is None, remove the commit.lock file
  pub async fn update_commit_lock(&self, lock: Option<&CommitLock>) -> FSResult<()> {
    let lock_path = self.root_dir.join(COMMIT_LOCK_FILE);

    match lock {
      Some(lock) => {
        // Write file
        self
          .fs
          .write(&lock_path, lock.to_string().as_bytes())
          .await
          .to_storage_fs_result(&lock_path, FSOperation::Write)?;
      }
      None => {
        // Remove the file
        self.fs.remove_file(&lock_path).await;
      }
    }

    Ok(())
  }
}
