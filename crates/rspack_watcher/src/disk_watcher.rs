use std::sync::{Arc, Mutex};

use notify::{
  Event, EventKind, RecommendedWatcher, Watcher,
  event::{ModifyKind, RenameMode},
};
use rspack_paths::{InternedPath, InternedPathSet};
use rspack_util::fx_hash::FxHashSet as HashSet;
use tokio::sync::mpsc;

use crate::{FsEventKind, WatchPattern, trigger};

/// `DiskWatcher` is responsible for managing the underlying file system watcher
/// and keeping track of the currently watched paths.
pub struct DiskWatcher {
  /// The actual file system watcher from the `notify` crate.
  inner: Option<RecommendedWatcher>,
  /// A set of pattern that are currently being watched.
  watch_patterns: HashSet<WatchPattern>,
  watched_roots: Arc<Mutex<WatchRoots>>,
  root_events: mpsc::UnboundedReceiver<RootEvent>,
}

#[derive(Default)]
struct WatchRoots {
  watched: InternedPathSet,
  recovery: InternedPathSet,
}

pub(crate) struct RootEvent {
  pub path: InternedPath,
  pub kind: FsEventKind,
  pub appeared: bool,
}

impl DiskWatcher {
  /// Creates a new `DiskWatcher` with the given configuration and trigger.
  pub fn new(
    follow_symlinks: bool,
    poll_interval: Option<u32>,
    trigger: Arc<trigger::Trigger>,
  ) -> Self {
    let watched_roots = Arc::new(Mutex::new(WatchRoots::default()));
    let (root_tx, root_events) = mpsc::unbounded_channel();
    if poll_interval.is_some_and(|interval| interval > 0) {
      return Self {
        inner: None,
        watch_patterns: HashSet::default(),
        watched_roots,
        root_events,
      };
    }
    let config = notify::Config::default().with_follow_symlinks(follow_symlinks);

    let callback_roots = Arc::clone(&watched_roots);
    let inner = RecommendedWatcher::new(
      move |result: notify::Result<Event>| match result {
        Ok(event) => {
          tracing::debug!(
            target: "rspack_watcher::fs_event",
            kind = ?event.kind,
            paths = ?event.paths,
            "fs_event",
          );

          if event.paths.is_empty() {
            return;
          }

          let appeared = matches!(
            event.kind,
            EventKind::Create(_)
              | EventKind::Modify(ModifyKind::Name(RenameMode::To | RenameMode::Any))
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
          let invalidates_root = cfg!(target_os = "linux")
            && matches!(
              event.kind,
              EventKind::Remove(_)
                | EventKind::Modify(ModifyKind::Name(RenameMode::From | RenameMode::Any))
            );
          for path in event.paths.into_iter().map(InternedPath::from) {
            let refresh = if cfg!(target_os = "linux") && (invalidates_root || appeared) {
              let roots = callback_roots.lock().expect("watch roots lock poisoned");
              (invalidates_root && roots.watched.contains(&path))
                || (appeared && roots.recovery.contains(&path))
            } else {
              false
            };
            if refresh {
              let _ = root_tx.send(RootEvent {
                path,
                kind,
                appeared,
              });
            } else {
              trigger.on_disk_event(&path, kind, appeared);
            }
          }
        }

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
      watched_roots,
      root_events,
    }
  }

  pub async fn next_root_event(&mut self) -> Option<RootEvent> {
    self.root_events.recv().await
  }

  pub fn set_recovery_paths(&mut self, paths: InternedPathSet) {
    self
      .watched_roots
      .lock()
      .expect("watch roots lock poisoned")
      .recovery = paths;
  }

  pub fn invalidate_root(&mut self, path: &InternedPath) -> rspack_error::Result<()> {
    if !self
      .watch_patterns
      .iter()
      .any(|pattern| &pattern.path == path)
    {
      return Ok(());
    }
    if let Some(watcher) = &mut self.inner
      && let Err(error) = watcher.unwatch(path)
      && !matches!(error.kind, notify::ErrorKind::WatchNotFound)
    {
      return Err(rspack_error::error!(error.to_string()));
    }
    self.watch_patterns.retain(|pattern| &pattern.path != path);
    self
      .watched_roots
      .lock()
      .expect("watch roots lock poisoned")
      .watched
      .remove(path);
    Ok(())
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
    if cfg!(target_os = "linux") {
      // The callback can run as soon as the backend attaches a watch.
      self
        .watched_roots
        .lock()
        .expect("watch roots lock poisoned")
        .watched = new_patterns
        .iter()
        .map(|pattern| pattern.path.clone())
        .collect();
    }

    // Collect stale paths that are no longer needed, then unwatch and remove them.
    let stale_paths: HashSet<InternedPath> = self
      .watch_patterns
      .iter()
      .filter(|p| !new_patterns.contains(*p))
      .map(|p| p.path.clone())
      .collect();

    for path in &stale_paths {
      if let Some(watcher) = &mut self.inner
        && let Err(e) = watcher.unwatch(path)
        && !matches!(e.kind, notify::ErrorKind::WatchNotFound)
      {
        return Err(rspack_error::error!(e.to_string()));
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

#[cfg(test)]
mod tests {
  use std::sync::Arc;

  use rspack_paths::InternedPath;
  use tokio::sync::mpsc;

  use super::*;

  fn create_disk_watcher() -> DiskWatcher {
    let (tx, _rx) = mpsc::unbounded_channel();
    let state = Arc::new(crate::WatcherManager::default());
    let trigger = Arc::new(trigger::Trigger::new(state, tx));
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
    for mode in [
      notify::RecursiveMode::Recursive,
      notify::RecursiveMode::NonRecursive,
    ] {
      watcher
        .watch(std::iter::once(WatchPattern {
          path: InternedPath::from(dir_b.clone()),
          mode,
        }))
        .unwrap();
      assert_eq!(
        watcher.watch_patterns.len(),
        1,
        "changing mode replaces the old pattern"
      );
      assert!(watcher.watch_patterns.contains(&WatchPattern {
        path: InternedPath::from(dir_b.clone()),
        mode
      }));
    }
    assert!(paths.contains(&InternedPath::from(dir_b)));
    assert!(paths.contains(&InternedPath::from(dir_c)));
    assert!(!paths.contains(&InternedPath::from(dir_a)));
  }
}
