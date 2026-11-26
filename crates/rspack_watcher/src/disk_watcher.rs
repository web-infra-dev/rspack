use std::{
  collections::BTreeMap,
  ops::Bound::{Included, Unbounded},
  path::{Path, PathBuf},
  sync::Arc,
  time::Duration,
};

use notify::{
  Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher,
  event::{ModifyKind, RenameMode},
};
use rspack_paths::{InternedPath, InternedPathMap};
use rspack_util::fx_hash::FxHashSet as HashSet;

use crate::{FsEventKind, WatchPattern, analyzer::Analyzer, trigger};

/// notify removes descendants when unwatching a recursive inotify root. Keep
/// independently owned, overlapping roots on separate backend instances.
struct WatchGroup {
  watcher: RecommendedWatcher,
  roots: BTreeMap<PathBuf, RecursiveMode>,
}

impl WatchGroup {
  fn new(config: notify::Config, trigger: Arc<trigger::Trigger>) -> notify::Result<Self> {
    let watcher = RecommendedWatcher::new(
      move |result: notify::Result<Event>| match result {
        Ok(event) => handle_event(event, &trigger),
        Err(e) => {
          tracing::error!(target: "rspack_watcher::fs_event", "file watcher error: {e:?}");
          eprintln!("Error in file watcher: {e:?}");
        }
      },
      config,
    )?;
    Ok(Self {
      watcher,
      roots: BTreeMap::new(),
    })
  }

  fn can_mount(&self, path: &Path, mode: RecursiveMode) -> bool {
    if self.roots.contains_key(path)
      || path
        .ancestors()
        .skip(1)
        .any(|parent| self.roots.get(parent) == Some(&RecursiveMode::Recursive))
    {
      return false;
    }
    mode == RecursiveMode::NonRecursive
      || self
        .roots
        .range::<Path, _>((Included(path), Unbounded))
        .next()
        .is_none_or(|(next, _)| !next.starts_with(path))
  }
}

pub struct DiskWatcher {
  groups: Vec<WatchGroup>,
  watch_patterns: HashSet<WatchPattern>,
  config: notify::Config,
  trigger: Arc<trigger::Trigger>,
}

impl DiskWatcher {
  pub fn new(
    follow_symlinks: bool,
    poll_interval: Option<u32>,
    trigger: Arc<trigger::Trigger>,
  ) -> Self {
    let mut config = notify::Config::default().with_follow_symlinks(follow_symlinks);
    if let Some(poll) = poll_interval {
      config = config.with_poll_interval(Duration::from_millis(u64::from(poll)));
    }
    let group =
      WatchGroup::new(config, Arc::clone(&trigger)).expect("Failed to create disk watcher");
    Self {
      groups: vec![group],
      watch_patterns: HashSet::default(),
      config,
      trigger,
    }
  }

  pub fn watch(
    &mut self,
    patterns: impl Iterator<Item = WatchPattern>,
  ) -> rspack_error::Result<()> {
    let mut desired = InternedPathMap::default();
    for pattern in patterns {
      let mode = desired.entry(pattern.path).or_insert(pattern.mode);
      if pattern.mode == RecursiveMode::Recursive {
        *mode = RecursiveMode::Recursive;
      }
    }
    for (path, &mode) in &desired {
      let pattern = WatchPattern {
        path: path.clone(),
        mode,
      };
      if self.watch_patterns.contains(&pattern) {
        continue;
      }
      let group = match self.groups.iter_mut().find(|g| g.can_mount(path, mode)) {
        Some(group) => group,
        None => {
          let group = WatchGroup::new(self.config, Arc::clone(&self.trigger))
            .map_err(|e| rspack_error::error!(e.to_string()))?;
          self.groups.push_mut(group)
        }
      };
      group
        .watcher
        .watch(path, mode)
        .map_err(|e| rspack_error::error!(e.to_string()))?;
      group.roots.insert(path.to_path_buf(), mode);
      self.watch_patterns.insert(pattern);
    }

    // Install every replacement before releasing any old coverage, even on a retry.
    self.watch_patterns.retain(|pattern| {
      if desired.get(&pattern.path) == Some(&pattern.mode) {
        return true;
      }
      let group = self.groups.iter_mut()
        .find(|g| g.roots.get(pattern.path.as_path()) == Some(&pattern.mode))
        .expect("mounted watch group");
      if let Err(e) = group.watcher.unwatch(&pattern.path)
        && !matches!(e.kind, notify::ErrorKind::WatchNotFound) {
        tracing::warn!(target: "rspack_watcher::disk_watcher", path = ?pattern.path, "failed to unwatch stale path: {e:?}");
      }
      group.roots.remove(pattern.path.as_path());
      false
    });
    self.groups.retain(|group| !group.roots.is_empty());
    Ok(())
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
