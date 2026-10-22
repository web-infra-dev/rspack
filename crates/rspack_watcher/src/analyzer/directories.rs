#![allow(unused)]
use std::sync::Mutex;

use rspack_paths::{InternedPath, InternedPathMap, InternedPathSet};
use rspack_util::fx_hash::FxHashMap;

use super::{Analyzer, WatchPattern};
use crate::paths::PathAccessor;

/// `WatcherDirectoriesAnalyzer` analyzes the path register and determines
///
/// which directories should be watched individually (non-recursively).
/// This is typically used on platforms where recursive watching is not
/// available or not desired, so each directory is watched separately.
#[derive(Default)]
pub struct WatcherDirectoriesAnalyzer {
  state: Mutex<DirectoryAnalyzerState>,
  invalidated: Mutex<InvalidatedPaths>,
}

#[derive(Default)]
struct InvalidatedPaths {
  paths: InternedPathSet,
  all: bool,
}

#[derive(Default)]
struct DirectoryAnalyzerState {
  paths: InternedPathMap<Option<WatchPattern>>,
  patterns: FxHashMap<WatchPattern, usize>,
  symlink_dependent_paths: InternedPathSet,
}

impl Analyzer for WatcherDirectoriesAnalyzer {
  fn analyze<'a>(&self, path_accessor: PathAccessor<'a>) -> Vec<WatchPattern> {
    let mut state = self.state.lock().expect("directory analyzer lock poisoned");
    let invalidated = std::mem::take(
      &mut *self
        .invalidated
        .lock()
        .expect("directory invalidation lock poisoned"),
    );
    let subscriptions = [
      path_accessor.files(),
      path_accessor.directories(),
      path_accessor.missing(),
    ];
    let is_registered =
      |path: &InternedPath| subscriptions.iter().any(|(all, _, _)| all.contains(path));
    let mut dirty = state.symlink_dependent_paths.clone();
    for (_, added, removed) in subscriptions {
      dirty.extend(added.iter().map(|path| path.clone()));
      for path in removed.iter() {
        if !is_registered(&path) && state.paths.contains_key(&*path) {
          dirty.insert(path.clone());
        }
      }
    }
    if invalidated.all || !invalidated.paths.is_empty() {
      for path in path_accessor.all() {
        if invalidated.all
          || path
            .ancestors()
            .any(|ancestor| invalidated.paths.contains(&InternedPath::from(ancestor)))
        {
          dirty.insert(path);
        }
      }
    }

    let mut resolutions = InternedPathMap::default();
    let mut symlink_ancestors = InternedPathMap::default();
    for path in dirty {
      state.symlink_dependent_paths.remove(&path);
      if let Some(Some(pattern)) = state.paths.remove(&path) {
        let count = state.patterns.get_mut(&pattern).expect("watch requirement");
        *count -= 1;
        if *count == 0 {
          state.patterns.remove(&pattern);
        }
      }
      if is_registered(&path) {
        let pattern =
          find_existing_directory(path.clone(), &mut resolutions, &mut symlink_ancestors).map(
            |(dir, depth, recheck, non_directory_symlink)| {
              if recheck
                || (non_directory_symlink
                  && (path_accessor.directories().0.contains(&path)
                    || path_accessor.missing().0.contains(&path)))
              {
                state.symlink_dependent_paths.insert(path.clone());
              }
              WatchPattern {
                path: dir,
                mode: if depth >= DIRECTORY_WATCH_DEPTH {
                  notify::RecursiveMode::Recursive
                } else {
                  notify::RecursiveMode::NonRecursive
                },
              }
            },
          );
        if let Some(pattern) = &pattern {
          *state.patterns.entry(pattern.clone()).or_default() += 1;
        }
        state.paths.insert(path, pattern);
      }
    }
    state.patterns.keys().cloned().collect()
  }

  fn invalidate_path(&self, path: &InternedPath) {
    let mut invalidated = self
      .invalidated
      .lock()
      .expect("directory invalidation lock poisoned");
    if invalidated.all {
      return;
    }
    invalidated.paths.insert(path.clone());
    // Filtered events may never trigger another analyze, so bound retained path names.
    if invalidated.paths.len() >= MAX_INVALIDATED_PATHS {
      *invalidated = InvalidatedPaths {
        all: true,
        ..Default::default()
      };
    }
  }

  fn invalidate_all(&self) {
    *self
      .invalidated
      .lock()
      .expect("directory invalidation lock poisoned") = InvalidatedPaths {
      all: true,
      ..Default::default()
    };
  }
}

const DIRECTORY_WATCH_DEPTH: u32 = 2;
const MAX_INVALIDATED_PATHS: usize = 1024;

fn find_existing_directory(
  path: InternedPath,
  resolutions: &mut InternedPathMap<Option<(InternedPath, u32, bool, bool)>>,
  symlink_ancestors: &mut InternedPathMap<bool>,
) -> Option<(InternedPath, u32, bool, bool)> {
  if let Some(resolution) = resolutions.get(&path) {
    return resolution.clone();
  }
  let metadata = path.metadata();
  let resolution = if metadata.as_ref().is_ok_and(|metadata| metadata.is_dir()) {
    Some((
      path.clone(),
      0,
      has_symlink_ancestor(&path, symlink_ancestors),
      false,
    ))
  } else {
    // A non-directory link can become a directory outside native watch coverage.
    let non_directory_symlink = path.is_symlink();
    let unresolved_symlink = metadata.is_err() && non_directory_symlink;
    path
      .parent()
      .and_then(|parent| {
        find_existing_directory(InternedPath::from(parent), resolutions, symlink_ancestors)
      })
      .map(|(directory, depth, recheck, parent_symlink)| {
        (
          directory,
          depth + 1,
          recheck || unresolved_symlink || parent_symlink,
          non_directory_symlink,
        )
      })
  };
  resolutions.insert(path, resolution.clone());
  resolution
}

fn has_symlink_ancestor(
  path: &InternedPath,
  symlink_ancestors: &mut InternedPathMap<bool>,
) -> bool {
  if let Some(has_symlink) = symlink_ancestors.get(path) {
    return *has_symlink;
  }
  let has_symlink = path.is_symlink()
    || path
      .parent()
      .is_some_and(|parent| has_symlink_ancestor(&InternedPath::from(parent), symlink_ancestors));
  symlink_ancestors.insert(path.clone(), has_symlink);
  has_symlink
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::paths::PathManager;

  #[tokio::test]
  async fn test_find_watch_directories() {
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

    path_manager.update(files, dirs, missing).await.unwrap();
    let analyzer = WatcherDirectoriesAnalyzer::default();
    let watch_patterns = analyzer.analyze(path_manager.access());

    assert_eq!(watch_patterns.len(), 2);
    assert!(watch_patterns.contains(&{
      WatchPattern {
        path: InternedPath::from(current_dir.clone()),
        mode: notify::RecursiveMode::NonRecursive,
      }
    }));
    assert!(watch_patterns.contains(&WatchPattern {
      path: InternedPath::from(current_dir.join("src")),
      mode: notify::RecursiveMode::NonRecursive
    }));
  }

  #[tokio::test]
  async fn test_find_non_exists_watcher_directories() {
    let current_dir = std::env::current_dir().expect("Failed to get current directory");
    let dir_0 = InternedPath::from(current_dir.join("src"));

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

    path_manager.update(files, dirs, missing).await.unwrap();

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
      path: InternedPath::from(current_dir),
      mode: notify::RecursiveMode::NonRecursive,
    }));
  }
}
