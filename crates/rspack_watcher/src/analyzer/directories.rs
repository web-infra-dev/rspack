#![allow(unused)]
use rspack_paths::ArcPath;
use rspack_util::fx_hash::{FxHashMap as HashMap, FxHashSet as HashSet};

use super::{Analyzer, WatchPattern};
use crate::paths::PathAccessor;

/// `WatcherDirectoriesAnalyzer` analyzes the path register and determines
///
/// which directories should be watched individually. File parents stay
/// non-recursive, while registered context directories must be recursive so
/// changes in pre-existing child directories are observable.
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
  /// Finds all directories that should be watched individually, keeping the
  /// strongest required mode when a file parent and context share a path.
  fn find_watch_directories<'a>(&self, path_accessor: PathAccessor<'a>) -> HashSet<WatchPattern> {
    let mut modes = HashMap::default();
    let directories = path_accessor.directories().0;

    for path in path_accessor.all() {
      if let Some((dir, deep)) = self.find_exists_path(path) {
        let recursive = deep >= DIRECTORY_WATCH_DEPTH || directories.contains(&dir);
        modes
          .entry(dir)
          .and_modify(|current| *current |= recursive)
          .or_insert(recursive);
      }
    }

    // A recursive root already covers its descendants. Re-registering a child
    // non-recursively duplicates inotify work and can downgrade that child's mode.
    let recursive_roots: HashSet<ArcPath> = modes
      .iter()
      .filter_map(|(path, recursive)| recursive.then_some(path.clone()))
      .collect();

    modes
      .into_iter()
      .filter(|(path, _)| {
        path
          .as_ref()
          .ancestors()
          .skip(1)
          .all(|parent| !recursive_roots.contains(&ArcPath::from(parent)))
      })
      .map(|(path, recursive)| WatchPattern {
        path,
        mode: if recursive {
          notify::RecursiveMode::Recursive
        } else {
          notify::RecursiveMode::NonRecursive
        },
      })
      .collect()
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
  use crate::paths::PathManager;

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
      mode: notify::RecursiveMode::Recursive
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

    assert_eq!(watch_patterns.len(), 2);
    assert!(watch_patterns.contains(&WatchPattern {
      path: dir_0,
      mode: notify::RecursiveMode::Recursive,
    }));
    assert!(watch_patterns.contains(&WatchPattern {
      path: ArcPath::from(current_dir),
      mode: notify::RecursiveMode::NonRecursive,
    }));
  }

  #[test]
  fn test_recursive_context_prunes_covered_file_parents() {
    let temp_dir = tempfile::TempDir::new().unwrap();
    let base = temp_dir.path().canonicalize().unwrap();
    let context_a = base.join("a");
    let context_b = base.join("b");
    let child = context_a.join("child");
    let file = child.join("file.txt");
    std::fs::create_dir_all(&child).unwrap();
    std::fs::create_dir_all(&context_b).unwrap();
    std::fs::write(&file, "file").unwrap();

    let path_manager = PathManager::default();
    path_manager
      .update(
        (std::iter::once(ArcPath::from(file)), std::iter::empty()),
        (
          [
            ArcPath::from(context_a.clone()),
            ArcPath::from(context_b.clone()),
          ]
          .into_iter(),
          std::iter::empty(),
        ),
        (std::iter::empty(), std::iter::empty()),
      )
      .unwrap();

    let watch_patterns = WatcherDirectoriesAnalyzer.analyze(path_manager.access());

    assert_eq!(watch_patterns.len(), 2);
    for context in [context_a, context_b] {
      assert!(watch_patterns.contains(&WatchPattern {
        path: ArcPath::from(context),
        mode: notify::RecursiveMode::Recursive,
      }));
    }
  }
}
