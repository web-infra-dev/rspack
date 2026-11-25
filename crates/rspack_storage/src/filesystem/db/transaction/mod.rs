mod lock;

use std::sync::LazyLock;

use rspack_fs::RelPath;

use self::lock::{CommitLock, StateLock};
use super::ScopeFileSystem;
use crate::Result;

fn temp_dir_path() -> &'static RelPath {
  static PATH: LazyLock<RelPath> =
    LazyLock::new(|| RelPath::new_contained(".temp").expect("valid temp dir path"));
  &PATH
}

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
  /// Creates a new transaction.
  pub async fn new(root_fs: &ScopeFileSystem) -> Result<Self> {
    // Replay any interrupted commit from a previous crashed process.
    Self::ensure_committed(root_fs).await?;

    let root_fs = root_fs.clone();
    let temp_fs = root_fs.child_fs(temp_dir_path())?;
    temp_fs.ensure_exist().await?;

    // Create state lock for current process
    let state_lock = StateLock::default();
    state_lock.save(&temp_fs).await?;

    Ok(Self { root_fs, temp_fs })
  }

  /// Ensures any previously interrupted commit is fully applied before reading.
  pub async fn ensure_committed(root_fs: &ScopeFileSystem) -> Result<()> {
    let temp_fs = root_fs.child_fs(temp_dir_path())?;
    let state_lock = match StateLock::load(&temp_fs).await {
      Ok(lock) => Some(lock),
      Err(e) if e.is_not_found() => None,
      Err(e) => return Err(e),
    };
    if let Some(state_lock) = state_lock
      && state_lock.is_running()
    {
      // Another process is actively using this transaction
      panic!("Transaction already in progress by process {state_lock} in directory '{temp_fs}'",);
    }

    // Process crashed — check for a pending commit.lock
    let commit_lock = match CommitLock::load(&temp_fs).await {
      Ok(lock) => Some(lock),
      Err(e) if e.is_not_found() => None,
      Err(e) => return Err(e),
    };
    if let Some(commit_lock) = commit_lock {
      let transaction = Self {
        root_fs: root_fs.clone(),
        temp_fs: temp_fs.clone(),
      };
      transaction.execute_commit(&commit_lock).await?;
      // Clean up temp directory
      temp_fs.remove().await?;
    }

    Ok(())
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
    added_relative_path: Vec<RelPath>,
    removed_relative_path: Vec<RelPath>,
  ) -> Result<()> {
    // Verify this transaction still belongs to current process
    match StateLock::load(&self.temp_fs).await {
      Ok(state_lock) if state_lock.is_current() => {}
      Ok(state_lock) => panic!(
        "State lock mismatch in '{}': expected current process (pid={}), found {}. \
         This indicates a race condition.",
        self.temp_fs,
        std::process::id(),
        state_lock
      ),
      Err(e) => {
        // Failed to read state lock — rollback and surface the error.
        let _ = self.temp_fs.remove().await;
        return Err(e);
      }
    }

    // Write commit lock to record operations (for crash recovery)
    let commit_lock = CommitLock::new(added_relative_path, removed_relative_path);
    if let Err(e) = commit_lock.save(&self.temp_fs).await {
      // Failed to write commit lock - rollback and surface the error.
      let _ = self.temp_fs.remove().await;
      return Err(e);
    }

    // Execute the commit operations
    if let Err(e) = self.execute_commit(&commit_lock).await {
      // Failed to commit, remove state lock
      let _ = StateLock::remove(&self.temp_fs).await;
      return Err(e);
    }

    // Clean up temp directory and ignore error
    let _ = self.temp_fs.remove().await;
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
      ScopeFileSystem::move_to(&self.temp_fs, &self.root_fs, path).await?;
    }

    Ok(())
  }

  /// Rollback the transaction, discard all changes.
  pub async fn rollback(self) -> Result<()> {
    self.temp_fs.remove().await
  }
}

#[cfg(test)]
mod test {
  use rspack_fs::RelPath;

  use super::{Result, ScopeFileSystem, Transaction};

  fn rel(path: &str) -> RelPath {
    RelPath::new_contained(path).expect("valid test relative path")
  }

  #[tokio::test]
  #[cfg_attr(miri, ignore)]
  async fn test_smoke() -> Result<()> {
    let fs = ScopeFileSystem::new_memory_fs("/");
    fs.ensure_exist().await?;
    fs.write(&rel("a.txt"), "a".as_bytes()).await?;
    fs.write(&rel("b.txt"), "b".as_bytes()).await?;
    fs.write(&rel("c.txt"), "c".as_bytes()).await?;
    assert!(fs.stat(&rel(".temp")).await.is_err());

    let transaction = Transaction::new(&fs).await?;
    assert!(fs.stat(&rel(".temp")).await.is_ok());

    // test read & write
    transaction
      .writable_fs()
      .write(&rel("a.txt"), "aa".as_bytes())
      .await?;
    transaction
      .writable_fs()
      .write(&rel("d.txt"), "dd".as_bytes())
      .await?;
    let readable_fs = transaction.readable_fs();
    assert_eq!(readable_fs.read(&rel("a.txt")).await?, "a".as_bytes());
    assert_eq!(readable_fs.read(&rel("b.txt")).await?, "b".as_bytes());
    assert_eq!(readable_fs.read(&rel("c.txt")).await?, "c".as_bytes());
    assert!(readable_fs.read(&rel("d.txt")).await.is_err());

    let writable_fs = transaction.writable_fs();
    assert_eq!(writable_fs.read(&rel("a.txt")).await?, "aa".as_bytes());
    assert!(writable_fs.read(&rel("b.txt")).await.is_err());
    assert!(writable_fs.read(&rel("c.txt")).await.is_err());
    assert_eq!(writable_fs.read(&rel("d.txt")).await?, "dd".as_bytes());

    transaction
      .commit(vec![rel("a.txt"), rel("d.txt")], vec![rel("b.txt")])
      .await?;

    assert!(fs.stat(&rel(".temp")).await.is_err());
    assert_eq!(fs.read(&rel("a.txt")).await?, "aa".as_bytes());
    assert_eq!(fs.read(&rel("c.txt")).await?, "c".as_bytes());
    assert_eq!(fs.read(&rel("d.txt")).await?, "dd".as_bytes());

    Ok(())
  }
}
