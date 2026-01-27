// Commit lock - records all operations in the commit
//
// Format:
// ```text
// [ADD]
// file1
// file2
// [REMOVE]
// file3
// file4
// ```

use rspack_paths::Utf8PathBuf;

/// Commit lock - records all operations in the commit
///
/// Format:
/// ```text
/// [ADD]
/// file1
/// file2
/// [REMOVE]
/// file3
/// file4
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommitLock {
  pub files_to_add: Vec<Utf8PathBuf>,
  pub files_to_remove: Vec<Utf8PathBuf>,
}

impl CommitLock {
  /// Create a new commit lock
  pub fn new(files_to_add: Vec<Utf8PathBuf>, files_to_remove: Vec<Utf8PathBuf>) -> Self {
    Self {
      files_to_add,
      files_to_remove,
    }
  }

  /// Serialize to string format
  ///
  /// Format:
  /// ```text
  /// [ADD]
  /// file1
  /// file2
  /// [REMOVE]
  /// file3
  /// file4
  /// ```
  pub fn to_string(&self) -> String {
    let mut lines = vec!["[ADD]".to_string()];
    lines.extend(self.files_to_add.iter().map(|p| p.to_string()));
    lines.push("[REMOVE]".to_string());
    lines.extend(self.files_to_remove.iter().map(|p| p.to_string()));
    lines.join("\n")
  }

  /// Deserialize from string format
  ///
  /// Returns None if the format is invalid
  pub fn from_string(s: &str) -> Option<Self> {
    let lines: Vec<&str> = s.split('\n').collect();
    let mut files_to_add = Vec::new();
    let mut files_to_remove = Vec::new();
    let mut current_section = "";

    for line in lines {
      let line = line.trim();
      if line.is_empty() {
        continue;
      }

      if line == "[ADD]" {
        current_section = "add";
      } else if line == "[REMOVE]" {
        current_section = "remove";
      } else if current_section == "add" {
        files_to_add.push(Utf8PathBuf::from(line));
      } else if current_section == "remove" {
        files_to_remove.push(Utf8PathBuf::from(line));
      }
    }

    Some(Self {
      files_to_add,
      files_to_remove,
    })
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_new() {
    let add_files = vec![Utf8PathBuf::from("file1"), Utf8PathBuf::from("file2")];
    let remove_files = vec![Utf8PathBuf::from("file3")];
    let lock = CommitLock::new(add_files.clone(), remove_files.clone());
    assert_eq!(lock.files_to_add, add_files);
    assert_eq!(lock.files_to_remove, remove_files);
  }

  #[test]
  fn test_to_string() {
    let add_files = vec![Utf8PathBuf::from("file1"), Utf8PathBuf::from("file2")];
    let remove_files = vec![Utf8PathBuf::from("file3")];
    let lock = CommitLock::new(add_files, remove_files);

    let s = lock.to_string();
    assert_eq!(s, "[ADD]\nfile1\nfile2\n[REMOVE]\nfile3");
  }

  #[test]
  fn test_from_string() {
    let s = "[ADD]\nfile1\nfile2\n[REMOVE]\nfile3";
    let lock = CommitLock::from_string(s).unwrap();

    assert_eq!(lock.files_to_add.len(), 2);
    assert_eq!(lock.files_to_add[0], Utf8PathBuf::from("file1"));
    assert_eq!(lock.files_to_add[1], Utf8PathBuf::from("file2"));
    assert_eq!(lock.files_to_remove.len(), 1);
    assert_eq!(lock.files_to_remove[0], Utf8PathBuf::from("file3"));
  }

  #[test]
  fn test_roundtrip() {
    let add_files = vec![Utf8PathBuf::from("file1"), Utf8PathBuf::from("file2")];
    let remove_files = vec![Utf8PathBuf::from("file3")];
    let lock = CommitLock::new(add_files, remove_files);

    let s = lock.to_string();
    let parsed = CommitLock::from_string(&s).unwrap();

    assert_eq!(lock, parsed);
  }
}
