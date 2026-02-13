mod lock;

use self::lock::{CommitLock, StateLock};
use super::{Error, Result};
use crate::fs::ScopeFileSystem;

/// Transaction for atomic file operations with crash recovery.
///
/// Implements a two-phase commit protocol:
/// 1. Write changes to a temporary directory (.temp)
/// 2. On commit, atomically move files to final location
///
/// Uses lock files for concurrency control and crash recovery:
/// - state.lock: Tracks which process owns the transaction
/// - commit.lock: Records pending operations for recovery
#[derive(Debug)]
pub struct Transaction {
  /// Root directory for final files
  root_fs: ScopeFileSystem,
  /// Temporary directory for staging files  
  temp_fs: ScopeFileSystem,
}

impl Transaction {
  /// Creates a new transaction, performing crash recovery if needed.
  pub async fn new(root_fs: &ScopeFileSystem) -> Result<Self> {
    let root_fs = root_fs.clone();
    let temp_fs = root_fs.child_fs(".temp");

    let transaction = Self { root_fs, temp_fs };
    transaction.init().await?;

    Ok(transaction)
  }

  /// Initializes the transaction, checking for stale locks and recovering crashed commits.
  async fn init(&self) -> Result<()> {
    // Try to load existing state lock
    let state_lock = match StateLock::load(&self.temp_fs).await {
      Ok(lock) => Some(lock),
      Err(e) if e.is_not_found() => None,
      Err(e) => return Err(e),
    };

    if let Some(state_lock) = state_lock {
      if state_lock.is_running() {
        // Another process is actively using this transaction
        panic!(
          "Transaction already in progress by process {} in directory '{}'",
          state_lock, self.temp_fs
        );
      } else {
        // Process crashed - attempt to recover
        let commit_lock = match CommitLock::load(&self.temp_fs).await {
          Ok(lock) => Some(lock),
          Err(e) if e.is_not_found() => None,
          Err(e) => return Err(e),
        };

        if let Some(commit_lock) = commit_lock {
          // Recover incomplete commit
          self.execute_commit(&commit_lock).await?;
        }
      }
    }

    // Clean up temp directory and prepare for new transaction
    self.temp_fs.remove().await?;
    self.temp_fs.ensure_exist().await?;

    // Create state lock for current process
    let state_lock = StateLock::default();
    state_lock.save(&self.temp_fs).await
  }

  /// Returns the readable filesystem (committed files).
  pub fn readable_fs(&self) -> &ScopeFileSystem {
    &self.root_fs
  }

  /// Returns the writable filesystem (staged files in temp directory).
  pub fn writable_fs(&self) -> &ScopeFileSystem {
    &self.temp_fs
  }

  /// Commits the transaction, atomically applying all changes.
  ///
  /// # Arguments
  /// * `added_relative_path` - Files to move from temp to root
  /// * `removed_relative_path` - Files to delete from root
  pub async fn commit(
    self,
    added_relative_path: Vec<String>,
    removed_relative_path: Vec<String>,
  ) -> Result<()> {
    // Verify this transaction still belongs to current process
    let state_lock = StateLock::load(&self.temp_fs).await?;

    if !state_lock.is_current() {
      panic!(
        "State lock mismatch in '{}': expected current process (pid={}), found {}. \
         This indicates a race condition.",
        self.temp_fs,
        std::process::id(),
        state_lock
      );
    }

    // Write commit lock to record operations (for crash recovery)
    let commit_lock = CommitLock::new(added_relative_path, removed_relative_path);
    commit_lock.save(&self.temp_fs).await?;

    // Execute the commit operations
    self.execute_commit(&commit_lock).await?;

    // Clean up temp directory
    self.temp_fs.remove().await?;
    Ok(())
  }

  /// Executes the actual commit operations (used by both commit and recovery).
  async fn execute_commit(&self, commit_lock: &CommitLock) -> Result<()> {
    // Delete removed files from root
    for path in commit_lock.removed_files() {
      self.root_fs.remove_file(path).await?;
    }

    // Move added files from temp to root
    for path in commit_lock.added_files() {
      ScopeFileSystem::move_to(self.temp_fs.clone(), self.root_fs.clone(), path).await?;
    }

    Ok(())
  }
}

#[cfg(test)]
mod test {
  use super::{Result, Transaction};
  use crate::fs::ScopeFileSystem;
  #[tokio::test]
  #[cfg_attr(miri, ignore)]
  async fn test_smoke() -> Result<()> {
    let fs = ScopeFileSystem::new_memory_fs("/".into());
    fs.ensure_exist().await?;
    fs.write("a.txt", "a".as_bytes()).await?;
    fs.write("b.txt", "b".as_bytes()).await?;
    fs.write("c.txt", "c".as_bytes()).await?;
    assert!(fs.stat("/.temp").await.is_err());

    let transaction = Transaction::new(&fs).await?;
    assert!(fs.stat("/.temp").await.is_ok());

    // test read & write
    transaction
      .writable_fs()
      .write("a.txt", "aa".as_bytes())
      .await?;
    transaction
      .writable_fs()
      .write("d.txt", "dd".as_bytes())
      .await?;
    let readable_fs = transaction.readable_fs();
    assert_eq!(readable_fs.read("a.txt").await?, "a".as_bytes());
    assert_eq!(readable_fs.read("b.txt").await?, "b".as_bytes());
    assert_eq!(readable_fs.read("c.txt").await?, "c".as_bytes());
    assert!(readable_fs.read("d.txt").await.is_err());

    let writable_fs = transaction.writable_fs();
    assert_eq!(writable_fs.read("a.txt").await?, "aa".as_bytes());
    assert!(writable_fs.read("b.txt").await.is_err());
    assert!(writable_fs.read("c.txt").await.is_err());
    assert_eq!(writable_fs.read("d.txt").await?, "dd".as_bytes());

    transaction
      .commit(vec!["a.txt".into(), "d.txt".into()], vec!["b.txt".into()])
      .await?;

    assert!(fs.stat("/.temp").await.is_err());
    assert_eq!(fs.read("a.txt").await?, "aa".as_bytes());
    assert_eq!(fs.read("c.txt").await?, "c".as_bytes());
    assert_eq!(fs.read("d.txt").await?, "dd".as_bytes());

    Ok(())
  }
}
