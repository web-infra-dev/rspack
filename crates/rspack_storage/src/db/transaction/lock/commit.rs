use rspack_paths::Utf8PathBuf;

/// Commit lock - records all operations in the commit
#[derive(Debug)]
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
      } else {
        // format invalid
        return None;
      }
    }

    Self {
      files_to_add,
      files_to_remove,
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_format() {
    let add_files = vec![Utf8PathBuf::from("file1"), Utf8PathBuf::from("file2")];
    let remove_files = vec![Utf8PathBuf::from("file3")];
    let lock = CommitLock::new(add_files, remove_files);

    let new_lock = CommitLock::from_string(&lock.to_string()).unwrap();

    assert_eq!(lock.to_string(), new_lock.to_string());
  }
}
