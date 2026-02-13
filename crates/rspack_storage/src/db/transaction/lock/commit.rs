use super::super::{Error, Result};
use crate::fs::ScopeFileSystem;

const COMMIT_LOCK_FILE: &str = "commit.lock";

/// Commit lock - records all operations in the commit
#[derive(Debug)]
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

  /// Load commit.lock from the given filesystem
  pub async fn load(fs: &ScopeFileSystem) -> Result<Option<Self>> {
    // Check if file exists
    match fs.stat(COMMIT_LOCK_FILE).await {
      Ok(_) => {}
      Err(_) => return Ok(None),
    }

    // Read file
    let mut reader = fs.read_file(COMMIT_LOCK_FILE).await?;
    let mut added_files = Vec::new();
    let mut removed_files = Vec::new();
    let mut current_section = "";

    while let Ok(line) = reader.read_line().await {
      let line = line.trim();
      if line.is_empty() {
        continue;
      }

      if line == "[ADD]" {
        current_section = "add";
      } else if line == "[REMOVE]" {
        current_section = "remove";
      } else if current_section == "add" {
        added_files.push(line.to_string());
      } else if current_section == "remove" {
        removed_files.push(line.to_string());
      } else {
        return Err(Error::InvalidFormat("failed to read commit lock".into()));
      }
    }

    Ok(Some(Self {
      added_files,
      removed_files,
    }))
  }

  /// Save commit.lock to the given filesystem
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

  pub fn removed_files(&self) -> &Vec<String> {
    &self.removed_files
  }

  pub fn added_files(&self) -> &Vec<String> {
    &self.added_files
  }
}

/*#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_format() {
    let add_files = vec!["file1".to_string(), "file2".to_string()];
    let remove_files = vec!["file3".to_string()];
    let lock = CommitLock::new(add_files, remove_files);

    let bytes = lock.to_bytes();
    let new_lock = CommitLock::from_bytes(&bytes).unwrap();

    assert_eq!(lock.to_bytes(), new_lock.to_bytes());
  }

  #[test]
  fn test_invalid_format() {
    // Invalid UTF-8
    assert!(CommitLock::from_bytes(&[0xFF, 0xFE]).is_err());

    // Missing section marker
    assert!(CommitLock::from_bytes(b"file1\nfile2").is_err());

    // Valid empty lock
    let empty = CommitLock::new(vec![], vec![]);
    assert!(CommitLock::from_bytes(&empty.to_bytes()).is_ok());
  }
}*/
