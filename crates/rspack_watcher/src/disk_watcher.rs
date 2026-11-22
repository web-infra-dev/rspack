use std::{path::Path, sync::Arc, time::Duration};

use notify::{
  Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher,
  event::{ModifyKind, RenameMode},
};
use rspack_paths::InternedPath;
use rspack_util::fx_hash::FxHashSet as HashSet;

use crate::{FsEventKind, WatchPattern, analyzer::Analyzer, trigger};

pub(crate) trait WatchBackend: Send {
  fn watch(&mut self, path: &Path, mode: RecursiveMode) -> notify::Result<()>;
  fn unwatch(&mut self, path: &Path) -> notify::Result<()>;
}

impl WatchBackend for RecommendedWatcher {
  fn watch(&mut self, path: &Path, mode: RecursiveMode) -> notify::Result<()> {
    Watcher::watch(self, path, mode)
  }

  fn unwatch(&mut self, path: &Path) -> notify::Result<()> {
    Watcher::unwatch(self, path)
  }
}

pub(crate) fn system_watch_backend(
  follow_symlinks: bool,
  poll_interval: Option<u32>,
  trigger: Arc<trigger::Trigger>,
) -> Box<dyn WatchBackend> {
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

  Box::new(inner)
}

/// `DiskWatcher` is responsible for managing the underlying file system watcher
/// and keeping track of the currently watched paths.
pub struct DiskWatcher {
  /// The actual file system watcher from the `notify` crate.
  inner: Option<Box<dyn WatchBackend>>,
  /// A set of pattern that are currently being watched.
  watch_patterns: HashSet<WatchPattern>,
}

impl DiskWatcher {
  /// Creates a new `DiskWatcher` wrapping the given [`WatchBackend`].
  pub(crate) fn new(backend: Box<dyn WatchBackend>) -> Self {
    DiskWatcher {
      inner: Some(backend),
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
      if is_covered_by_kept_recursive_ancestor(path.as_path(), &self.watch_patterns, &new_patterns)
      {
        continue;
      }
      // A failed unwatch must not stop the new patterns from being watched. For example, on
      // Linux `notify` can return `EINVAL` for a subdirectory that was deleted while its parent
      // was watched recursively (https://github.com/notify-rs/notify/issues/709).
      if let Some(watcher) = &mut self.inner
        && let Err(e) = watcher.unwatch(path.as_path())
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

    let mut first_error: Option<rspack_error::Error> = None;

    for pattern in &new_patterns {
      if pattern.mode != RecursiveMode::NonRecursive || self.watch_patterns.contains(pattern) {
        continue;
      }
      if new_patterns.contains(&WatchPattern {
        path: pattern.path.clone(),
        mode: RecursiveMode::Recursive,
      }) {
        continue;
      }

      if let Some(watcher) = &mut self.inner
        && let Err(e) = watcher.watch(pattern.path.as_path(), RecursiveMode::NonRecursive)
      {
        if first_error.is_none() {
          first_error = Some(rspack_error::error!(e.to_string()));
        }
        continue;
      }

      self.watch_patterns.remove(&WatchPattern {
        path: pattern.path.clone(),
        mode: RecursiveMode::Recursive,
      });
      self.watch_patterns.insert(pattern.clone());
    }

    for pattern in &new_patterns {
      if pattern.mode != RecursiveMode::Recursive {
        continue;
      }
      let non_rec = WatchPattern {
        path: pattern.path.clone(),
        mode: RecursiveMode::NonRecursive,
      };
      if self.watch_patterns.contains(pattern) {
        if new_patterns.contains(&non_rec) {
          self.watch_patterns.insert(non_rec);
        }
        continue;
      }

      if let Some(watcher) = &mut self.inner
        && let Err(e) = watcher.watch(pattern.path.as_path(), RecursiveMode::Recursive)
      {
        if first_error.is_none() {
          first_error = Some(rspack_error::error!(e.to_string()));
        }
        continue;
      }

      self.watch_patterns.insert(pattern.clone());
      if new_patterns.contains(&non_rec) {
        self.watch_patterns.insert(non_rec);
      }
    }

    if let Some(err) = first_error {
      return Err(err);
    }

    Ok(())
  }

  pub fn close(&mut self) {
    // the trigger.tx is dropped in the FsWatcher
    std::mem::drop(self.inner.take());
  }
}

fn is_covered_by_kept_recursive_ancestor(
  path: &Path,
  current: &HashSet<WatchPattern>,
  desired: &HashSet<WatchPattern>,
) -> bool {
  let mut ancestors = path.ancestors();
  ancestors.next();
  for ancestor in ancestors {
    let rec = WatchPattern {
      path: InternedPath::from(ancestor),
      mode: RecursiveMode::Recursive,
    };
    if current.contains(&rec) && desired.contains(&rec) && !has_symlink_between(ancestor, path) {
      return true;
    }
  }
  false
}

fn has_symlink_between(ancestor: &Path, path: &Path) -> bool {
  let Ok(rel) = path.strip_prefix(ancestor) else {
    return true;
  };
  let mut cur = ancestor.to_path_buf();
  for component in rel.components() {
    cur.push(component);
    if let Ok(meta) = std::fs::symlink_metadata(&cur)
      && meta.file_type().is_symlink()
    {
      return true;
    }
  }
  false
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

  for (index, path) in event.paths.into_iter().map(InternedPath::from).enumerate() {
    let kind = match event.kind {
      EventKind::Create(_) | EventKind::Modify(ModifyKind::Name(RenameMode::To)) => {
        FsEventKind::Create
      }
      EventKind::Remove(_) | EventKind::Modify(ModifyKind::Name(RenameMode::From)) => {
        FsEventKind::Remove
      }
      EventKind::Modify(ModifyKind::Name(RenameMode::Both)) => {
        if index == 0 {
          FsEventKind::Remove
        } else {
          FsEventKind::Create
        }
      }
      EventKind::Modify(
        ModifyKind::Data(_) | ModifyKind::Any | ModifyKind::Name(_) | ModifyKind::Metadata(_),
      ) => FsEventKind::Change,
      _ => return,
    };
    let appeared = kind == FsEventKind::Create
      || matches!(
        event.kind,
        EventKind::Modify(ModifyKind::Name(RenameMode::Any))
      );
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
    DiskWatcher::new(system_watch_backend(false, None, trigger))
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
