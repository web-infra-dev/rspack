use std::{path::PathBuf, sync::Arc};

use dashmap::DashSet as HashSet;

use super::StdSender;
use crate::watcher::register::PathRegister;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FsEventKind {
  Change,
  Delete,
}

#[derive(Debug, Clone)]
pub struct FsEvent {
  pub path: PathBuf,
  pub kind: FsEventKind,
}

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
  pub fn find_dependency(&self, path: &PathBuf) -> Vec<PathBuf> {
    let mut paths = Vec::new();

    // If the given path is a directory and is registered as a directory, add it to the result.
    if path.is_dir() && self.contains_directory(path) {
      paths.push(path.clone());
    }

    // If the given path is a file and is registered as a file, add it to the result.
    if path.is_file() && self.contains_file(path) {
      paths.push(path.clone());
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

  /// Recursively adds all parent directories that are registered as directories or missing.
  fn recursiron_directories(&self, path: &PathBuf, paths: &mut Vec<PathBuf>) {
    match path.parent() {
      Some(parent) => {
        let parent = parent.to_path_buf();
        if self.contains_directory(&parent) {
          paths.push(parent.to_path_buf());
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
  path_register: Arc<PathRegister>,
  /// Sender for communicating file system events to the watcher executor.
  tx: StdSender<FsEvent>,
}

impl Trigger {
  /// Create a new `Trigger` with the given path register and event sender.
  pub fn new(register: Arc<PathRegister>, tx: StdSender<FsEvent>) -> Self {
    Self {
      path_register: register,
      tx,
    }
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
    let dependencies = finder.find_dependency(path);
    for dep in dependencies {
      self.trigger_event(dep, kind);
    }
  }

  /// Helper to construct a `DependencyFinder` for the current path register state.
  fn finder(&self) -> DependencyFinder<'_> {
    DependencyFinder {
      files: self.path_register.files(),
      directories: self.path_register.directories(),
      missing: self.path_register.missing(),
    }
  }

  /// Sends a file system event for the given path and event kind.
  /// Ignores any error if the receiver has been dropped.
  fn trigger_event(&self, path: PathBuf, kind: FsEventKind) {
    let event = FsEvent { path: path, kind };
    _ = self.tx.send(event);
  }
}
