use rspack_paths::ArcPath;
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

const DIRECTORY_WATCH_DEPTH: u32 = 2;

impl<'a> WatcherDirectoriesAnalyzer<'a> {
  /// Finds all directories that should be watched individually (non-recursively).
  fn find_watch_directories(&self) -> HashSet<WatchPattern> {
    let mut patterns = HashSet::default();
    for_each(&self.path_accessor, |path| {
      if let Some((dir, deep)) = self.find_exists_path(path) {
        // Insert the parent directory of the file
        patterns.insert(WatchPattern {
          path: dir,
          mode: if deep >= DIRECTORY_WATCH_DEPTH {
            notify::RecursiveMode::Recursive
          } else {
            notify::RecursiveMode::NonRecursive
          },
        });
      }
    });
    patterns
  }

  fn find_exists_path(&self, path: ArcPath) -> Option<(ArcPath, u32)> {
    let mut current = path;
    let mut deep = 0u32;
    // Traverse up the path until we find a directory that exists
    while !current.is_dir() {
      deep += 1;
      if let Some(parent) = current.parent() {
        current = ArcPath::from(parent);
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
    let file_0 = ArcPath::from(current_dir.join("Cargo.toml"));
    let file_1 = ArcPath::from(current_dir.join("src"));
    let file_2 = ArcPath::from(current_dir.join("src/lib.rs"));

    files.insert(file_0.clone());
    files.insert(file_1.clone());
    files.insert(file_2.clone());
    let directories = HashSet::with_capacity(1);
    let missing = HashSet::new();
    let path_accessor = PathAccessor::new(&files, &directories, &missing);
    let analyzer = WatcherDirectoriesAnalyzer::new(path_accessor);
    let watch_patterns = analyzer.analyze();

    assert_eq!(watch_patterns.len(), 2);
    assert!(watch_patterns.contains(&{
      WatchPattern {
        path: ArcPath::from(current_dir),
        mode: notify::RecursiveMode::NonRecursive,
      }
    }));
    assert!(watch_patterns.contains(&WatchPattern {
      path: file_1,
      mode: notify::RecursiveMode::NonRecursive
    }));
  }

  #[test]
  fn test_find_non_exsists_watcher_directories() {
    let files = HashSet::with_capacity(3);
    let current_dir = std::env::current_dir().expect("Failed to get current directory");
    let file_0 = ArcPath::from(current_dir.join("Cargo.toml"));
    let file_1 = ArcPath::from(current_dir.join("src/a/b/c/d.rs"));
    files.insert(file_0.clone());
    files.insert(file_1.clone());
    let directories = HashSet::new();
    let dir_0 = ArcPath::from(current_dir.join("src"));
    let dir_1 = ArcPath::from(current_dir.join("src/b/c/d/e"));

    directories.insert(dir_0.clone());
    directories.insert(dir_1.clone());
    let missing = HashSet::new();
    let path_accessor = PathAccessor::new(&files, &directories, &missing);
    let analyzer = WatcherDirectoriesAnalyzer::new(path_accessor);
    let watch_patterns = analyzer.analyze();

    assert_eq!(watch_patterns.len(), 3);
    assert!(watch_patterns.contains(&WatchPattern {
      path: dir_0.clone(),
      mode: notify::RecursiveMode::NonRecursive,
    }));
    assert!(watch_patterns.contains(&WatchPattern {
      path: dir_0,
      mode: notify::RecursiveMode::Recursive,
    }));
    assert!(watch_patterns.contains(&WatchPattern {
      path: ArcPath::from(current_dir),
      mode: notify::RecursiveMode::NonRecursive,
    }));
  }
}
