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
  requirements: InternedPathMap<WatchRequirement>,
  recovering: InternedPathMap<InternedPathSet>,
  dirty: InternedPathSet,
  patterns: Vec<WatchPattern>,
}

#[derive(Default)]
struct DirectorySubscriptions {
  files: usize,
  contexts: usize,
  resolution: Option<DirectoryResolution>,
}

struct DirectoryResolution {
  path: InternedPath,
  recursive: bool,
  recovery: Option<InternedPath>,
}

#[derive(Default)]
struct WatchRequirement {
  directories: InternedPathSet,
  recursive: usize,
}

impl DirectoryAnalyzerState {
  fn refresh_directory(&mut self, directory: &InternedPath) {
    let subscriptions = self
      .directories
      .get_mut(directory)
      .expect("subscribed directory");
    let active = subscriptions.files + subscriptions.contexts > 0;
    let context = subscriptions.contexts > 0;
    if let Some(old) = subscriptions.resolution.take() {
      let requirement = self
        .requirements
        .get_mut(&old.path)
        .expect("watch requirement");
      requirement.directories.remove(directory);
      requirement.recursive -= usize::from(old.recursive);
      if requirement.directories.is_empty() {
        self.requirements.remove(&old.path);
      }
      if let Some(recovery) = old.recovery {
        let directories = self
          .recovering
          .get_mut(&recovery)
          .expect("recovering directory");
        directories.remove(directory);
        if directories.is_empty() {
          self.recovering.remove(&recovery);
        }
      }
    }
    if !active {
      self.directories.remove(directory);
      return;
    }
    let resolution = find_existing_directory(directory.clone()).map(|(path, ancestor)| {
      let recovery = ancestor.then(|| {
        InternedPath::from(
          directory
            .ancestors()
            .find(|missing| missing.parent() == Some(&*path))
            .expect("missing child"),
        )
      });
      let recursive = context && !ancestor;
      let requirement = self.requirements.entry(path.clone()).or_default();
      requirement.directories.insert(directory.clone());
      requirement.recursive += usize::from(recursive);
      if let Some(recovery) = &recovery {
        let directories = self.recovering.entry(recovery.clone()).or_default();
        directories.insert(directory.clone());
      }
      DirectoryResolution {
        path,
        recursive,
        recovery,
      }
    });
    self
      .directories
      .get_mut(directory)
      .expect("subscribed directory")
      .resolution = resolution;
  }
}

impl Analyzer for WatcherDirectoriesAnalyzer {
  fn analyze<'a>(&self, path_accessor: PathAccessor<'a>) -> Vec<WatchPattern> {
    let mut state = self.state.lock().expect("directory analyzer lock poisoned");
    let DirectoryAnalyzerState {
      registrations,
      directories,
      dirty,
      ..
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
        let subscriptions = directories.entry(directory.clone()).or_default();
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
        dirty.insert(directory);
      }
    }
    if !state.dirty.is_empty() {
      for directory in std::mem::take(&mut state.dirty) {
        state.refresh_directory(&directory);
      }
      state.patterns = state
        .requirements
        .iter()
        .filter(|(path, _)| {
          !path.ancestors().skip(1).any(|ancestor| {
            state
              .requirements
              .get(&InternedPath::from(ancestor))
              .is_some_and(|requirement| requirement.recursive > 0)
          })
        })
        .map(|(path, requirement)| WatchPattern {
          path: path.clone(),
          mode: if requirement.recursive > 0 {
            notify::RecursiveMode::Recursive
          } else {
            notify::RecursiveMode::NonRecursive
          },
        })
        .collect();
    }
    state
      .patterns
      .iter()
      .map(|pattern| WatchPattern {
        path: pattern.path.clone(),
        mode: pattern.mode,
      })
      .collect()
  }

  fn invalidate_directory(&self, path: &InternedPath) {
    let mut state = self.state.lock().expect("directory analyzer lock poisoned");
    let affected: InternedPathSet = state
      .requirements
      .iter()
      .filter(|(root, _)| root.starts_with(path.as_ref() as &std::path::Path))
      .flat_map(|(_, requirement)| requirement.directories.iter())
      .chain(state.recovering.get(path).into_iter().flatten())
      .cloned()
      .collect();
    state.dirty.extend(affected);
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
