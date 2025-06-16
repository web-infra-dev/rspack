use std::{collections::HashSet, path::PathBuf};

use notify::{Event, RecommendedWatcher, Watcher};

use crate::watcher::{trigger, FsEventKind};

/// `DiskWatcher` is responsible for managing the underlying file system watcher
/// and keeping track of the currently watched paths.
pub struct DiskWatcher {
  /// The actual file system watcher from the `notify` crate.
  inner: RecommendedWatcher,
  /// A set of paths that are currently being watched.
  watch_paths: HashSet<PathBuf>,
}

impl DiskWatcher {
  /// Creates a new `DiskWatcher` with the given configuration and trigger.
  pub fn new<'a>(follow_symlinks: bool, trigger: trigger::Trigger) -> Self {
    let config = notify::Config::default().with_follow_symlinks(follow_symlinks);
    let inner = RecommendedWatcher::new(
      move |result: notify::Result<Event>| match result {
        Ok(event) => {
          let kind = match event.kind {
            notify::EventKind::Modify(_) => FsEventKind::Change,
            notify::EventKind::Remove(_) => FsEventKind::Delete,
            // TODO: Handle other kinds of events if needed
            _ => return, // Ignore other kinds of events
          };
          let paths = event.paths;
          for path in paths {
            trigger.on_event(&path, kind);
          }
        }

        Err(e) => {
          eprintln!("Error occurred while watching disk: {}", e);
        }
      },
      config,
    )
    .expect("Failed to create disk watcher");

    DiskWatcher {
      inner,
      watch_paths: HashSet::new(),
    }
  }

  /// Watches the given path with the specified recursive mode.
  ///
  /// This method first checks if the path is already being watched.
  /// It iterates over all currently watched paths:
  ///   - If the path matches the one to be watched, it sets `wathing` to true.
  ///   - Regardless, it unwatches all currently watched paths.
  /// If the path was not already being watched, it starts watching it.
  ///
  /// # Arguments
  ///
  /// * `path` - The path to watch.
  /// * `mode` - The recursive mode for watching.
  ///
  /// # Returns
  ///
  /// * `rspack_error::Result<()>` - Ok if successful, otherwise an error.
  pub fn watch(&mut self, path: &PathBuf, mode: notify::RecursiveMode) -> rspack_error::Result<()> {
    if self.watch_paths.contains(path) {
      return Ok(());
    }

    self
      .inner
      .watch(path, mode)
      .map_err(|e| rspack_error::error!(e))?;
    self.watch_paths.insert(path.clone());

    Ok(())
  }
}
