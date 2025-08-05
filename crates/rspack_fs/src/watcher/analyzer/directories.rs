#![allow(unused)]
use rspack_paths::ArcPath;
use rspack_util::fx_hash::FxHashSet as HashSet;

use super::{Analyzer, WatchPattern};
use crate::watcher::paths::PathAccessor;

/// `WatcherDirectoriesAnalyzer` analyzes the path register and determines
///
/// which directories should be watched individually (non-recursively).
/// This is typically used on platforms where recursive watching is not
/// available or not desired, so each directory is watched separately.
#[derive(Default)]
pub struct WatcherDirectoriesAnalyzer;

impl Analyzer for WatcherDirectoriesAnalyzer {
  fn analyze<'a>(&self, path_accessor: PathAccessor<'a>) -> Vec<WatchPattern> {
    self
      .find_watch_directories(path_accessor)
      .into_iter()
      .collect()
  }
}

const DIRECTORY_WATCH_DEPTH: u32 = 2;

impl WatcherDirectoriesAnalyzer {
  /// Finds all directories that should be watched individually (non-recursively).
  fn find_watch_directories<'a>(&self, path_accessor: PathAccessor<'a>) -> HashSet<WatchPattern> {
    let mut patterns = HashSet::default();
    let all = path_accessor.all();
    for path in all {
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
    }

    patterns
  }

  /// Finds the deepest existing directory path and its depth.
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
  use super::*;
  use crate::watcher::paths::PathManager;

  #[test]
  fn test_find_watch_directories() {
    let current_dir = std::env::current_dir().expect("Failed to get current directory");
    let path_manager = PathManager::default();
    let files = (
      vec![
        current_dir.join("Cargo.toml").into(),
        current_dir.join("src/lib.rs").into(),
      ]
      .into_iter(),
      vec![].into_iter(),
    );

    let dirs = (
      vec![current_dir.join("src").into()].into_iter(),
      vec![].into_iter(),
    );

    let missing = (vec![].into_iter(), vec![].into_iter());

    path_manager.update(files, dirs, missing).unwrap();
    let analyzer = WatcherDirectoriesAnalyzer::default();
    let watch_patterns = analyzer.analyze(path_manager.access());

    assert_eq!(watch_patterns.len(), 2);
    assert!(watch_patterns.contains(&{
      WatchPattern {
        path: ArcPath::from(current_dir.clone()),
        mode: notify::RecursiveMode::NonRecursive,
      }
    }));
    assert!(watch_patterns.contains(&WatchPattern {
      path: ArcPath::from(current_dir.join("src")),
      mode: notify::RecursiveMode::NonRecursive
    }));
  }

  #[test]
  fn test_find_non_exists_watcher_directories() {
    let current_dir = std::env::current_dir().expect("Failed to get current directory");
    let dir_0 = ArcPath::from(current_dir.join("src"));

    let path_manager = PathManager::default();
    let files = (
      vec![
        current_dir.join("Cargo.toml").into(),
        current_dir.join("src/a/b/c/d.rs").into(),
      ]
      .into_iter(),
      vec![].into_iter(),
    );
    let dirs = (
      vec![
        current_dir.join("src").into(),
        current_dir.join("src/b/c/d/e").into(),
      ]
      .into_iter(),
      vec![].into_iter(),
    );
    let missing = (vec![].into_iter(), vec![].into_iter());

    path_manager.update(files, dirs, missing).unwrap();

    let analyzer = WatcherDirectoriesAnalyzer::default();
    let watch_patterns = analyzer.analyze(path_manager.access());

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
