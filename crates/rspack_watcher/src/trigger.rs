use std::sync::Arc;

use rspack_paths::{ArcPath, ArcPathDashSet};
use tokio::sync::mpsc::UnboundedSender;

use super::{FsEvent, FsEventKind};
use crate::{EventBatch, paths::PathManager};
/// `DependencyFinder` provides references to sets of files, directories, and missing paths,
/// allowing efficient lookup and dependency resolution for a given path.
///
/// This struct is typically used to determine which registered dependencies (files, directories,
/// or missing paths) are related to a specific filesystem path, such as when handling file system events.
pub struct DependencyFinder<'a> {
  /// Reference to the set of registered file paths.
  pub files: &'a ArcPathDashSet,
  /// Reference to the set of registered directory paths.
  pub directories: &'a ArcPathDashSet,
  /// Reference to the set of registered missing paths (paths that were expected but not found).
  pub missing: &'a ArcPathDashSet,
}

impl<'a> DependencyFinder<'a> {
  /// Finds all registered dependencies related to the given path.
  ///
  /// This method checks if the path is a directory or file and then determines if it is registered
  /// in the dependency sets. If it is a directory, it also recursively adds all parent directories
  /// that are registered as directories or missing.
  pub fn find_associated_event(
    &self,
    path: &ArcPath,
    kind: FsEventKind,
  ) -> Vec<(ArcPath, FsEventKind)> {
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
    } else if self.contains_path(path) {
      // If the path does not exist but is registered as missing, add it to the result.
      paths.push((path.clone(), kind));
    }

    // Recursively add all parent directories that are registered as directories or missing.
    self.recurse_parent_directories(path, &mut paths);

    paths
  }

  /// Checks if the given path is registered as a file or missing.
  fn contains_file(&self, path: &ArcPath) -> bool {
    self.files.contains(path) || self.missing.contains(path)
  }

  /// Checks if the given path is registered as a directory or missing.
  fn contains_directory(&self, path: &ArcPath) -> bool {
    self.directories.contains(path) || self.missing.contains(path)
  }

  fn contains_path(&self, path: &ArcPath) -> bool {
    self.files.contains(path) || self.directories.contains(path) || self.missing.contains(path)
  }

  /// Recursively adds all parent directories that are registered as directories or missing.
  fn recurse_parent_directories(&self, path: &ArcPath, paths: &mut Vec<(ArcPath, FsEventKind)>) {
    match path.parent() {
      Some(parent) => {
        if self.contains_directory(&ArcPath::from(parent)) {
          // For parent directory, it always FsEventKind::Change its recursive children no matter what kind is
          paths.push((ArcPath::from(parent), FsEventKind::Change));
        }
        self.recurse_parent_directories(&ArcPath::from(parent), paths);
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
  tx: UnboundedSender<EventBatch>,
}

impl Trigger {
  /// Create a new `Trigger` with the given path register and event sender.
  pub fn new(path_manager: Arc<PathManager>, tx: UnboundedSender<EventBatch>) -> Self {
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
  pub fn on_event(&self, path: &ArcPath, kind: FsEventKind) {
    if self
      .path_manager
      .ignored
      .should_be_ignored(&path.to_string_lossy())
    {
      return;
    }

    let is_registered_file = self.path_manager.access().files().0.contains(path);

    // Filter stale FSEvents: on macOS, FSEvents can deliver events for files
    // written before the watcher was created. Stat the file and compare mtime
    // against the recorded baseline to suppress events where nothing changed.
    // Apply the same suppression to Create for already-registered files, since
    // macOS may emit stale Create events for files that predate the watcher.
    if (kind == FsEventKind::Change || (kind == FsEventKind::Create && is_registered_file))
      && !self.path_manager.has_mtime_changed(path)
    {
      return;
    }

    let finder = self.finder();
    let associated_event = finder.find_associated_event(path, kind);
    self.trigger_events(associated_event);
  }
  /// Helper to construct a `DependencyFinder` for the current path register state.
  fn finder(&self) -> DependencyFinder<'_> {
    let accessor = self.path_manager.access();

    let files = accessor.files().0;
    let directories = accessor.directories().0;
    let missing = accessor.missing().0;

    DependencyFinder {
      files,
      directories,
      missing,
    }
  }

  /// Sends a group of file system events for the given path and event kind.
  /// If the event is successfully sent, it returns true; otherwise, it returns false.
  fn trigger_events(&self, events: Vec<(ArcPath, FsEventKind)>) -> bool {
    self
      .tx
      .send(
        events
          .into_iter()
          .map(|(path, kind)| FsEvent { path, kind })
          .collect(),
      )
      .is_ok()
  }
}
#[cfg(test)]
mod tests {

  use std::path::Path;

  use rspack_paths::ArcPath;
  use tokio::sync::mpsc;

  use super::*;

  #[test]
  fn test_find_dependency_for_file() {
    let files = ArcPathDashSet::default();
    let directories = ArcPathDashSet::default();
    let missing = ArcPathDashSet::default();

    let file_0 = ArcPath::from(Path::new("/path/a/b/c/index.js"));
    let dir_0 = ArcPath::from(Path::new("/path/a/b"));
    files.insert(file_0.clone());

    directories.insert(dir_0.clone());
    let finder = DependencyFinder {
      files: &files,
      directories: &directories,
      missing: &missing,
    };

    let associated_events = finder.find_associated_event(&file_0, FsEventKind::Remove);

    assert_eq!(associated_events.len(), 2);

    assert!(associated_events.contains(&(file_0, FsEventKind::Remove)));
    assert!(associated_events.contains(&(dir_0, FsEventKind::Change)));
  }

  #[test]
  fn test_find_dependency_for_directory() {
    let files = ArcPathDashSet::default();
    let directories = ArcPathDashSet::default();
    let missing = ArcPathDashSet::default();

    let dir_0 = ArcPath::from(Path::new("/path/a/b/c"));
    let dir_1 = ArcPath::from(Path::new("/path/a/b"));

    directories.insert(dir_0.clone());
    directories.insert(dir_1.clone());

    let finder = DependencyFinder {
      files: &files,
      directories: &directories,
      missing: &missing,
    };

    let associated_events = finder.find_associated_event(
      &ArcPath::from(Path::new("/path/a/b/c/index.js")),
      FsEventKind::Create,
    );
    assert_eq!(associated_events.len(), 2);
    assert!(associated_events.contains(&(dir_0, FsEventKind::Change)));
    assert!(associated_events.contains(&(dir_1, FsEventKind::Change)));
  }

  #[test]
  fn test_trigger_ignores_event_path_before_parent_association() {
    let watched_dir = ArcPath::from(Path::new("/path/project"));
    let ignored_file = ArcPath::from(Path::new("/path/project/.cache/file.js"));
    let path_manager = Arc::new(PathManager::new(crate::FsWatcherIgnored::Mixed(vec![
      crate::FsWatcherIgnoredItem::Path("**/.cache".to_string()),
      crate::FsWatcherIgnoredItem::Regex(
        rspack_regex::RspackRegex::new("generated").expect("valid regex"),
      ),
    ])));
    path_manager
      .update(
        (std::iter::empty(), std::iter::empty()),
        (std::iter::once(watched_dir), std::iter::empty()),
        (std::iter::empty(), std::iter::empty()),
      )
      .expect("update watched directories");
    let (tx, mut rx) = mpsc::unbounded_channel();
    let trigger = Trigger::new(path_manager, tx);

    trigger.on_event(&ignored_file, FsEventKind::Change);

    assert!(rx.try_recv().is_err());
  }

  #[cfg(unix)]
  #[test]
  fn test_trigger_ignores_non_utf8_event_path_under_ignored_parent() {
    use std::{ffi::OsString, os::unix::ffi::OsStringExt, path::PathBuf};

    let watched_dir = ArcPath::from(Path::new("/path/project"));
    let mut ignored_file = PathBuf::from("/path/project/.cache");
    ignored_file.push(OsString::from_vec(vec![b'f', 0x80, b'.', b'j', b's']));
    let path_manager = Arc::new(PathManager::new(crate::FsWatcherIgnored::Mixed(vec![
      crate::FsWatcherIgnoredItem::Path("**/.cache".to_string()),
    ])));
    path_manager
      .update(
        (std::iter::empty(), std::iter::empty()),
        (std::iter::once(watched_dir), std::iter::empty()),
        (std::iter::empty(), std::iter::empty()),
      )
      .expect("update watched directories");
    let (tx, mut rx) = mpsc::unbounded_channel();
    let trigger = Trigger::new(path_manager, tx);

    trigger.on_event(&ArcPath::from(ignored_file), FsEventKind::Change);

    assert!(rx.try_recv().is_err());
  }
}
