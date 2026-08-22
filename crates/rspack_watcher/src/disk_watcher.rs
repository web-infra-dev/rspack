use std::{sync::Arc, time::Duration};

use notify::{
  Event, EventKind, RecommendedWatcher, Watcher,
  event::{ModifyKind, RenameMode},
};
use rspack_paths::ArcPath;
use rspack_util::fx_hash::FxHashSet as HashSet;

use crate::{FsEventKind, WatchPattern, trigger};

/// Translate a `notify::Event` into `(path, kind)` pairs, splitting renames
/// so the source path becomes `Remove` and the destination becomes `Create`
fn map_notify_event_to_fs_events(event: Event) -> Vec<(ArcPath, FsEventKind)> {
  if event.paths.is_empty() {
    return Vec::new();
  }

  match event.kind {
    EventKind::Create(_) => event
      .paths
      .into_iter()
      .map(|p| (ArcPath::from(p), FsEventKind::Create))
      .collect(),
    EventKind::Remove(_) => event
      .paths
      .into_iter()
      .map(|p| (ArcPath::from(p), FsEventKind::Remove))
      .collect(),
    EventKind::Modify(ModifyKind::Name(RenameMode::From)) => event
      .paths
      .into_iter()
      .map(|p| (ArcPath::from(p), FsEventKind::Remove))
      .collect(),
    EventKind::Modify(ModifyKind::Name(RenameMode::To)) => event
      .paths
      .into_iter()
      .map(|p| (ArcPath::from(p), FsEventKind::Create))
      .collect(),
    EventKind::Modify(ModifyKind::Name(RenameMode::Both)) => {
      // `RenameMode::Both` paths are ordered `(from, to)` per notify docs
      let mut iter = event.paths.into_iter();
      let mut result = Vec::with_capacity(2);
      if let Some(from) = iter.next() {
        result.push((ArcPath::from(from), FsEventKind::Remove));
      }
      if let Some(to) = iter.next() {
        result.push((ArcPath::from(to), FsEventKind::Create));
      }
      result
    }
    EventKind::Modify(
      ModifyKind::Data(_)
      | ModifyKind::Any
      | ModifyKind::Name(RenameMode::Any | RenameMode::Other)
      | ModifyKind::Metadata(_),
    ) => event
      .paths
      .into_iter()
      .map(|p| (ArcPath::from(p), FsEventKind::Change))
      .collect(),
    _ => Vec::new(),
  }
}

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
          tracing::debug!(
            target: "rspack_watcher::fs_event",
            kind = ?event.kind,
            paths = ?event.paths,
            "fs_event",
          );

          for (path, kind) in map_notify_event_to_fs_events(event) {
            trigger.on_event(&path, kind);
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
    let stale_paths: HashSet<ArcPath> = self
      .watch_patterns
      .iter()
      .filter(|p| !new_paths.contains(&p.path))
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

  use rspack_paths::ArcPath;
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
            path: ArcPath::from(dir_a.clone()),
            mode: notify::RecursiveMode::NonRecursive,
          },
          WatchPattern {
            path: ArcPath::from(dir_b.clone()),
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
            path: ArcPath::from(dir_b.clone()),
            mode: notify::RecursiveMode::NonRecursive,
          },
          WatchPattern {
            path: ArcPath::from(dir_c.clone()),
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
    assert!(paths.contains(&ArcPath::from(dir_b)));
    assert!(paths.contains(&ArcPath::from(dir_c)));
    assert!(!paths.contains(&ArcPath::from(dir_a)));
  }

  fn make_event(kind: EventKind, paths: Vec<&str>) -> Event {
    let mut event = Event::new(kind);
    for p in paths {
      event = event.add_path(std::path::PathBuf::from(p));
    }
    event
  }

  #[test]
  fn test_map_rename_from_emits_remove() {
    let event = make_event(
      EventKind::Modify(ModifyKind::Name(RenameMode::From)),
      vec!["/path/to/index.js"],
    );
    let result = map_notify_event_to_fs_events(event);
    assert_eq!(
      result,
      vec![(
        ArcPath::from(std::path::PathBuf::from("/path/to/index.js")),
        FsEventKind::Remove
      )]
    );
  }

  #[test]
  fn test_map_rename_to_emits_create() {
    let event = make_event(
      EventKind::Modify(ModifyKind::Name(RenameMode::To)),
      vec!["/path/to/index.js.map"],
    );
    let result = map_notify_event_to_fs_events(event);
    assert_eq!(
      result,
      vec![(
        ArcPath::from(std::path::PathBuf::from("/path/to/index.js.map")),
        FsEventKind::Create
      )]
    );
  }

  #[test]
  fn test_map_rename_both_splits_into_remove_and_create() {
    let event = make_event(
      EventKind::Modify(ModifyKind::Name(RenameMode::Both)),
      vec!["/path/to/index.js", "/path/to/index.js.map"],
    );
    let result = map_notify_event_to_fs_events(event);
    assert_eq!(
      result,
      vec![
        (
          ArcPath::from(std::path::PathBuf::from("/path/to/index.js")),
          FsEventKind::Remove
        ),
        (
          ArcPath::from(std::path::PathBuf::from("/path/to/index.js.map")),
          FsEventKind::Create
        ),
      ]
    );
  }

  #[test]
  fn test_map_rename_any_falls_back_to_change() {
    let event = make_event(
      EventKind::Modify(ModifyKind::Name(RenameMode::Any)),
      vec!["/path/to/file"],
    );
    let result = map_notify_event_to_fs_events(event);
    assert_eq!(
      result,
      vec![(
        ArcPath::from(std::path::PathBuf::from("/path/to/file")),
        FsEventKind::Change
      )]
    );
  }

  #[test]
  fn test_map_data_modify_is_change() {
    use notify::event::DataChange;
    let event = make_event(
      EventKind::Modify(ModifyKind::Data(DataChange::Content)),
      vec!["/path/to/file"],
    );
    let result = map_notify_event_to_fs_events(event);
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].1, FsEventKind::Change);
  }

  #[test]
  fn test_map_create_and_remove_pass_through() {
    use notify::event::{CreateKind, RemoveKind};
    let create = make_event(EventKind::Create(CreateKind::File), vec!["/a"]);
    assert_eq!(
      map_notify_event_to_fs_events(create),
      vec![(
        ArcPath::from(std::path::PathBuf::from("/a")),
        FsEventKind::Create
      )]
    );

    let remove = make_event(EventKind::Remove(RemoveKind::File), vec!["/a"]);
    assert_eq!(
      map_notify_event_to_fs_events(remove),
      vec![(
        ArcPath::from(std::path::PathBuf::from("/a")),
        FsEventKind::Remove
      )]
    );
  }

  #[test]
  fn test_map_empty_paths_returns_empty() {
    let event = make_event(
      EventKind::Modify(ModifyKind::Name(RenameMode::Both)),
      vec![],
    );
    assert!(map_notify_event_to_fs_events(event).is_empty());
  }
}
