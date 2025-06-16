use std::{collections::HashSet, path::PathBuf};

use super::{for_each, Analyzer, WatchTarget};
use crate::watcher::register::PathRegister;

/// `WatcherDirectoriesAnalyzer` analyzes the path register and determines
///
/// which directories should be watched individually (non-recursively).
/// This is typically used on platforms where recursive watching is not
/// available or not desired, so each directory is watched separately.
#[derive(Default)]
pub struct WatcherDirectoriesAnalyzer;

impl Analyzer for WatcherDirectoriesAnalyzer {
  fn analyze(&self, register: &PathRegister) -> Vec<WatchTarget> {
    self
      .find_watch_directories(register)
      .into_iter()
      .map(|path| WatchTarget {
        path,
        mode: notify::RecursiveMode::NonRecursive,
      })
      .collect()
  }
}

impl WatcherDirectoriesAnalyzer {
  /// Finds all directories that should be watched individually (non-recursively).
  fn find_watch_directories(&self, register: &PathRegister) -> HashSet<PathBuf> {
    let mut directories = HashSet::new();
    for_each(register, |path| {
      if path.is_dir() {
        directories.insert(path.clone());
      } else {
        directories.insert(path.parent().unwrap().to_path_buf());
      }
    });
    directories
  }
}
