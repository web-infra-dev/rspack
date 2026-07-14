use std::{path::Path, sync::Arc, time::Duration};

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
          tracing::debug!(
            target: "rspack_watcher::fs_event",
            kind = ?event.kind,
            paths = ?event.paths,
            "fs_event",
          );

          if event.paths.is_empty() {
            return;
          }

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
          for path in event.paths.into_iter().map(ArcPath::from) {
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

    // A changed recursive mode must be unwatched before it is registered again.
    let stale_patterns: Vec<(ArcPath, bool)> = self
      .watch_patterns
      .iter()
      .filter(|p| !new_patterns.contains(*p))
      .map(|p| {
        (
          p.path.clone(),
          matches!(p.mode, notify::RecursiveMode::Recursive),
        )
      })
      .collect();

    for (path, _) in &stale_patterns {
      if let Some(watcher) = &mut self.inner
        && let Err(e) = watcher.unwatch(path)
        && !matches!(e.kind, notify::ErrorKind::WatchNotFound)
      {
        return Err(rspack_error::error!(e.to_string()));
      }
    }

    // notify's inotify backend removes every descendant watch when a recursive
    // parent is unwatched, so retained children must also be registered again.
    let stale_paths: HashSet<&Path> = stale_patterns
      .iter()
      .map(|(path, _)| path.as_ref())
      .collect();
    let stale_recursive_paths: HashSet<&Path> = stale_patterns
      .iter()
      .filter_map(|(path, recursive)| recursive.then_some(path.as_ref()))
      .collect();
    self.watch_patterns.retain(|p| {
      !stale_paths.contains(p.path.as_ref())
        && !p
          .path
          .as_ref()
          .ancestors()
          .any(|path| stale_recursive_paths.contains(path))
    });

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
  use std::{
    sync::Arc,
    time::{Duration, Instant},
  };

  use rspack_paths::ArcPath;
  use tokio::sync::mpsc;

  use super::*;
  use crate::{
    analyzer::{Analyzer, RecommendedAnalyzer},
    paths::PathManager,
  };

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

  #[test]
  fn test_watch_replaces_recursive_mode_for_existing_path() {
    let temp_dir = tempfile::TempDir::new().unwrap();
    let dir = ArcPath::from(temp_dir.path().canonicalize().unwrap());
    let mut watcher = create_disk_watcher();

    watcher
      .watch(std::iter::once(WatchPattern {
        path: dir.clone(),
        mode: notify::RecursiveMode::NonRecursive,
      }))
      .unwrap();
    watcher
      .watch(std::iter::once(WatchPattern {
        path: dir.clone(),
        mode: notify::RecursiveMode::Recursive,
      }))
      .unwrap();

    assert_eq!(watcher.watch_patterns.len(), 1);
    assert!(watcher.watch_patterns.contains(&WatchPattern {
      path: dir.clone(),
      mode: notify::RecursiveMode::Recursive,
    }));

    watcher
      .watch(std::iter::once(WatchPattern {
        path: dir.clone(),
        mode: notify::RecursiveMode::NonRecursive,
      }))
      .unwrap();

    assert_eq!(watcher.watch_patterns.len(), 1);
    assert!(watcher.watch_patterns.contains(&WatchPattern {
      path: dir,
      mode: notify::RecursiveMode::NonRecursive,
    }));
  }

  #[test]
  fn test_many_stale_siblings_keep_retained_children_and_prefix_siblings() {
    let root = std::path::PathBuf::from("/virtual-project");
    let recursive_parent = root.join("package-1");
    let retained_child = recursive_parent.join("retained");
    let prefix_sibling = root.join("package-10").join("retained");
    let retained_siblings = (0..1024)
      .map(|index| root.join(format!("retained-{index}")))
      .collect::<Vec<_>>();
    let stale_siblings = (0..1024)
      .map(|index| root.join(format!("stale-{index}")))
      .collect::<Vec<_>>();

    let pattern = |path, mode| WatchPattern {
      path: ArcPath::from(path),
      mode,
    };
    let mut watcher = create_disk_watcher();
    watcher.inner = None;
    watcher.watch_patterns =
      std::iter::once(pattern(recursive_parent, notify::RecursiveMode::Recursive))
        .chain(std::iter::once(pattern(
          retained_child.clone(),
          notify::RecursiveMode::NonRecursive,
        )))
        .chain(std::iter::once(pattern(
          prefix_sibling.clone(),
          notify::RecursiveMode::NonRecursive,
        )))
        .chain(
          retained_siblings
            .iter()
            .cloned()
            .map(|path| pattern(path, notify::RecursiveMode::NonRecursive)),
        )
        .chain(stale_siblings.into_iter().enumerate().map(|(index, path)| {
          pattern(
            path,
            if index % 2 == 0 {
              notify::RecursiveMode::Recursive
            } else {
              notify::RecursiveMode::NonRecursive
            },
          )
        }))
        .collect();

    let expected: HashSet<WatchPattern> =
      std::iter::once(pattern(retained_child, notify::RecursiveMode::NonRecursive))
        .chain(std::iter::once(pattern(
          prefix_sibling,
          notify::RecursiveMode::NonRecursive,
        )))
        .chain(
          retained_siblings
            .into_iter()
            .map(|path| pattern(path, notify::RecursiveMode::NonRecursive)),
        )
        .collect();

    watcher
      .watch(expected.iter().map(|pattern| WatchPattern {
        path: pattern.path.clone(),
        mode: pattern.mode,
      }))
      .unwrap();

    assert_eq!(watcher.watch_patterns, expected);
  }

  #[test]
  fn test_removing_recursive_parent_keeps_retained_child_observable() {
    let temp_dir = tempfile::TempDir::new().unwrap();
    let parent = temp_dir.path().canonicalize().unwrap();
    let child = parent.join("child");
    let file = child.join("file.txt");
    std::fs::create_dir_all(&child).unwrap();
    std::fs::write(&file, "before").unwrap();

    let (tx, mut rx) = mpsc::unbounded_channel();
    let path_manager = Arc::new(PathManager::default());
    path_manager
      .update(
        (
          std::iter::once(ArcPath::from(file.clone())),
          std::iter::empty(),
        ),
        (std::iter::empty(), std::iter::empty()),
        (std::iter::empty(), std::iter::empty()),
      )
      .unwrap();
    let trigger = Arc::new(trigger::Trigger::new(path_manager, tx));
    let mut watcher = DiskWatcher::new(false, None, trigger);

    watcher
      .watch(
        [
          WatchPattern {
            path: ArcPath::from(parent),
            mode: notify::RecursiveMode::Recursive,
          },
          WatchPattern {
            path: ArcPath::from(child.clone()),
            mode: notify::RecursiveMode::NonRecursive,
          },
        ]
        .into_iter(),
      )
      .unwrap();
    watcher
      .watch(std::iter::once(WatchPattern {
        path: ArcPath::from(child),
        mode: notify::RecursiveMode::NonRecursive,
      }))
      .unwrap();

    std::thread::sleep(Duration::from_millis(100));
    while rx.try_recv().is_ok() {}
    std::fs::write(&file, "after").unwrap();

    let deadline = Instant::now() + Duration::from_secs(10);
    let observed = loop {
      if let Ok(events) = rx.try_recv()
        && events.iter().any(|event| event.path.as_ref() == file)
      {
        break true;
      }
      if Instant::now() >= deadline {
        break false;
      }
      std::thread::sleep(Duration::from_millis(10));
    };

    assert!(observed, "retained child was no longer watched");
  }

  #[test]
  fn test_recursive_context_observes_file_in_new_grandchild() {
    let temp_dir = tempfile::TempDir::new().unwrap();
    let parent = temp_dir.path().canonicalize().unwrap();
    let child = parent.join("child");
    let existing = child.join("existing.txt");
    std::fs::create_dir_all(&child).unwrap();
    std::fs::write(&existing, "existing").unwrap();

    let (tx, mut rx) = mpsc::unbounded_channel();
    let path_manager = Arc::new(PathManager::default());
    path_manager
      .update(
        (std::iter::once(ArcPath::from(existing)), std::iter::empty()),
        (
          std::iter::once(ArcPath::from(parent.clone())),
          std::iter::empty(),
        ),
        (std::iter::empty(), std::iter::empty()),
      )
      .unwrap();
    let trigger = Arc::new(trigger::Trigger::new(path_manager.clone(), tx));
    let mut watcher = DiskWatcher::new(false, None, trigger);

    watcher
      .watch(std::iter::once(WatchPattern {
        path: ArcPath::from(parent.clone()),
        mode: notify::RecursiveMode::Recursive,
      }))
      .unwrap();
    watcher
      .watch(
        RecommendedAnalyzer::default()
          .analyze(path_manager.access())
          .into_iter(),
      )
      .unwrap();

    let grandchild = child.join("new");
    std::thread::sleep(Duration::from_millis(100));
    std::fs::create_dir(&grandchild).unwrap();
    std::thread::sleep(Duration::from_millis(100));
    while rx.try_recv().is_ok() {}
    std::fs::write(grandchild.join("created.txt"), "created").unwrap();

    let deadline = Instant::now() + Duration::from_secs(10);
    let observed = loop {
      if let Ok(events) = rx.try_recv()
        && events.iter().any(|event| event.path.as_ref() == parent)
      {
        break true;
      }
      if Instant::now() >= deadline {
        break false;
      }
      std::thread::sleep(Duration::from_millis(10));
    };

    assert!(
      observed,
      "recursive context did not watch the new grandchild"
    );
  }
}
