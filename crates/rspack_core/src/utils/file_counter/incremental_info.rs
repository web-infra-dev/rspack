use std::path::PathBuf;

use rustc_hash::FxHashSet as HashSet;

/// Used to collect file add or remove.
#[derive(Debug, Default)]
pub struct IncrementalInfo {
  added_files: HashSet<PathBuf>,
  removed_files: HashSet<PathBuf>,
}

impl IncrementalInfo {
  /// Get added files
  pub fn added_files(&self) -> &HashSet<PathBuf> {
    &self.added_files
  }

  /// Get removed files
  pub fn removed_files(&self) -> &HashSet<PathBuf> {
    &self.removed_files
  }

  /// Add a file
  pub fn add(&mut self, path: &PathBuf) {
    if !self.removed_files.remove(path) {
      self.added_files.insert(path.clone());
    }
  }

  /// Remove a file
  pub fn remove(&mut self, path: &PathBuf) {
    if !self.added_files.remove(path) {
      self.removed_files.insert(path.clone());
    }
  }

  /// Reset added and removed files
  pub fn reset(&mut self) {
    self.added_files.clear();
    self.removed_files.clear();
  }
}

#[cfg(test)]
mod test {
  use super::IncrementalInfo;
  #[test]
  fn incremental_info_is_available() {
    let mut info = IncrementalInfo::default();
    let file_a = std::path::PathBuf::from("/a");

    info.add(&file_a);
    info.add(&file_a);
    assert_eq!(info.added_files().len(), 1);
    assert_eq!(info.removed_files().len(), 0);

    info.remove(&file_a);
    assert_eq!(info.added_files().len(), 0);
    assert_eq!(info.removed_files().len(), 0);

    info.remove(&file_a);
    assert_eq!(info.added_files().len(), 0);
    assert_eq!(info.removed_files().len(), 1);

    info.remove(&file_a);
    assert_eq!(info.added_files().len(), 0);
    assert_eq!(info.removed_files().len(), 1);
  }
}
