use std::sync::LazyLock;

use rspack_fs::RelPath;

use super::super::ScopeFileSystem;
use crate::{Error, Result};

const COMMIT_LOCK_FILE: &str = "commit.lock";

fn commit_lock_file() -> &'static RelPath {
  static PATH: LazyLock<RelPath> =
    LazyLock::new(|| RelPath::new_contained(COMMIT_LOCK_FILE).expect("valid commit lock filename"));
  &PATH
}

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
  added_files: Vec<RelPath>,
  removed_files: Vec<RelPath>,
}

impl CommitLock {
  pub fn new(added_files: Vec<RelPath>, removed_files: Vec<RelPath>) -> Self {
    // added_files and removed_files should be mutually exclusive.
    debug_assert!(!removed_files.iter().any(|item| added_files.contains(item)));
    Self {
      added_files,
      removed_files,
    }
  }

  /// Loads commit lock from filesystem and returns the pending operations.
  pub async fn load(fs: &ScopeFileSystem) -> Result<Self> {
    let mut reader = fs.stream_read(commit_lock_file()).await?;
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
        "[ADD]" => current_section = Some(&mut added_files),
        "[REMOVE]" => current_section = Some(&mut removed_files),
        _ => {
          let Some(files) = current_section.as_deref_mut() else {
            return Err(Error::InvalidFormat(format!(
              "Unexpected line in '{COMMIT_LOCK_FILE}' before section header: '{line}'"
            )));
          };
          files.push(RelPath::new_contained(line).map_err(|e| {
            Error::InvalidFormat(format!(
              "Invalid path in '{COMMIT_LOCK_FILE}': '{line}' ({e})"
            ))
          })?);
        }
      }
    }

    Ok(Self {
      added_files,
      removed_files,
    })
  }

  /// Saves commit lock to filesystem with pending operations.
  pub async fn save(&self, fs: &ScopeFileSystem) -> Result<()> {
    let mut writer = fs.stream_write(commit_lock_file()).await?;

    writer.write_line("[ADD]").await?;
    for file in &self.added_files {
      writer.write_line(file.as_str()).await?;
    }

    writer.write_line("[REMOVE]").await?;
    for file in &self.removed_files {
      writer.write_line(file.as_str()).await?;
    }

    writer.flush().await?;
    Ok(())
  }

  /// Returns the list of files to be removed.
  pub fn removed_files(&self) -> &[RelPath] {
    &self.removed_files
  }

  /// Returns the list of files to be added.
  pub fn added_files(&self) -> &[RelPath] {
    &self.added_files
  }
}

#[cfg(test)]
mod tests {
  use rspack_fs::RelPath;

  use super::{CommitLock, Result, ScopeFileSystem};

  fn rel(path: &str) -> RelPath {
    RelPath::new_contained(path).expect("valid test relative path")
  }

  #[tokio::test]
  #[cfg_attr(miri, ignore)]
  async fn test_commit_lock() -> Result<()> {
    let fs = ScopeFileSystem::new_memory_fs("/bucket1");
    fs.ensure_exist().await?;

    // commit lock not found
    assert!(CommitLock::load(&fs).await.is_err());

    let lock = CommitLock::new(vec![rel("file1"), rel("file2")], vec![rel("file3")]);
    lock.save(&fs).await?;
    let new_lock = CommitLock::load(&fs).await?;
    assert_eq!(lock, new_lock);
    Ok(())
  }
}
