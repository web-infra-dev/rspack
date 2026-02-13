use rspack_paths::{Utf8Path, Utf8PathBuf};
use rustc_hash::FxHashSet as HashSet;

use crate::fs::{FSOperation, FSResult, FileSystem, Writer, error::FsResultToStorageFsResult};

mod lock;

use lock::{CommitLock, LockHelper, StateLock};

/// Transaction for atomic file operations
///
/// # Directory Structure
/// All temporary files and locks are stored in `.temp` directory:
/// - `.temp/state.lock`: Process lock, records PID
/// - `.temp/commit.lock`: Commit lock, records all operations (add + remove)
/// - `.temp/bucket1/0.hot.pack`: Temporary data files (mirrors bucket structure)
///
/// # Example
/// ```ignore
/// let tx = Transaction::new(root, fs);
///
/// // Start transaction
/// tx.begin().await?;
///
/// // Add files (Transaction writes to .temp directory)
/// tx.add_file("bucket1/0.hot.pack", content1).await?;
/// tx.add_file("bucket1/0.hot.index", content2).await?;
///
/// // Mark files for deletion
/// tx.remove_file("bucket1/old.pack");
///
/// // Commit all changes (moves from .temp/bucket1/ to bucket1/)
/// tx.commit().await?;
/// ```
#[derive(Debug)]
pub struct Transaction {
  /// Root directory for final files
  root: Utf8PathBuf,
  /// Temporary directory for staging files  
  temp_root: Utf8PathBuf,
  /// Filesystem
  fs: FileSystem,
  /// Lock helper for .temp directory (manages state.lock and commit.lock)
  lock_helper: LockHelper,
  /// Files written to temp (relative paths from root)
  added_files: HashSet<Utf8PathBuf>,
  /// Files to remove from root
  removed_files: HashSet<Utf8PathBuf>,
}

impl Transaction {
  pub async fn recovery(root: Utf8PathBuf, fs: FileSystem) -> FSResult<()> {
    let temp_root = root.join(".temp");
    let lock_helper = LockHelper::new(temp_root.clone(), fs.clone());

    let s = Self {
      root,
      temp_root,
      fs,
      lock_helper,
      added_files: HashSet::default(),
      removed_files: HashSet::default(),
    };

    // Check for existing state.lock and handle recovery
    if let Ok(Some(state_lock)) = s.lock_helper.state_lock().await {
      if state_lock.is_running() {
        // Process is alive, this is a conflict
        panic!(
          "Transaction already in progress by process {} at {}",
          state_lock.pid, s.root
        );
      } else {
        // Process is dead, check for commit.lock
        if let Ok(Some(commit_lock)) = s.lock_helper.commit_lock().await {
          // Recover the commit operation
          s.added_files.extend(commit_lock.files_to_add);
          s.removed_files.extend(commit_lock.files_to_remove);
          //            s.added_files = commit_lock.
          s.execute_commit().await?;
        }
      }
    }
  }

  /// Begin transaction with recovery logic
  ///
  /// Creates state.lock in .temp directory with current process info. If state.lock
  /// already exists, it will check if the process is still running and handle accordingly.
  ///
  /// # Recovery Logic
  /// 1. Check if state.lock exists in .temp directory
  /// 2. If exists and process is running:
  ///    - Panic (another process or thread is using this DB)
  /// 3. If exists but process not running:
  ///    - If commit.lock exists -> recover commit operation
  ///    - Otherwise -> clean up temp directory
  /// 4. If state.lock doesn't exist -> clean up temp directory
  pub async fn new(root: Utf8PathBuf, fs: FileSystem) -> FSResult<Self> {
    let temp_root = root.join(".temp");
    fs.ensure_dir(&temp_root).await?;

    let lock_helper = LockHelper::new(temp_root.clone(), fs.clone());

    Ok(Self {
      root,
      temp_root,
      fs,
      lock_helper,
      added_files: HashSet::default(),
      removed_files: HashSet::default(),
    })
  }

  /// Helper to move a file from temp workspace to root workspace
  async fn move_from_temp_to_root(&self, path: &Utf8Path) -> FSResult<()> {
    let temp_abs = self.temp_root.join(path);
    let root_abs = self.root.join(path);

    self.fs.move_file(&temp_abs, &root_abs).await?;
    Ok(())
  }

  /// Add a file to the transaction
  pub async fn add_file(&mut self, path: impl AsRef<Utf8Path>, content: &[u8]) -> FSResult<()> {
    let path = path.as_ref();
    let temp_path = self.temp_root.join(path);

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
  pub fn remove_file(&mut self, path: impl AsRef<Utf8Path>) {
    // Will be checked in commit()
    self.removed_files.insert(path.as_ref().to_path_buf());
  }

  /// Commit the transaction
  ///
  /// 1. Validates state.lock matches current process (panics if not)
  /// 2. Writes commit.lock to .temp directory with all operations
  /// 3. Moves new files from .temp to root
  /// 4. Deletes old files from root
  /// 5. Removes commit.lock from .temp
  /// 6. Removes state.lock from .temp (transaction complete)
  pub async fn commit(self) -> FSResult<()> {
    // Read and validate state lock
    let state_lock = self
      .lock_helper
      .state_lock()
      .await?
      .expect("state.lock should exist - did you call begin()?");

    // Panic if not current process (race condition detected)
    // This prevents the case where:
    //   T1: Process A checks state.lock -> not exists
    //   T2: Process B checks state.lock -> not exists
    //   T3: Process A creates state.lock (PID=100)
    //   T4: Process B creates state.lock (PID=200) <- overwrites A's lock
    //   T5: Process A commit() -> detects PID mismatch -> panic
    if !state_lock.is_current() {
      panic!(
        "state.lock mismatch: expected current process (pid={}), found pid={}. \
         This indicates a race condition between multiple processes.",
        std::process::id(),
        state_lock.pid
      );
    }

    // Write commit.lock to .temp directory (ensures atomic record)
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

    // Remove commit lock from .temp
    self.lock_helper.update_commit_lock(None).await?;

    // Remove state lock from .temp (transaction complete)
    self.lock_helper.update_state_lock(None).await?;

    // Clear tracked files
    self.added_files.clear();
    self.removed_files.clear();

    Ok(())
  }

  // TODO 检查要前置，因为有可能会触发读取操作，但是这个时候有上次的脏数据
  async fn clean(&mut self) {
    let _ = self.fs.remove_dir(&self.temp_root).await;
    self.fs.ensure_dir(&self.temp_root).await?;
    // Clear tracked files
    self.added_files.clear();
    self.removed_files.clear();
  }

  /// Execute the actual commit operations
  async fn execute_commit(&self) -> FSResult<()> {
    // Move new files from temp to root first
    for path in &self.added_files {
      self.move_from_temp_to_root(path).await?;
    }

    // Then delete old files
    for path in &self.removed_files {
      let root_path = self.root.join(path);
      let _ = self.fs.remove_file(&root_path).await;
    }

    // Clean up temp directory
    let _ = self.fs.remove_dir(&self.temp_root).await;

    Ok(())
  }
}
