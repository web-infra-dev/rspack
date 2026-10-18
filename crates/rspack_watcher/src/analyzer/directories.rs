#![allow(unused)]
use std::sync::Mutex;

use rspack_paths::{InternedPath, InternedPathMap, InternedPathSet};

use super::{Analyzer, WatchPattern};
use crate::registry::PathAccessor;

/// Track subscription deltas by their directory. Files and missing paths share
/// their parent watch; contexts require recursive coverage of their contents.
#[derive(Default)]
pub struct WatcherDirectoriesAnalyzer {
  state: Mutex<DirectoryAnalyzerState>,
}

#[derive(Default)]
struct DirectoryAnalyzerState {
  registrations: [InternedPathSet; 3],
  directories: InternedPathMap<DirectorySubscriptions>,
  recovery_paths: InternedPathSet,
}

#[derive(Default)]
struct DirectorySubscriptions {
  files: usize,
  contexts: usize,
}

impl Analyzer for WatcherDirectoriesAnalyzer {
  fn analyze<'a>(&self, path_accessor: PathAccessor<'a>) -> Vec<WatchPattern> {
    let mut state = self.state.lock().expect("directory analyzer lock poisoned");
    let DirectoryAnalyzerState {
      registrations,
      directories,
      recovery_paths,
    } = &mut *state;
    for (index, (all, added, removed)) in [
      path_accessor.files(),
      path_accessor.directories(),
      path_accessor.missing(),
    ]
    .into_iter()
    .enumerate()
    {
      for path in added.iter().chain(removed.iter()) {
        let registered = all.contains(&*path);
        let changed = if registered {
          registrations[index].insert(path.clone())
        } else {
          registrations[index].remove(&*path)
        };
        if !changed {
          continue;
        }
        let directory = if index == 1 {
          path.clone()
        } else {
          path
            .parent()
            .map_or_else(|| path.clone(), InternedPath::from)
        };
        let subscriptions = directories.entry(directory).or_default();
        let count = if index == 1 {
          &mut subscriptions.contexts
        } else {
          &mut subscriptions.files
        };
        if registered {
          *count += 1;
        } else {
          *count -= 1;
        }
      }
    }
    directories.retain(|_, subscriptions| subscriptions.files + subscriptions.contexts > 0);
    recovery_paths.clear();
    let mut patterns = InternedPathMap::<bool>::default();
    for (directory, subscriptions) in directories.iter() {
      if let Some((existing, ancestor)) = find_existing_directory(directory.clone()) {
        if ancestor
          && let Some(missing) = directory
            .ancestors()
            .find(|path| path.parent() == Some(&*existing))
        {
          recovery_paths.insert(InternedPath::from(missing));
        }
        *patterns.entry(existing).or_default() |= subscriptions.contexts > 0 && !ancestor;
      }
    }
    patterns
      .iter()
      .filter(|(path, _)| {
        !path
          .ancestors()
          .skip(1)
          .any(|ancestor| patterns.get(&InternedPath::from(ancestor)) == Some(&true))
      })
      .map(|(path, recursive)| WatchPattern {
        path: path.clone(),
        mode: if *recursive {
          notify::RecursiveMode::Recursive
        } else {
          notify::RecursiveMode::NonRecursive
        },
      })
      .collect()
  }

  fn recovery_paths(&self) -> InternedPathSet {
    self
      .state
      .lock()
      .expect("directory analyzer lock poisoned")
      .recovery_paths
      .clone()
  }
}

fn find_existing_directory(mut directory: InternedPath) -> Option<(InternedPath, bool)> {
  let mut ancestor = false;
  while !directory.is_dir() {
    directory = InternedPath::from(directory.parent()?);
    ancestor = true;
  }
  Some((directory, ancestor))
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::registry::Registry;

  #[tokio::test]
  async fn test_find_watch_directories() {
    let current_dir = std::env::current_dir().expect("Failed to get current directory");
    let registry = Registry::default();
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

    registry.update(files, dirs, missing).await.unwrap();
    let analyzer = WatcherDirectoriesAnalyzer::default();
    let watch_patterns = analyzer.analyze(registry.access());

    assert_eq!(watch_patterns.len(), 2);
    assert!(watch_patterns.contains(&{
      WatchPattern {
        path: InternedPath::from(current_dir.clone()),
        mode: notify::RecursiveMode::NonRecursive,
      }
    }));
    assert!(watch_patterns.contains(&WatchPattern {
      path: InternedPath::from(current_dir.join("src")),
      mode: notify::RecursiveMode::Recursive
    }));
  }

  #[tokio::test]
  async fn test_find_non_exists_watcher_directories() {
    let current_dir = std::env::current_dir().expect("Failed to get current directory");
    let dir_0 = InternedPath::from(current_dir.join("src"));

    let registry = Registry::default();
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

    registry.update(files, dirs, missing).await.unwrap();

    let analyzer = WatcherDirectoriesAnalyzer::default();
    let watch_patterns = analyzer.analyze(registry.access());

    assert_eq!(watch_patterns.len(), 2);
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
