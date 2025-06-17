use std::{collections::HashSet, path::PathBuf};

use super::{for_each, Analyzer, WatchPattern};
use crate::watcher::manager::PathAccessor;

/// `WatcherDirectoriesAnalyzer` analyzes the path register and determines
///
/// which directories should be watched individually (non-recursively).
/// This is typically used on platforms where recursive watching is not
/// available or not desired, so each directory is watched separately.
pub struct WatcherDirectoriesAnalyzer<'a> {
  path_accessor: PathAccessor<'a>,
}

impl<'a> Analyzer<'a> for WatcherDirectoriesAnalyzer<'a> {
  fn new(path_accessor: PathAccessor<'a>) -> Self {
    Self { path_accessor }
  }

  fn analyze(&self) -> Vec<WatchPattern> {
    self
      .find_watch_directories()
      .into_iter()
      .map(|path| WatchPattern {
        path,
        // WatcherDirectoriesAnalyzer is using in non-macos and non-window os.
        // so we watch every directory non-recursively.
        // This is because recursive watching is not available on these platforms.
        mode: notify::RecursiveMode::NonRecursive,
      })
      .collect()
  }
}

impl<'a> WatcherDirectoriesAnalyzer<'a> {
  /// Finds all directories that should be watched individually (non-recursively).
  fn find_watch_directories(&self) -> HashSet<PathBuf> {
    let mut directories = HashSet::new();
    for_each(&self.path_accessor, |path| {
      if path.is_dir() {
        directories.insert(path.clone());
      } else if let Some(parent) = path.parent() {
        directories.insert(parent.to_path_buf());
      }
    });
    directories
  }
}

#[cfg(test)]
mod tests {
  use dashmap::DashSet as HashSet;

  use super::*;

  #[test]
  fn test_find_watch_directories() {
    let files = HashSet::with_capacity(3);
    let current_dir = std::env::current_dir().expect("Failed to get current directory");
    files.insert(current_dir.join("Cargo.toml"));
    files.insert(current_dir.join("src"));
    files.insert(current_dir.join("src/lib.rs"));
    let directories = HashSet::with_capacity(1);
    let missing = HashSet::new();
    let path_accessor = PathAccessor::new(&files, &directories, &missing);
    let analyzer = WatcherDirectoriesAnalyzer::new(path_accessor);
    let watch_directories = analyzer.find_watch_directories();

    assert_eq!(watch_directories.len(), 2);
    assert!(watch_directories.contains(&current_dir));
    assert!(watch_directories.contains(&current_dir.join("src")));
  }
}
