// Transaction module - Simplified atomic file operations
//
// This module provides a simple transaction mechanism for file operations:
// 1. begin() - Create state.lock with process info
// 2. add_file() - Write file to temp directory
// 3. remove_file() - Mark file for deletion
// 4. commit() - Create commit.lock and execute all operations

use std::sync::Arc;

use rspack_paths::{Utf8Path, Utf8PathBuf};
use rustc_hash::FxHashSet as HashSet;

use crate::{FileSystem, fs::FSResult};

mod lock;

use lock::{CommitLock, LockHelper, StateLock};

/// Transaction for atomic file operations
///
/// # Lock Files
/// - `state.lock`: Process lock, created in begin() in root directory, records PID
/// - `commit.lock`: Commit lock, created in commit() in root directory, records all operations (add + remove)
///
/// # Example
/// ```ignore
/// let mut tx = Transaction::new(root, temp_root, fs).await;
///
/// // Start transaction
/// tx.begin().await?;
///
/// // Add files (Transaction writes to temp directory)
/// tx.add_file("scope/file1.pack", content1).await?;
/// tx.add_file("scope/file2.pack", content2).await?;
///
/// // Mark files for deletion
/// tx.remove_file("scope/old.pack");
///
/// // Commit all changes
/// tx.commit().await?;
/// ```
#[derive(Debug)]
pub struct Transaction {
  /// Root directory for final files
  root: Utf8PathBuf,
  /// Temporary directory for staging files
  temp_root: Utf8PathBuf,
  /// File system abstraction
  fs: Arc<dyn FileSystem>,
  /// Lock helper for root directory (manages state.lock and commit.lock)
  lock_helper: LockHelper,
  /// Files written to temp (relative paths from root)
  added_files: HashSet<Utf8PathBuf>,
  /// Files to remove from root
  removed_files: HashSet<Utf8PathBuf>,
}

impl Transaction {
  /// Create a new transaction with recovery logic
  ///
  /// # Recovery Logic
  /// 1. Check if state.lock exists in root directory
  /// 2. If exists and process is running -> panic (another process is using it)
  /// 3. If exists but process not running -> try to recover from commit.lock
  ///    - If commit.lock exists in root -> load files from commit.lock and continue
  ///    - Otherwise -> clean up temp directory
  /// 4. If state.lock doesn't exist -> clean up temp directory
  pub async fn new(root: Utf8PathBuf, temp_root: Utf8PathBuf, fs: Arc<dyn FileSystem>) -> Self {
    let lock_helper = LockHelper::new(root.clone(), fs.clone());

    let mut instance = Self {
      root: root.clone(),
      temp_root: temp_root.clone(),
      fs: fs.clone(),
      lock_helper: lock_helper.clone(),
      added_files: HashSet::default(),
      removed_files: HashSet::default(),
    };

    // Check for existing state.lock in root
    let should_cleanup = if let Ok(Some(state_lock)) = lock_helper.state_lock().await {
      // state.lock exists, check if process is running
      if state_lock.is_running() {
        panic!(
          "Transaction already in progress by process {} at {}",
          state_lock.pid, root
        );
      }

      // Process not running, check for commit.lock
      if let Ok(Some(commit_lock)) = lock_helper.commit_lock().await {
        // Load files from commit.lock into the transaction
        instance.added_files = commit_lock.files_to_add.into_iter().collect();
        instance.removed_files = commit_lock.files_to_remove.into_iter().collect();
        false // Don't cleanup, we have files to commit
      } else {
        true // No commit.lock, cleanup needed
      }
    } else {
      // No state.lock, cleanup temp directory
      true
    };

    if should_cleanup {
      let _ = instance.fs.remove_dir(&temp_root).await;
    }

    instance
  }

  /// Begin transaction
  ///
  /// Creates state.lock with current process info. If state.lock already exists,
  /// it will be forcibly overwritten.
  ///
  /// Clears any existing added_files and removed_files from previous incomplete transactions.
  ///
  /// # Lock format (state.lock)
  /// ```text
  /// pid
  /// info
  /// ```
  pub async fn begin(&mut self) -> FSResult<()> {
    // Create state lock with current process info in root directory
    let state_lock = StateLock::default();
    self
      .lock_helper
      .update_state_lock(Some(&state_lock))
      .await?;

    // Clear any existing file tracking and ensure temp directory is clean
    self.added_files.clear();
    self.removed_files.clear();
    let _ = self.fs.remove_dir(&self.temp_root).await;
    self.fs.ensure_dir(&self.temp_root).await?;

    Ok(())
  }

  /// Add a file to the transaction
  ///
  /// The file content will be written to the temp directory immediately.
  /// The path is relative to root.
  ///
  /// # Arguments
  /// * `path` - Relative path from root (e.g., "scope1/0/pack_0.pack")
  /// * `content` - File content bytes
  ///
  /// # Example
  /// ```ignore
  /// tx.add_file("scope1/file.pack", b"content").await?;
  /// // File written to: temp_root/scope1/file.pack
  /// ```
  pub async fn add_file(&mut self, path: impl AsRef<Utf8Path>, content: &[u8]) -> FSResult<()> {
    let path = path.as_ref();
    let temp_path = self.temp_root.join(path);

    // Ensure parent directory exists
    if let Some(parent) = temp_path.parent() {
      self.fs.ensure_dir(parent).await?;
    }

    // Write file to temp
    let mut writer = self.fs.write_file(&temp_path).await?;
    writer.write_all(content).await?;
    writer.flush().await?;

    // Track this file and remove from removed_files if it was marked for deletion
    self.added_files.insert(path.to_path_buf());
    self.removed_files.remove(path);

    Ok(())
  }

  /// Mark a file for removal
  ///
  /// The file will be deleted during commit. The path is relative to root.
  ///
  /// # Arguments
  /// * `path` - Relative path from root (e.g., "scope1/0/pack_old.pack")
  pub fn remove_file(&mut self, path: impl AsRef<Utf8Path>) {
    // Will be checked in commit()
    self.removed_files.insert(path.as_ref().to_path_buf());
  }

  /// Commit the transaction
  ///
  /// 1. Validates state.lock matches current process (panics if not)
  /// 2. Writes commit.lock to root directory with all operations
  /// 3. Moves new files from temp to root
  /// 4. Deletes old files from root
  /// 5. Removes commit.lock from root
  /// 6. Keeps state.lock
  ///
  /// # Lock format (commit.lock in root)
  /// ```text
  /// [ADD]
  /// file1
  /// file2
  /// [REMOVE]
  /// file3
  /// file4
  /// ```
  pub async fn commit(&mut self) -> FSResult<()> {
    // Read and validate state lock
    let state_lock = self
      .lock_helper
      .state_lock()
      .await?
      .expect("state.lock should exist - did you call begin()?");

    // Panic if not current process
    if !state_lock.is_current() {
      // TODO mark cache dirty
      panic!(
        "state.lock mismatch: expected current process (pid={}), found pid={}",
        std::process::id(),
        state_lock.pid
      );
    }

    // Write commit.lock to root directory (ensures atomic record)
    let commit_lock = CommitLock::new(
      self.added_files.iter().cloned().collect(),
      self.removed_files.iter().cloned().collect(),
    );
    self
      .lock_helper
      .update_commit_lock(Some(&commit_lock))
      .await?;

    // Execute operations
    self.execute_commit().await?;

    // Remove commit lock from root
    self.lock_helper.update_commit_lock(None).await?;

    // Clear tracked files
    self.added_files.clear();
    self.removed_files.clear();

    Ok(())
  }

  /// Execute the actual commit operations
  async fn execute_commit(&self) -> FSResult<()> {
    // Move new files from temp to root first
    for path in &self.added_files {
      let temp_path = self.temp_root.join(path);
      let root_path = self.root.join(path);

      // Ensure parent directory exists in root
      if let Some(parent) = root_path.parent() {
        self.fs.ensure_dir(parent).await?;
      }

      self.fs.move_file(&temp_path, &root_path).await?;
    }

    // Then delete old files
    for path in &self.removed_files {
      let full_path = self.root.join(path);
      let _ = self.fs.remove_file(&full_path).await;
    }

    // Clean up temp directory
    let _ = self.fs.remove_dir(&self.temp_root).await;

    Ok(())
  }

  /// Get root directory
  pub fn root(&self) -> &Utf8Path {
    &self.root
  }

  /// Get temp directory
  pub fn temp_root(&self) -> &Utf8Path {
    &self.temp_root
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  // Tests require FileSystem mock
}
