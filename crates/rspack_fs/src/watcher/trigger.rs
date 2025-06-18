use std::{path::PathBuf, sync::Arc};

use dashmap::DashSet as HashSet;

use super::{FsEvent, FsEventKind, StdSender};
use crate::watcher::manager::PathManager;
/// `DependencyFinder` provides references to sets of files, directories, and missing paths,
/// allowing efficient lookup and dependency resolution for a given path.
///
/// This struct is typically used to determine which registered dependencies (files, directories,
/// or missing paths) are related to a specific filesystem path, such as when handling file system events.
pub struct DependencyFinder<'a> {
  /// Reference to the set of registered file paths.
  pub files: &'a HashSet<PathBuf>,
  /// Reference to the set of registered directory paths.
  pub directories: &'a HashSet<PathBuf>,
  /// Reference to the set of registered missing paths (paths that were expected but not found).
  pub missing: &'a HashSet<PathBuf>,
}

impl<'a> DependencyFinder<'a> {
  /// Finds all registered dependencies related to the given path.
  ///
  /// This method checks if the path is a directory or file and then determines if it is registered
  /// in the dependency sets. If it is a directory, it also recursively adds all parent directories
  /// that are registered as directories or missing.
  pub fn find_associated_event(
    &self,
    path: &PathBuf,
    kind: FsEventKind,
  ) -> Vec<(PathBuf, FsEventKind)> {
    let mut paths = Vec::new();

    if path.exists() {
      // If the given path is a directory and is registered as a directory, add it to the result.
      if path.is_dir() && self.contains_directory(path) {
        paths.push((path.clone(), kind));
      }

      // If the given path is a file and is registered as a file, add it to the result.
      if path.is_file() && self.contains_file(path) {
        paths.push((path.clone(), kind));
      }
    } else {
      if self.contains_path(path) {
        // If the path does not exist but is registered as missing, add it to the result.
        paths.push((path.clone(), kind));
      }
    }

    // Recursively add all parent directories that are registered as directories or missing.
    self.recursiron_directories(path, &mut paths);

    paths
  }

  /// Checks if the given path is registered as a file or missing.
  fn contains_file(&self, path: &PathBuf) -> bool {
    self.files.contains(path) || self.missing.contains(path)
  }

  /// Checks if the given path is registered as a directory or missing.
  fn contains_directory(&self, path: &PathBuf) -> bool {
    self.directories.contains(path) || self.missing.contains(path)
  }

  fn contains_path(&self, path: &PathBuf) -> bool {
    self.files.contains(path) || self.directories.contains(path) || self.missing.contains(path)
  }

  /// Recursively adds all parent directories that are registered as directories or missing.
  fn recursiron_directories(&self, path: &PathBuf, paths: &mut Vec<(PathBuf, FsEventKind)>) {
    match path.parent() {
      Some(parent) => {
        let parent = parent.to_path_buf();
        if self.contains_directory(&parent) {
          // For parent directory, it always FsEventKind::Change its recursive children no matter what kind is
          paths.push((parent.to_path_buf(), FsEventKind::Change));
        }
        self.recursiron_directories(&parent, paths);
      }
      None => {
        // Reached the root directory, stop recursion
      }
    }
  }
}

/// `Trigger` is responsible for sending file system events to the event channel
/// when a relevant file or directory change is detected.
pub struct Trigger {
  /// Shared reference to the path register, which tracks watched files/directories/missing.
  path_manager: Arc<PathManager>,
  /// Sender for communicating file system events to the watcher executor.
  tx: StdSender<FsEvent>,
}

impl Trigger {
  /// Create a new `Trigger` with the given path register and event sender.
  pub fn new(path_manager: Arc<PathManager>, tx: StdSender<FsEvent>) -> Self {
    Self { path_manager, tx }
  }

  /// Called when a file system event occurs.
  /// Finds all dependencies related to the given path and triggers events for each.
  /// # Example
  /// Consider the following path register:
  ///
  /// ```bash
  /// - /path
  /// - /path/to
  /// - /path/to/file.js
  /// ```
  /// If the file `/path/to/file.js` is changed, the trigger will send an event for the following paths:
  /// - `/path`
  /// - `/path/to`
  pub fn on_event(&self, path: &PathBuf, kind: FsEventKind) {
    let finder = self.finder();
    let associated_event = finder.find_associated_event(path, kind);
    for (path, kind) in associated_event {
      self.trigger_event(path, kind);
    }
  }

  /// Helper to construct a `DependencyFinder` for the current path register state.
  fn finder(&self) -> DependencyFinder<'_> {
    let accessor = self.path_manager.access();

    DependencyFinder {
      files: accessor.files(),
      directories: accessor.directories(),
      missing: accessor.missing(),
    }
  }

  /// Sends a file system event for the given path and event kind.
  /// Ignores any error if the receiver has been dropped.
  fn trigger_event(&self, path: PathBuf, kind: FsEventKind) {
    let event = FsEvent { path: path, kind };
    _ = self.tx.send(event);
  }
}
#[cfg(test)]
mod tests {

  // use std::path::Path;

  use super::*;

  #[test]
  fn test_find_dependency_for_file() {
    let files = HashSet::new();
    let directories = HashSet::new();
    let missing = HashSet::new();

    files.insert(PathBuf::from("/path/a/b/index.js"));
    directories.insert(PathBuf::from("/path/a/b"));
    let finder = DependencyFinder {
      files: &files,
      directories: &directories,
      missing: &missing,
    };

    let associated_events =
      finder.find_associated_event(&PathBuf::from("/path/a/b/index.js"), FsEventKind::Remove);

    assert_eq!(associated_events.len(), 2);

    assert!(associated_events.contains(&(PathBuf::from("/path/a/b/index.js"), FsEventKind::Remove)));
    assert!(associated_events.contains(&(PathBuf::from("/path/a/b"), FsEventKind::Change)));
  }

  #[test]
  fn test_find_dependency_for_directory() {
    let files = HashSet::new();
    let directories = HashSet::new();
    let missing = HashSet::new();

    let current_dir = std::env::current_dir().expect("Failed to get current directory");
    directories.insert(PathBuf::from("/path/a/b"));
    directories.insert(PathBuf::from("/path/a/b/c"));

    let finder = DependencyFinder {
      files: &files,
      directories: &directories,
      missing: &missing,
    };

    let associated_events =
      finder.find_associated_event(&PathBuf::from("/path/a/b/c/index.js"), FsEventKind::Create);
    assert_eq!(associated_events.len(), 2);
    assert!(associated_events.contains(&(PathBuf::from("/path/a/b/c"), FsEventKind::Change)));
    assert!(associated_events.contains(&(PathBuf::from("/path/a/b"), FsEventKind::Change)));
  }
}
