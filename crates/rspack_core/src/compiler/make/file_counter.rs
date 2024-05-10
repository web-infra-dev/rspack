use std::path::PathBuf;

use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};

/// Used to count file usage
#[derive(Debug, Default)]
pub struct FileCounter {
  inner: HashMap<PathBuf, usize>,
}

impl FileCounter {
  /// Add a pathbuf to counter
  ///
  /// It will +1 to the pathbuf in inner hashmap
  fn add_file(&mut self, path: &PathBuf) {
    if let Some(value) = self.inner.get_mut(path) {
      *value += 1;
    } else {
      self.inner.insert(path.clone(), 1);
    }
  }

  /// Remove a pathbuf from counter
  ///
  /// It will -1 to the pathbuf in inner hashmap
  ///
  /// If the pathbuf usage is 0 after reduction, the record will be deleted
  /// If pathbuf does not exist, panic will occur.
  fn remove_file(&mut self, path: &PathBuf) {
    if let Some(value) = self.inner.get_mut(path) {
      *value -= 1;
      if value == &0 {
        self.inner.remove(path);
      }
    } else {
      panic!("can not remove file {:?}", path);
    }
  }

  /// Add batch pathbuf to counter
  pub fn add_batch_file(&mut self, paths: &HashSet<PathBuf>) {
    for path in paths {
      self.add_file(path);
    }
  }

  /// Remove batch pathbuf to counter
  pub fn remove_batch_file(&mut self, paths: &HashSet<PathBuf>) {
    for path in paths {
      self.remove_file(path);
    }
  }

  /// Get files with count more than 0
  pub fn files(&self) -> impl Iterator<Item = &PathBuf> {
    self.inner.keys()
  }
}

#[cfg(test)]
mod test {
  use super::FileCounter;
  #[test]
  fn file_counter_is_available() {
    let mut counter = FileCounter::default();
    let file_a = std::path::PathBuf::from("/a");
    let file_b = std::path::PathBuf::from("/b");

    counter.add_file(&file_a);
    counter.add_file(&file_a);
    counter.add_file(&file_b);
    assert_eq!(counter.files().collect::<Vec<_>>().len(), 2);

    counter.remove_file(&file_a);
    assert_eq!(counter.files().collect::<Vec<_>>().len(), 2);

    counter.remove_file(&file_b);
    assert_eq!(counter.files().collect::<Vec<_>>().len(), 1);

    counter.remove_file(&file_a);
    assert_eq!(counter.files().collect::<Vec<_>>().len(), 0);
  }

  #[test]
  fn file_counter_add_file() {
    let mut counter = FileCounter::default();
    let file_a = std::path::PathBuf::from("/a");
    assert_eq!(counter.inner.get(&file_a), None);

    counter.add_file(&file_a);
    assert_eq!(counter.inner.get(&file_a), Some(&1));

    counter.add_file(&file_a);
    assert_eq!(counter.inner.get(&file_a), Some(&2));
  }

  #[test]
  fn file_counter_remove_file() {
    let mut counter = FileCounter::default();
    let file_a = std::path::PathBuf::from("/a");
    counter.add_file(&file_a);
    assert_eq!(counter.inner.get(&file_a), Some(&1));

    counter.remove_file(&file_a);
    assert_eq!(counter.inner.get(&file_a), None);
  }

  #[test]
  #[should_panic]
  fn file_counter_remove_file_with_panic() {
    let mut counter = FileCounter::default();
    let file_a = std::path::PathBuf::from("/a");
    counter.remove_file(&file_a);
  }
}
