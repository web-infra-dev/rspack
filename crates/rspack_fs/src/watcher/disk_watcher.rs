// use std::collections::HashSet;
use notify::{Event, RecommendedWatcher, Watcher};
use rspack_util::fx_hash::FxHashSet as HashSet;

use crate::watcher::{trigger, FsEventKind, WatchPattern};

/// `DiskWatcher` is responsible for managing the underlying file system watcher
/// and keeping track of the currently watched paths.
pub struct DiskWatcher {
  /// The actual file system watcher from the `notify` crate.
  inner: Option<RecommendedWatcher>,
  /// A set of pattern that are currently being watched.
  watch_patterns: HashSet<WatchPattern>,
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
      inner: Some(inner),
      watch_patterns: HashSet::default(),
    }
  }

  /// Watches the given path with the specified recursive mode.
  ///
  /// # Returns
  ///
  /// * `rspack_error::Result<()>` - Ok if successful, otherwise an error.
  pub fn watch(
    &mut self,
    patterns: impl Iterator<Item = WatchPattern>,
  ) -> rspack_error::Result<()> {
    let patterns: HashSet<WatchPattern> = patterns.collect();

    for pattern in self.watch_patterns.difference(&patterns) {
      if let Some(watcher) = &mut self.inner {
        watcher
          .unwatch(&pattern.path)
          .map_err(|e| rspack_error::error!(e))?;
      }
    }

    for pattern in patterns {
      if self.watch_patterns.contains(&pattern) {
        continue;
      }

      if let Some(watcher) = &mut self.inner {
        watcher
          .watch(&pattern.path, pattern.mode)
          .map_err(|e| rspack_error::error!(e))?;
      }

      self.watch_patterns.insert(pattern);
    }

    Ok(())
  }

  pub fn close(&mut self) {
    // the trigger.tx is dropped in the FsWatcher
    std::mem::drop(self.inner.take());
  }
}
