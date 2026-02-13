mod lock;

use self::lock::{CommitLock, StateLock};
use super::{Error, Result};
use crate::fs::ScopeFileSystem;

/// Transaction for atomic file operations
#[derive(Debug)]
pub struct Transaction {
  /// Root directory for final files
  pub root_fs: ScopeFileSystem,
  /// Temporary directory for staging files  
  pub temp_fs: ScopeFileSystem,
}

impl Transaction {
  pub fn new(root_fs: &ScopeFileSystem) -> Self {
    let root_fs = root_fs.clone();
    let temp_fs = root_fs.child_fs(".temp");

    Self { root_fs, temp_fs }
  }

  pub async fn init(&self) -> Result<()> {
    // TODO check init has been run

    self.root_fs.ensure_exist().await?;
    self.temp_fs.ensure_exist().await?;
    // Check for existing state.lock in root_fs and handle recovery
    // TODO one nodejs probably run multi compiler, remove state.lock after finish, remove to temp dir
    /*if let Some(state_lock) = StateLock::load(&self.root_fs).await? {
      if state_lock.is_running() {
        // Process is alive, this is a conflict
        panic!(
          "Transaction already in progress by process {} at {}",
          state_lock, self.root_fs
        );
      } else {
        // Process is dead, check for commit.lock in temp_fs
        if let Some(commit_lock) = CommitLock::load(&self.temp_fs).await? {
          // Recover the commit operation
          self.execute_commit(&commit_lock).await?;
        }
      }
    }*/
    // Clean up temp directory
    self.clean().await?;

    // Create state.lock for current process in root_fs
    let state_lock = StateLock::default();
    state_lock.save(&self.root_fs).await
  }

  /// Commit the transaction
  pub async fn commit(
    &self,
    added_relative_path: Vec<String>,
    removed_relative_path: Vec<String>,
  ) -> Result<()> {
    // Read and validate state lock from root_fs
    let state_lock = StateLock::load(&self.root_fs)
      .await?
      .expect("state.lock should exist - did you call init()?");

    if !state_lock.is_current() {
      panic!(
        "state.lock mismatch: expected current process (pid={}), found pid={}. \
         This indicates a race condition between multiple processes.",
        std::process::id(),
        state_lock
      );
    }

    // Write commit.lock to temp_fs (ensures atomic record)
    let commit_lock = CommitLock::new(added_relative_path, removed_relative_path);
    commit_lock.save(&self.temp_fs).await?;

    // Execute operations
    self.execute_commit(&commit_lock).await?;

    // Clean up temp directory
    self.clean().await
  }

  /// Execute the actual commit operations
  async fn execute_commit(&self, commit_lock: &CommitLock) -> Result<()> {
    // Delete old files
    for path in commit_lock.removed_files() {
      self.root_fs.remove_file(path).await?;
    }

    // Move new files from temp to root first
    for path in commit_lock.added_files() {
      ScopeFileSystem::move_to(self.temp_fs.clone(), self.root_fs.clone(), path).await?;
    }

    Ok(())
  }

  async fn clean(&self) -> Result<()> {
    self.temp_fs.clean().await?;
    Ok(())
  }
}
