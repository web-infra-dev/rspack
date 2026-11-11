#[cfg(target_os = "linux")]
use std::os::unix::fs::MetadataExt;
use std::{sync::Arc, time::Duration};

use notify::{
  Event, EventKind, RecommendedWatcher, Watcher,
  event::{ModifyKind, RenameMode},
};
use rspack_paths::{InternedPath, InternedPathMap};
use rspack_util::fx_hash::FxHashSet as HashSet;

use crate::{FsEventKind, WatchPattern, analyzer::Analyzer, trigger};

/// `DiskWatcher` is responsible for managing the underlying file system watcher
/// and keeping track of the currently watched paths.
pub struct DiskWatcher {
  /// The actual file system watcher from the `notify` crate.
  inner: Option<RecommendedWatcher>,
  /// The patterns currently registered with the backend.
  watch_patterns: HashSet<WatchPattern>,
  follow_symlinks: bool,
  #[cfg(target_os = "linux")]
  directory_identities: Option<InternedPathMap<(u64, u64)>>,
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
        .with_follow_symlinks(false)
        .with_poll_interval(Duration::from_millis(u64::from(poll))),
      None => notify::Config::default().with_follow_symlinks(false),
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
      follow_symlinks,
      #[cfg(target_os = "linux")]
      directory_identities: follow_symlinks.then(InternedPathMap::default),
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
    // Register each physical directory once: inotify aliases share a descriptor
    // and registering one overwrites the other's event path.
    let mut new_patterns = InternedPathMap::default();
    for mut pattern in patterns {
      if self.follow_symlinks
        && let Ok(real) = dunce::canonicalize(&pattern.path)
      {
        pattern.path = real.into();
      }
      new_patterns
        .entry(pattern.path)
        .and_modify(|mode| {
          if pattern.mode == notify::RecursiveMode::Recursive {
            *mode = pattern.mode;
          }
        })
        .or_insert(pattern.mode);
    }

    // Collect stale paths that are no longer needed, then unwatch and remove them.
    let stale_paths: HashSet<InternedPath> = self
      .watch_patterns
      .iter()
      .filter(|pattern| new_patterns.get(&pattern.path) != Some(&pattern.mode))
      .map(|pattern| pattern.path.clone())
      .collect();

    #[cfg(target_os = "linux")]
    let identities: Option<InternedPathMap<_>> = self.directory_identities.as_ref().map(|_| {
      new_patterns
        .keys()
        .filter_map(|path| {
          path
            .metadata()
            .ok()
            .map(|metadata| (path.clone(), (metadata.dev(), metadata.ino())))
        })
        .collect()
    });
    #[cfg(target_os = "linux")]
    let stale_paths = {
      let mut stale_paths = stale_paths;
      if let (Some(previous), Some(current)) = (&self.directory_identities, &identities) {
        stale_paths.extend(
          previous
            .iter()
            .filter(|(path, identity)| current.get(*path) != Some(*identity))
            .map(|(path, _)| path.clone()),
        );
      }
      stale_paths
    };

    // Removing a recursive inotify watch also removes its descendants, even
    // when a descendant is still requested by another symbolic link.
    let stale_recursive_paths: HashSet<_> = stale_paths
      .iter()
      .filter(|path| {
        self.watch_patterns.contains(&WatchPattern {
          path: (*path).clone(),
          mode: notify::RecursiveMode::Recursive,
        })
      })
      .cloned()
      .collect();
    let mut stale_paths = stale_paths;
    if !stale_recursive_paths.is_empty() {
      stale_paths.extend(
        self
          .watch_patterns
          .iter()
          .filter(|pattern| {
            pattern
              .path
              .ancestors()
              .skip(1)
              .any(|ancestor| stale_recursive_paths.contains(&InternedPath::from(ancestor)))
          })
          .map(|pattern| pattern.path.clone()),
      );
    }

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
      .retain(|pattern| !stale_paths.contains(&pattern.path));

    for (path, mode) in new_patterns {
      let pattern = WatchPattern { path, mode };
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

    #[cfg(target_os = "linux")]
    {
      self.directory_identities = identities;
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
