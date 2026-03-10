use super::super::{Error, Result};
use crate::fs::ScopeFileSystem;

const COMMIT_LOCK_FILE: &str = "commit.lock";

/// Commit lock file that records pending file operations in a transaction.
///
/// Format:
/// ```text
/// [ADD]
/// file1.txt
/// file2.txt
/// [REMOVE]
/// file3.txt
/// ```
///
/// Used for crash recovery to ensure atomic commits.
#[derive(Debug, PartialEq, Eq)]
pub struct CommitLock {
  added_files: Vec<String>,
  removed_files: Vec<String>,
}

impl CommitLock {
  pub fn new(added_files: Vec<String>, removed_files: Vec<String>) -> Self {
    Self {
      added_files,
      removed_files,
    }
  }

  /// Loads commit lock from filesystem and returns the pending operations.
  pub async fn load(fs: &ScopeFileSystem) -> Result<Self> {
    let mut reader = fs.read_file(COMMIT_LOCK_FILE).await?;
    let mut added_files = Vec::new();
    let mut removed_files = Vec::new();
    let mut current_section = None;

    while let Ok(line) = reader.read_line().await {
      if line.is_empty() {
        break;
      }
      let line = line.trim();
      if line.is_empty() {
        continue;
      }

      match line {
        "[ADD]" => current_section = Some("add"),
        "[REMOVE]" => current_section = Some("remove"),
        _ => match current_section {
          Some("add") => added_files.push(line.to_string()),
          Some("remove") => removed_files.push(line.to_string()),
          None => {
            return Err(Error::InvalidFormat(format!(
              "Unexpected line in '{}' before section header: '{}'",
              COMMIT_LOCK_FILE, line
            )));
          }
          _ => unreachable!(),
        },
      }
    }

    Ok(Self {
      added_files,
      removed_files,
    })
  }

  /// Saves commit lock to filesystem with pending operations.
  pub async fn save(&self, fs: &ScopeFileSystem) -> Result<()> {
    let mut writer = fs.write_file(COMMIT_LOCK_FILE).await?;

    writer.write_line("[ADD]").await?;
    for file in &self.added_files {
      writer.write_line(file).await?;
    }

    writer.write_line("[REMOVE]").await?;
    for file in &self.removed_files {
      writer.write_line(file).await?;
    }

    writer.flush().await?;
    Ok(())
  }

  /// Returns the list of files to be removed.
  pub fn removed_files(&self) -> &[String] {
    &self.removed_files
  }

  /// Returns the list of files to be added.
  pub fn added_files(&self) -> &[String] {
    &self.added_files
  }
}

#[cfg(test)]
mod tests {
  use super::{CommitLock, Result};
  use crate::fs::ScopeFileSystem;

  #[tokio::test]
  #[cfg_attr(miri, ignore)]
  async fn test_commit_lock() -> Result<()> {
    let fs = ScopeFileSystem::new_memory_fs("/bucket1".into());
    fs.ensure_exist().await?;

    // commit lock not found
    assert!(CommitLock::load(&fs).await.is_err());

    let lock = CommitLock::new(
      vec!["file1".to_string(), "file2".to_string()],
      vec!["file3".to_string()],
    );
    lock.save(&fs).await?;
    let new_lock = CommitLock::load(&fs).await?;
    assert_eq!(lock, new_lock);
    Ok(())
  }
}
