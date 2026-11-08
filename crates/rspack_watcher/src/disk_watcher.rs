use std::{sync::Arc, time::Duration};

use notify::{
  Event, EventKind, RecommendedWatcher, Watcher,
  event::{ModifyKind, RenameMode},
};
use rspack_paths::InternedPath;
use rspack_util::fx_hash::FxHashSet as HashSet;

use crate::{FsEventKind, WatchPattern, analyzer::Analyzer, trigger};

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
        Ok(event) => handle_event(event, &trigger),

        Err(e) => {
          tracing::error!(target: "rspack_watcher::fs_event", "file watcher error: {e:?}");
          eprintln!("Error in file watcher: {e:?}",);
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
    let new_patterns: HashSet<WatchPattern> = patterns.collect();

    let new_paths = new_patterns.iter().map(|p| &p.path).collect::<HashSet<_>>();

    // Collect stale paths that are no longer needed, then unwatch and remove them.
    let stale_paths: HashSet<InternedPath> = self
      .watch_patterns
      .iter()
      .filter(|p| !new_paths.contains(&p.path))
      .map(|p| p.path.clone())
      .collect();

    for path in &stale_paths {
      // A failed unwatch must not stop the new patterns from being watched. For example, on
      // Linux `notify` can return `EINVAL` for a subdirectory that was deleted while its parent
      // was watched recursively (https://github.com/notify-rs/notify/issues/709).
      if let Some(watcher) = &mut self.inner
        && let Err(e) = watcher.unwatch(path)
        && !matches!(e.kind, notify::ErrorKind::WatchNotFound)
      {
        tracing::warn!(
          target: "rspack_watcher::disk_watcher",
          path = ?path,
          "failed to unwatch stale path: {e:?}",
        );
      }
    }

    self
      .watch_patterns
      .retain(|p| !stale_paths.contains(&p.path));

    for pattern in new_patterns {
      if self.watch_patterns.contains(&pattern) {
        continue;
      }

      if let Some(watcher) = &mut self.inner {
        watcher
          .watch(&pattern.path, pattern.mode)
          .map_err(|e| rspack_error::error!(e.to_string()))?;
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

pub(crate) fn handle_event(event: Event, trigger: &trigger::Trigger) {
  tracing::debug!(
    target: "rspack_watcher::fs_event",
    kind = ?event.kind,
    paths = ?event.paths,
    "fs_event",
  );

  if event.need_rescan() {
    trigger.analyzer.invalidate_all();
  }
  if event.paths.is_empty() {
    return;
  }

  let appeared = matches!(
    event.kind,
    EventKind::Create(_) | EventKind::Modify(ModifyKind::Name(RenameMode::To | RenameMode::Any))
  );
  let kind = match event.kind {
    EventKind::Create(_) => FsEventKind::Create,
    EventKind::Modify(
      ModifyKind::Data(_) | ModifyKind::Any | ModifyKind::Name(_) | ModifyKind::Metadata(_),
    ) => FsEventKind::Change,
    EventKind::Remove(_) => FsEventKind::Remove,
    // TODO: handle this case /path/to/index.js -> /path/to/index.js.map
    // path/to/index.js should be removed, and path/to/index.js.map should be changed
    // Now /path/to/index.js and /path/to/index.js.map will both be changed
    _ => return,
  };
  for path in event.paths.into_iter().map(InternedPath::from) {
    // Content writes cannot change which ancestor directory exists.
    // Invalidate before filtering so suppressed events still refresh watch coverage.
    if !matches!(event.kind, EventKind::Modify(ModifyKind::Data(_))) {
      trigger.analyzer.invalidate_path(&path);
    }
    trigger.on_disk_event(&path, kind, appeared);
  }
}

#[cfg(test)]
mod tests {
  use std::sync::Arc;

  use rspack_paths::InternedPath;
  use tokio::sync::mpsc;

  use super::*;
  use crate::paths::PathManager;

  fn create_disk_watcher() -> DiskWatcher {
    let (tx, _rx) = mpsc::unbounded_channel();
    let path_manager = Arc::new(PathManager::default());
    let trigger = Arc::new(trigger::Trigger::new(path_manager, tx));
    DiskWatcher::new(false, None, trigger)
  }

  #[test]
  fn test_watch_removes_stale_patterns() {
    let temp_dir = tempfile::TempDir::new().unwrap();
    let base = temp_dir.path().canonicalize().unwrap();

    let dir_a = base.join("a");
    let dir_b = base.join("b");
    let dir_c = base.join("c");
    std::fs::create_dir_all(&dir_a).unwrap();
    std::fs::create_dir_all(&dir_b).unwrap();
    std::fs::create_dir_all(&dir_c).unwrap();

    let mut watcher = create_disk_watcher();

    // First watch: {A, B}
    watcher
      .watch(
        vec![
          WatchPattern {
            path: InternedPath::from(dir_a.clone()),
            mode: notify::RecursiveMode::NonRecursive,
          },
          WatchPattern {
            path: InternedPath::from(dir_b.clone()),
            mode: notify::RecursiveMode::NonRecursive,
          },
        ]
        .into_iter(),
      )
      .unwrap();
    assert_eq!(watcher.watch_patterns.len(), 2);

    // Second watch: {B, C} — A should be removed
    watcher
      .watch(
        vec![
          WatchPattern {
            path: InternedPath::from(dir_b.clone()),
            mode: notify::RecursiveMode::NonRecursive,
          },
          WatchPattern {
            path: InternedPath::from(dir_c.clone()),
            mode: notify::RecursiveMode::NonRecursive,
          },
        ]
        .into_iter(),
      )
      .unwrap();

    assert_eq!(watcher.watch_patterns.len(), 2);
    let paths: HashSet<_> = watcher
      .watch_patterns
      .iter()
      .map(|p| p.path.clone())
      .collect();
    assert!(paths.contains(&InternedPath::from(dir_b)));
    assert!(paths.contains(&InternedPath::from(dir_c)));
    assert!(!paths.contains(&InternedPath::from(dir_a)));
  }

  #[cfg(target_os = "linux")]
  #[test]
  fn test_watch_continues_after_unwatch_of_deleted_subdirectory() {
    let temp_dir = tempfile::TempDir::new().unwrap();
    let base = temp_dir.path().canonicalize().unwrap();

    let dir = base.join("dir");
    let other = base.join("other");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::create_dir_all(&other).unwrap();

    let mut watcher = create_disk_watcher();
    watcher
      .watch(
        vec![WatchPattern {
          path: InternedPath::from(dir.clone()),
          mode: notify::RecursiveMode::Recursive,
        }]
        .into_iter(),
      )
      .unwrap();

    // Let the inotify backend add and then drop a watch for the subdirectory.
    let sub = dir.join("sub");
    std::fs::create_dir(&sub).unwrap();
    std::thread::sleep(std::time::Duration::from_millis(200));
    std::fs::remove_dir(&sub).unwrap();
    std::thread::sleep(std::time::Duration::from_millis(200));

    // Unwatching `dir` can fail for the deleted subdirectory; `other` must still be watched.
    watcher
      .watch(
        vec![WatchPattern {
          path: InternedPath::from(other.clone()),
          mode: notify::RecursiveMode::NonRecursive,
        }]
        .into_iter(),
      )
      .unwrap();

    let paths: HashSet<_> = watcher
      .watch_patterns
      .iter()
      .map(|p| p.path.clone())
      .collect();
    assert_eq!(paths.len(), 1);
    assert!(paths.contains(&InternedPath::from(other)));
  }
}
