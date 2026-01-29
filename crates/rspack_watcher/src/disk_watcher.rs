use std::{sync::Arc, time::Duration};

use notify::{Event, EventKind, RecommendedWatcher, Watcher, event::ModifyKind};
use rspack_paths::ArcPath;
use rspack_util::fx_hash::FxHashSet as HashSet;

use crate::{FsEventKind, WatchPattern, trigger};

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
  pub fn new(
    follow_symlinks: bool,
    poll_interval: Option<u32>,
    trigger: Arc<trigger::Trigger>,
  ) -> Self {
    let config = match poll_interval {
      Some(poll) => notify::Config::default()
        .with_follow_symlinks(follow_symlinks)
        .with_poll_interval(Duration::from_millis(u64::from(poll))),
      None => notify::Config::default().with_follow_symlinks(follow_symlinks),
    };

    let inner = RecommendedWatcher::new(
      move |result: notify::Result<Event>| match result {
        Ok(event) => {
          let paths = &event.paths;

          if paths.is_empty() {
            println!(
              "[WATCHER_DEBUG] DiskWatcher - Received event with no paths: {:?}",
              event.kind
            );
            return; // Ignore events with no paths
          }

          let kind = match event.kind {
            EventKind::Create(_) => {
              println!(
                "[WATCHER_DEBUG] DiskWatcher - Create event for {} paths",
                paths.len()
              );
              FsEventKind::Create
            }
            EventKind::Modify(
              ModifyKind::Data(_) | ModifyKind::Any | ModifyKind::Name(_) | ModifyKind::Metadata(_),
            ) => {
              println!(
                "[WATCHER_DEBUG] DiskWatcher - Modify event ({:?}) for {} paths",
                event.kind,
                paths.len()
              );
              FsEventKind::Change
            }
            EventKind::Remove(_) => {
              println!(
                "[WATCHER_DEBUG] DiskWatcher - Remove event for {} paths",
                paths.len()
              );
              FsEventKind::Remove
            }
            // TODO: handle this case /path/to/index.js -> /path/to/index.js.map
            // path/to/index.js should be removed, and path/to/index.js.map should be changed
            // Now /path/to/index.js and /path/to/index.js.map will both be changed
            _ => {
              println!(
                "[WATCHER_DEBUG] DiskWatcher - Ignoring event kind: {:?}",
                event.kind
              );
              return; // Ignore other kinds of events
            }
          };
          let paths = event.paths.into_iter().map(ArcPath::from);
          for path in paths {
            println!(
              "[WATCHER_DEBUG] DiskWatcher - Triggering {:?} for path: {:?}",
              kind, path
            );
            trigger.on_event(&path, kind);
          }
        }

        Err(e) => {
          // Handle error, e.g., log it or notify the user
          println!("[WATCHER_DEBUG] DiskWatcher - ERROR in file watcher: {e:?}");
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

    println!(
      "[WATCHER_DEBUG] DiskWatcher::watch() - Updating watch patterns, {} new patterns",
      patterns.len()
    );

    let already_watched_paths = self
      .watch_patterns
      .iter()
      .map(|p| &p.path)
      .collect::<HashSet<_>>();
    let current_should_watch_paths = patterns.iter().map(|p| &p.path).collect::<HashSet<_>>();

    // notify::Watcher only unwatchs the path, so we need to check which paths instead of patterns.
    for pattern in already_watched_paths.difference(&current_should_watch_paths) {
      // If the path is no longer in the patterns to watch, unwatch it
      if let Some(watcher) = &mut self.inner {
        println!(
          "[WATCHER_DEBUG] DiskWatcher::watch() - Unwatching path: {:?}",
          pattern
        );
        // Currently, we unwatch the path even if it might still be in other patterns, as we lack a way to track paths precisely.
        // The `notify` crate automatically removes the watch path when it is removed internally.
        // If we attempt to unwatch the path again, it may return an error.
        // Consider enhancing the tracking of paths to avoid unnecessary `unwatch` calls and handle errors more robustly.
        if let Err(e) = watcher.unwatch(pattern)
          && !matches!(e.kind, notify::ErrorKind::WatchNotFound)
        {
          println!(
            "[WATCHER_DEBUG] DiskWatcher::watch() - ERROR unwatching path {:?}: {}",
            pattern, e
          );
          return Err(rspack_error::error!(e.to_string()));
        }
      }
    }

    for pattern in patterns {
      if self.watch_patterns.contains(&pattern) {
        continue;
      }

      if let Some(watcher) = &mut self.inner {
        println!(
          "[WATCHER_DEBUG] DiskWatcher::watch() - Watching path: {:?}, mode: {:?}",
          pattern.path, pattern.mode
        );
        watcher.watch(&pattern.path, pattern.mode).map_err(|e| {
          println!(
            "[WATCHER_DEBUG] DiskWatcher::watch() - ERROR watching path {:?}: {}",
            pattern.path, e
          );
          rspack_error::error!(e.to_string())
        })?;
      }

      self.watch_patterns.insert(pattern);
    }

    println!(
      "[WATCHER_DEBUG] DiskWatcher::watch() - Now watching {} paths total",
      self.watch_patterns.len()
    );
    for pattern in &self.watch_patterns {
      println!(
        "[WATCHER_DEBUG]   - {:?} ({:?})",
        pattern.path, pattern.mode
      );
    }

    Ok(())
  }

  pub fn close(&mut self) {
    // the trigger.tx is dropped in the FsWatcher
    std::mem::drop(self.inner.take());
  }
}
