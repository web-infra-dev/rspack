mod incremental_info;

use incremental_info::IncrementalInfo;
use rspack_paths::ArcPath;
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};

/// Used to count file usage
#[derive(Debug, Default)]
pub struct FileCounter {
  inner: HashMap<ArcPath, usize>,
  incremental_info: IncrementalInfo,
}

impl FileCounter {
  /// Add a [`PathBuf`] to counter
  ///
  /// It will +1 to the PathBuf in inner hashmap
  fn add_file(&mut self, path: ArcPath) {
    if let Some(value) = self.inner.get_mut(&path) {
      *value += 1;
    } else {
      self.incremental_info.add(&path);
      self.inner.insert(path, 1);
    }
  }

  /// Remove a [`PathBuf`] from counter
  ///
  /// It will -1 to the PathBuf in inner hashmap
  ///
  /// If the PathBuf usage is 0 after reduction, the record will be deleted
  /// If PathBuf does not exist, panic will occur.
  fn remove_file(&mut self, path: &ArcPath) {
    if let Some(value) = self.inner.get_mut(path) {
      *value -= 1;
      if value == &0 {
        self.incremental_info.remove(path);
        self.inner.remove(path);
      }
    } else {
      panic!("can not remove file {path:?}");
    }
  }

  /// Add batch [`PathBuf``] to counter
  pub fn add_batch_file(&mut self, paths: &HashSet<ArcPath>) {
    for path in paths {
      self.add_file(path.clone());
    }
  }

  /// Remove batch [`PathBuf`] to counter
  pub fn remove_batch_file(&mut self, paths: &HashSet<ArcPath>) {
    for path in paths {
      self.remove_file(path);
    }
  }

  /// Get files with count more than 0
  pub fn files(&self) -> impl Iterator<Item = &ArcPath> {
    self.inner.keys()
  }

  /// reset incremental info
  pub fn reset_incremental_info(&mut self) {
    self.incremental_info.reset();
  }

  /// Added files compared to the `files()` when call reset_incremental_info
  pub fn added_files(&self) -> &HashSet<ArcPath> {
    self.incremental_info.added_files()
  }

  /// Removed files compared to the `files()` when call reset_incremental_info
  pub fn removed_files(&self) -> &HashSet<ArcPath> {
    self.incremental_info.removed_files()
  }
}

#[cfg(test)]
mod test {
  use rspack_paths::ArcPath;

  use super::FileCounter;
  #[test]
  fn file_counter_is_available() {
    let mut counter = FileCounter::default();
    let file_a = std::path::PathBuf::from("/a");
    let file_b = std::path::PathBuf::from("/b");

    let file_a = ArcPath::from(file_a);
    let file_b = ArcPath::from(file_b);

    counter.add_file(file_a.clone());
    counter.add_file(file_a.clone());
    counter.add_file(file_b.clone());
    assert_eq!(counter.files().collect::<Vec<_>>().len(), 2);
    assert_eq!(counter.added_files().len(), 2);
    assert_eq!(counter.removed_files().len(), 0);

    counter.remove_file(&file_a);
    assert_eq!(counter.files().collect::<Vec<_>>().len(), 2);
    assert_eq!(counter.added_files().len(), 2);
    assert_eq!(counter.removed_files().len(), 0);

    counter.remove_file(&file_b);
    assert_eq!(counter.files().collect::<Vec<_>>().len(), 1);
    assert_eq!(counter.added_files().len(), 1);
    assert_eq!(counter.removed_files().len(), 0);

    counter.remove_file(&file_a);
    assert_eq!(counter.files().collect::<Vec<_>>().len(), 0);
    assert_eq!(counter.added_files().len(), 0);
    assert_eq!(counter.removed_files().len(), 0);
  }

  #[test]
  fn file_counter_add_file() {
    let mut counter = FileCounter::default();
    let file_a = std::path::PathBuf::from("/a");
    let file_a = ArcPath::from(file_a);
    assert_eq!(counter.inner.get(&file_a), None);

    counter.add_file(file_a.clone());
    assert_eq!(counter.inner.get(&file_a), Some(&1));

    counter.add_file(file_a.clone());
    assert_eq!(counter.inner.get(&file_a), Some(&2));
  }

  #[test]
  fn file_counter_remove_file() {
    let mut counter = FileCounter::default();
    let file_a = std::path::PathBuf::from("/a");
    let file_a = ArcPath::from(file_a);
    counter.add_file(file_a.clone());
    assert_eq!(counter.inner.get(&file_a), Some(&1));

    counter.remove_file(&file_a);
    assert_eq!(counter.inner.get(&file_a), None);
  }

  #[test]
  #[should_panic]
  fn file_counter_remove_file_with_panic() {
    let mut counter = FileCounter::default();
    let file_a = std::path::PathBuf::from("/a");
    let file_a = ArcPath::from(file_a);
    counter.remove_file(&file_a);
  }

  #[test]
  fn file_counter_reset_incremental_info() {
    let mut counter = FileCounter::default();
    let file_a = std::path::PathBuf::from("/a");
    let file_a = ArcPath::from(file_a);

    counter.add_file(file_a.clone());
    assert_eq!(counter.added_files().len(), 1);
    assert_eq!(counter.removed_files().len(), 0);

    counter.reset_incremental_info();
    assert_eq!(counter.added_files().len(), 0);
    assert_eq!(counter.removed_files().len(), 0);

    counter.add_file(file_a.clone());
    assert_eq!(counter.added_files().len(), 0);
    assert_eq!(counter.removed_files().len(), 0);

    counter.remove_file(&file_a);
    assert_eq!(counter.added_files().len(), 0);
    assert_eq!(counter.removed_files().len(), 0);

    counter.remove_file(&file_a);
    assert_eq!(counter.added_files().len(), 0);
    assert_eq!(counter.removed_files().len(), 1);
  }
}
