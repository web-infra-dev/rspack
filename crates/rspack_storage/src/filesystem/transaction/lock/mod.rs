// Lock file structures for transaction management

use std::sync::Arc;

use rspack_paths::Utf8PathBuf;

use crate::{
  FileSystem,
  fs::{FSError, FSOperation, FSResult},
};

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
  fs: Arc<dyn FileSystem>,
}

impl LockHelper {
  /// Create a new lock helper for the given directory
  pub fn new(root_dir: Utf8PathBuf, fs: Arc<dyn FileSystem>) -> Self {
    Self { root_dir, fs }
  }

  // StateLock methods

  /// Read state.lock from the directory
  ///
  /// Returns None if the file doesn't exist or is invalid.
  pub async fn state_lock(&self) -> FSResult<Option<StateLock>> {
    let lock_path = self.root_dir.join(STATE_LOCK_FILE);

    if !self.fs.exists(&lock_path).await? {
      return Ok(None);
    }

    let mut reader = self.fs.read_file(&lock_path).await?;
    let content = String::from_utf8(reader.read_to_end().await?).map_err(|e| {
      FSError::from_message(
        &lock_path,
        FSOperation::Read,
        format!("parse utf8 failed: {e}"),
      )
    })?;

    StateLock::from_string(&content)
      .ok_or_else(|| {
        FSError::from_message(
          &lock_path,
          FSOperation::Read,
          "invalid state.lock format".to_string(),
        )
      })
      .map(Some)
  }

  /// Update state.lock in the directory
  ///
  /// - If `lock` is Some, write/overwrite the state.lock file
  /// - If `lock` is None, remove the state.lock file
  pub async fn update_state_lock(&self, lock: Option<&StateLock>) -> FSResult<()> {
    let lock_path = self.root_dir.join(STATE_LOCK_FILE);

    match lock {
      Some(lock) => {
        // Ensure directory exists
        self.fs.ensure_dir(&self.root_dir).await?;

        let content = lock.to_string();
        let mut writer = self.fs.write_file(&lock_path).await?;
        writer.write_all(content.as_bytes()).await?;
        writer.flush().await?;
      }
      None => {
        // Remove the file if it exists
        if self.fs.exists(&lock_path).await? {
          self.fs.remove_file(&lock_path).await?;
        }
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

    if !self.fs.exists(&lock_path).await? {
      return Ok(None);
    }

    let mut reader = self.fs.read_file(&lock_path).await?;
    let content = String::from_utf8(reader.read_to_end().await?).map_err(|e| {
      FSError::from_message(
        &lock_path,
        FSOperation::Read,
        format!("parse utf8 failed: {e}"),
      )
    })?;

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
        let content = lock.to_string();
        let mut writer = self.fs.write_file(&lock_path).await?;
        writer.write_all(content.as_bytes()).await?;
        writer.flush().await?;
      }
      None => {
        // Remove the file if it exists
        if self.fs.exists(&lock_path).await? {
          self.fs.remove_file(&lock_path).await?;
        }
      }
    }

    Ok(())
  }
}
