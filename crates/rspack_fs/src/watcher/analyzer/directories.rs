use std::path::{Path, PathBuf};

use rspack_util::fx_hash::FxHashSet as HashSet;

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
    self.find_watch_directories().into_iter().collect()
  }
}

impl<'a> WatcherDirectoriesAnalyzer<'a> {
  /// Finds all directories that should be watched individually (non-recursively).
  fn find_watch_directories(&self) -> HashSet<WatchPattern> {
    let mut patterns = HashSet::default();
    for_each(&self.path_accessor, |path| {
      if let Some((dir, deep)) = self.find_exists_path(path) {
        // Insert the parent directory of the file
        patterns.insert(WatchPattern {
          path: dir,
          mode: if deep >= 2 {
            notify::RecursiveMode::Recursive
          } else {
            notify::RecursiveMode::NonRecursive
          },
        });
      }
    });
    patterns
  }

  fn find_exists_path(&self, path: &Path) -> Option<(PathBuf, u32)> {
    let mut current = path.to_path_buf();
    let mut deep = 0u32;
    // Traverse up the path until we find a directory that exists
    while !current.is_dir() {
      deep += 1;
      if let Some(parent) = current.parent() {
        current = parent.to_path_buf();
      } else {
        return None; // No parent exists
      }
    }
    Some((current, deep))
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
    let watch_patterns = analyzer.analyze();

    assert_eq!(watch_patterns.len(), 2);
    assert!(watch_patterns.contains(&{
      WatchPattern {
        path: current_dir.clone(),
        mode: notify::RecursiveMode::NonRecursive,
      }
    }));
    assert!(watch_patterns.contains(&WatchPattern {
      path: current_dir.join("src"),
      mode: notify::RecursiveMode::NonRecursive
    }));
  }

  #[test]
  fn test_find_non_exsists_watcher_directories() {
    let files = HashSet::with_capacity(3);
    let current_dir = std::env::current_dir().expect("Failed to get current directory");
    files.insert(current_dir.join("Cargo.toml"));
    files.insert(current_dir.join("src/a/b/c/d.rs"));
    let directories = HashSet::new();
    directories.insert(current_dir.join("src"));
    directories.insert(current_dir.join("src/b/c/d/e"));
    let missing = HashSet::new();
    let path_accessor = PathAccessor::new(&files, &directories, &missing);
    let analyzer = WatcherDirectoriesAnalyzer::new(path_accessor);
    let watch_patterns = analyzer.analyze();

    // println!("watch_directories: {:?}", watch_directories)ko
    assert_eq!(watch_patterns.len(), 3);
    assert!(watch_patterns.contains(&WatchPattern {
      path: current_dir.join("src"),
      mode: notify::RecursiveMode::NonRecursive,
    }));
    assert!(watch_patterns.contains(&WatchPattern {
      path: current_dir.join("src"),
      mode: notify::RecursiveMode::Recursive,
    }));
    assert!(watch_patterns.contains(&WatchPattern {
      path: current_dir,
      mode: notify::RecursiveMode::NonRecursive,
    }));
    // assert!(watch_directories.contains(&current_dir));
    // assert!(watch_directories.contains(&current_dir.join("src")));
  }
}
