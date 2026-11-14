#[cfg(target_os = "linux")]
use std::sync::{Weak, atomic::Ordering};
use std::{
  sync::{Arc, Mutex, atomic::AtomicBool},
  time::Duration,
};

use notify::{
  Event, EventKind, RecommendedWatcher,
  RecursiveMode::{NonRecursive, Recursive},
  Watcher,
  event::{ModifyKind, RenameMode},
};
use rspack_paths::{InternedPath, InternedPathMap};
use rspack_util::fx_hash::FxHashSet as HashSet;

use crate::{FsEventKind, WatchPattern, analyzer::Analyzer, trigger};

#[cfg(target_os = "linux")]
static CONTEXTS_DISABLED: AtomicBool = AtomicBool::new(false);
#[cfg(target_os = "linux")]
static LIVE: Mutex<Vec<Weak<Mutex<DiskWatcher>>>> = Mutex::new(Vec::new());
#[cfg(target_os = "linux")]
static SWEEP: Mutex<()> = Mutex::new(());

#[cfg(target_os = "linux")]
pub(crate) fn register(watcher: &Arc<Mutex<DiskWatcher>>) {
  let mut live = LIVE.lock().expect("live disk watchers lock poisoned");
  prune_live_watchers(&mut live);
  live.push(Arc::downgrade(watcher));
}

#[cfg(target_os = "linux")]
pub(crate) fn unregister(watcher: &Arc<Mutex<DiskWatcher>>) {
  let watcher = Arc::downgrade(watcher);
  let mut live = LIVE.lock().expect("live disk watchers lock poisoned");
  live.retain(|entry| !entry.ptr_eq(&watcher));
  prune_live_watchers(&mut live);
}

#[cfg(target_os = "linux")]
fn prune_live_watchers(live: &mut Vec<Weak<Mutex<DiskWatcher>>>) {
  live.retain(|watcher| watcher.strong_count() > 0);
  if live.is_empty() {
    // A new compiler generation can try nested contexts again. Live peers,
    // including those retained during a sweep, keep sharing the fallback.
    CONTEXTS_DISABLED.store(false, Ordering::Release);
  }
}

/// The caller must release every disk watcher, alias and path index guard first.
#[cfg(target_os = "linux")]
pub(crate) fn reclaim_and_retry_all() {
  let _sweep = SWEEP.lock().expect("context reclaim lock poisoned");
  let watchers: Vec<_> = {
    let mut live = LIVE.lock().expect("live disk watchers lock poisoned");
    let watchers = live.iter().filter_map(Weak::upgrade).collect();
    prune_live_watchers(&mut live);
    watchers
  };
  // Wait for in-flight registrations and finish every release before any retry.
  for watcher in &watchers {
    let mut disk = watcher.lock().expect("disk watcher lock poisoned");
    if disk.inner.is_some() {
      disk.release_context_only();
    }
  }
  for watcher in &watchers {
    let mut disk = watcher.lock().expect("disk watcher lock poisoned");
    if disk.inner.is_some() {
      disk.retry_missing_analyzer_patterns();
    }
  }
}

#[derive(Default)]
struct PathAliases {
  native_paths: InternedPathMap<InternedPath>,
  patterns: InternedPathMap<HashSet<WatchPattern>>,
}

/// `DiskWatcher` is responsible for managing the underlying file system watcher
/// and keeping track of the currently watched paths.
pub struct DiskWatcher {
  /// The actual file system watcher from the `notify` crate.
  inner: Option<RecommendedWatcher>,
  aliases: Arc<Mutex<PathAliases>>,
  has_aliases: Arc<AtomicBool>,
  /// A set of pattern that are currently being watched.
  watch_patterns: HashSet<WatchPattern>,
  /// Requirements from registered files, contexts and missing paths.
  analyzer_patterns: HashSet<WatchPattern>,
  /// Descendant directories found by the filtered context scan.
  context_paths: HashSet<InternedPath>,
  #[cfg(not(target_os = "linux"))]
  context_watch_limit_reached: bool,
  #[cfg(target_os = "linux")]
  reclaim_requested: bool,
  context_watch_errors: (usize, Option<notify::Error>),
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

    let aliases = Arc::new(Mutex::new(PathAliases::default()));
    let event_aliases = Arc::clone(&aliases);
    let has_aliases = Arc::new(AtomicBool::new(false));
    let event_has_aliases = Arc::clone(&has_aliases);
    let inner = RecommendedWatcher::new(
      move |result: notify::Result<Event>| match result {
        Ok(event) => {
          let event = if event_has_aliases.load(std::sync::atomic::Ordering::Relaxed) {
            let mut event = event;
            let aliases = event_aliases.lock().expect("watch aliases lock poisoned");
            event.paths = std::mem::take(&mut event.paths)
              .into_iter()
              .flat_map(|path| {
                let mut paths = HashSet::default();
                for ancestor in path.ancestors() {
                  let key = InternedPath::from(ancestor);
                  let native = aliases.native_paths.get(&key).unwrap_or(&key);
                  // An event on a link itself is not an event on its target.
                  if (ancestor != path || *native == key)
                    && let Some(patterns) = aliases.patterns.get(native)
                  {
                    for pattern in patterns {
                      if ancestor == path
                        || Some(ancestor) == path.parent()
                        || pattern.mode == Recursive
                      {
                        paths.insert(if ancestor == path {
                          pattern.path.to_path_buf()
                        } else {
                          pattern
                            .path
                            .join(path.strip_prefix(ancestor).expect("watched ancestor"))
                        });
                      }
                    }
                  }
                }
                if paths.is_empty() {
                  paths.insert(path);
                }
                paths
              })
              .collect();
            event
          } else {
            event
          };
          handle_event(event, &trigger);
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
      aliases,
      has_aliases,
      watch_patterns: HashSet::default(),
      analyzer_patterns: HashSet::default(),
      context_paths: HashSet::default(),
      #[cfg(not(target_os = "linux"))]
      context_watch_limit_reached: false,
      #[cfg(target_os = "linux")]
      reclaim_requested: false,
      context_watch_errors: (0, None),
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
    self.analyzer_patterns = patterns.collect();
    let new_patterns: HashSet<_> = self
      .analyzer_patterns
      .iter()
      .cloned()
      .chain(self.context_paths.iter().map(|path| WatchPattern {
        path: path.clone(),
        mode: NonRecursive,
      }))
      .collect();
    // Collect stale paths that are no longer needed, then unwatch and remove them.
    let stale_paths: HashSet<InternedPath> = self
      .watch_patterns
      .iter()
      .filter(|p| !new_patterns.contains(p))
      .map(|p| p.path.clone())
      .collect();

    for path in &stale_paths {
      // A failed unwatch must not stop the new patterns from being watched. For example, on
      // Linux `notify` can return `EINVAL` for a subdirectory that was deleted while its parent
      // was watched recursively (https://github.com/notify-rs/notify/issues/709).
      if let Err(e) = self.unwatch_path(path)
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

    // A recursive root must be installed before its covered directories, or
    // those directories keep redundant native watches for the whole session.
    #[cfg(any(target_os = "macos", target_os = "windows"))]
    let new_patterns = {
      let mut patterns: Vec<_> = new_patterns.into_iter().collect();
      patterns.sort_by_key(|pattern| pattern.mode == NonRecursive);
      patterns
    };
    for pattern in new_patterns {
      if !self.analyzer_patterns.contains(&pattern) && !self.context_paths.contains(&pattern.path) {
        continue;
      }
      if self.watch_patterns.contains(&pattern) {
        continue;
      }
      let result = self.watch_pattern(&pattern).or_else(|error| {
        if !self.analyzer_patterns.contains(&pattern)
          || !matches!(error.kind, notify::ErrorKind::MaxFilesWatch)
        {
          return Err(error);
        }
        #[cfg(target_os = "linux")]
        {
          let first = !CONTEXTS_DISABLED.swap(true, Ordering::AcqRel);
          self.reclaim_requested = true;
          let released = self.release_context_only();
          if first {
            eprintln!("Error in file watcher: nested context folders are no longer watched because the OS watch limit was reached; released {released} context watches");
          }
        }
        #[cfg(not(target_os = "linux"))]
        {
          let paths: Vec<_> = self.context_paths.iter().filter(|path| {
            !self.analyzer_patterns.iter().any(|p| p.path == **path)
          }).cloned().collect();
          for path in &paths {
            self.remove_context_directory(path, false);
          }
          if !self.context_watch_limit_reached {
            self.context_watch_limit_reached = true;
            eprintln!("Error in file watcher: nested context folders are no longer watched because the OS watch limit was reached; released {} context watches", paths.len());
          }
        }
        self.watch_pattern(&pattern)
      });
      if let Err(error) = result {
        if self.analyzer_patterns.contains(&pattern)
          && !matches!(error.kind, notify::ErrorKind::MaxFilesWatch)
        {
          self.report_context_watch_errors();
          return Err(rspack_error::error!(error.to_string()));
        }
        self.context_watch_failed(&pattern, error);
        continue;
      }
      self.watch_patterns.insert(pattern);
    }

    self.report_context_watch_errors();
    Ok(())
  }

  #[cfg(target_os = "linux")]
  pub(crate) fn take_reclaim_request(&mut self) -> bool {
    std::mem::take(&mut self.reclaim_requested)
  }

  #[cfg(target_os = "linux")]
  fn release_context_only(&mut self) -> usize {
    let paths: Vec<_> = self
      .context_paths
      .iter()
      .filter(|path| !self.analyzer_patterns.iter().any(|p| p.path == **path))
      .cloned()
      .collect();
    for path in &paths {
      self.remove_context_directory(path, false);
    }
    paths.len()
  }

  #[cfg(target_os = "linux")]
  fn retry_missing_analyzer_patterns(&mut self) {
    let patterns: Vec<_> = self
      .analyzer_patterns
      .difference(&self.watch_patterns)
      .cloned()
      .collect();
    for pattern in patterns {
      if let Err(error) = self.watch_pattern(&pattern) {
        self.context_watch_failed(&pattern, error);
      } else {
        self.watch_patterns.insert(pattern);
      }
    }
    self.report_context_watch_errors();
  }

  fn context_watch_failed(&mut self, pattern: &WatchPattern, error: notify::Error) {
    if !self.analyzer_patterns.contains(pattern) {
      self.context_paths.remove(&pattern.path);
      self.watch_patterns.retain(|p| p.path != pattern.path);
      let _ = self.unwatch_path(&pattern.path);
    }
    self.context_watch_errors.0 += 1;
    self.context_watch_errors.1.get_or_insert(error);
  }

  fn report_context_watch_errors(&mut self) {
    let (watch_errors, first_error) = std::mem::take(&mut self.context_watch_errors);
    if let Some(error) = first_error {
      tracing::error!(target: "rspack_watcher::disk_watcher", watch_errors, "{error}");
      let failed = if matches!(error.kind, notify::ErrorKind::MaxFilesWatch) {
        "watch patterns"
      } else {
        "context directories"
      };
      eprintln!(
        "Error in file watcher: failed to watch {watch_errors} {failed}; first error: {error}"
      );
    }
  }

  /// Register before reading a directory, so a file is found either by the scan
  /// or by its live event. The caller has already applied the ignored filter.
  pub(crate) fn watch_context_directory(
    &mut self,
    path: &InternedPath,
  ) -> rspack_error::Result<bool> {
    #[cfg(target_os = "linux")]
    let context_watch_limit_reached = CONTEXTS_DISABLED.load(Ordering::Acquire);
    #[cfg(not(target_os = "linux"))]
    let context_watch_limit_reached = self.context_watch_limit_reached;
    if self.inner.is_none()
      || (context_watch_limit_reached && !self.analyzer_patterns.iter().any(|p| p.path == *path))
    {
      return Ok(false);
    }
    let pattern = WatchPattern {
      path: path.clone(),
      mode: NonRecursive,
    };
    if !self.watch_patterns.contains(&pattern) {
      self.watch_pattern(&pattern).map_err(|error| {
        rspack_error::error!("failed to watch context directory {path:?}: {error}")
      })?;
      self.watch_patterns.insert(pattern);
    }
    Ok(self.context_paths.insert(path.clone()))
  }

  /// Release only this scan's requirement. Another registration can still need
  /// the same directory. Removed directories also invalidate the native watch.
  pub(crate) fn remove_context_directory(&mut self, path: &InternedPath, removed: bool) {
    if !self.context_paths.remove(path) {
      return;
    }
    let mut pattern = WatchPattern {
      path: path.clone(),
      mode: NonRecursive,
    };
    if !removed
      && (self.analyzer_patterns.contains(&pattern)
        || self.analyzer_patterns.contains(&WatchPattern {
          path: path.clone(),
          mode: Recursive,
        }))
    {
      return;
    }
    if removed && !self.watch_patterns.contains(&pattern) {
      pattern.mode = Recursive;
    }
    if self.watch_patterns.remove(&pattern)
      && let Err(error) = self.unwatch_path(path)
      && !matches!(error.kind, notify::ErrorKind::WatchNotFound)
    {
      tracing::warn!(target: "rspack_watcher::disk_watcher", ?path, "failed to unwatch context directory: {error:?}");
    }
  }

  // Keep each native watch under one physical spelling. Logical aliases
  // hold references here, so removing one cannot release another's directory.
  fn watch_pattern(&mut self, pattern: &WatchPattern) -> notify::Result<()> {
    let (path, mode) = {
      let path =
        InternedPath::from(std::fs::canonicalize(&pattern.path).map_err(notify::Error::io)?);
      if pattern.path != path {
        self
          .has_aliases
          .store(true, std::sync::atomic::Ordering::Relaxed);
      }
      let mut aliases = self.aliases.lock().expect("watch aliases lock poisoned");
      if aliases
        .native_paths
        .get(&pattern.path)
        .is_some_and(|old| *old != path)
      {
        drop(aliases);
        self.unwatch_path(&pattern.path)?;
        aliases = self.aliases.lock().expect("watch aliases lock poisoned");
      }
      aliases
        .patterns
        .entry(path.clone())
        .or_default()
        .insert(pattern.clone());
      aliases
        .native_paths
        .insert(pattern.path.clone(), path.clone());
      // Retain covered spellings for event fan-out and later restoration, but
      // do not add another OS watch below a canonical recursive requirement.
      let covered = path
        .ancestors()
        .skip(usize::from(pattern.mode == Recursive))
        .any(|ancestor| {
          aliases
            .patterns
            .get(&InternedPath::from(ancestor))
            .is_some_and(|patterns| patterns.iter().any(|p| p.mode == Recursive))
        });
      if covered {
        return Ok(());
      }
      (path, pattern.mode)
    };
    if let Some(watcher) = &mut self.inner
      && let Err(error) = watcher.watch(&path, mode)
    {
      if !self.watch_patterns.contains(pattern) {
        if matches!(error.kind, notify::ErrorKind::MaxFilesWatch) {
          let _ = self.unwatch_patterns(&pattern.path, Some(pattern.mode));
        } else {
          let _ = self.unwatch_path(&pattern.path);
        }
      }
      return Err(error);
    }
    Ok(())
  }

  fn unwatch_path(&mut self, path: &InternedPath) -> notify::Result<()> {
    self.unwatch_patterns(path, None)
  }

  fn unwatch_patterns(
    &mut self,
    path: &InternedPath,
    mode: Option<notify::RecursiveMode>,
  ) -> notify::Result<()> {
    let (path, restore) = {
      let mut aliases = self.aliases.lock().expect("watch aliases lock poisoned");
      let Some(native) = aliases.native_paths.get(path).cloned() else {
        return Ok(());
      };
      let patterns = aliases
        .patterns
        .get_mut(&native)
        .expect("native watch aliases");
      let recursive = patterns.iter().any(|p| p.mode == Recursive);
      patterns.retain(|p| p.path != *path || mode.is_some_and(|mode| p.mode != mode));
      let remaining_path = patterns.iter().any(|p| p.path == *path);
      let remaining_recursive = patterns.iter().any(|p| p.mode == Recursive);
      let remaining = !patterns.is_empty();
      if !remaining {
        aliases.patterns.remove(&native);
      }
      if !remaining_path {
        aliases.native_paths.remove(path);
      }
      let covered = native.ancestors().skip(1).any(|ancestor| {
        aliases
          .patterns
          .get(&InternedPath::from(ancestor))
          .is_some_and(|patterns| patterns.iter().any(|p| p.mode == Recursive))
      });
      if covered || (remaining && recursive == remaining_recursive) {
        return Ok(());
      }
      let restore: Vec<_> = if recursive {
        aliases
          .patterns
          .iter()
          .filter(|(p, _)| p.starts_with(&native))
          .flat_map(|(_, patterns)| patterns.iter().cloned())
          .collect()
      } else {
        Vec::new()
      };
      (native, restore)
    };
    let result = self
      .inner
      .as_mut()
      .map_or(Ok(()), |watcher| watcher.unwatch(&path));
    {
      let mut restore_error = None;
      for pattern in restore {
        self.watch_patterns.remove(&pattern);
        if let Err(error) = self.watch_pattern(&pattern) {
          if self.analyzer_patterns.contains(&pattern) {
            restore_error.get_or_insert(error);
          } else {
            self.context_watch_failed(&pattern, error);
          }
        } else {
          self.watch_patterns.insert(pattern);
        }
      }
      if let Some(error) = restore_error {
        return Err(error);
      }
    }
    result
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
            mode: NonRecursive,
          },
          WatchPattern {
            path: InternedPath::from(dir_b.clone()),
            mode: NonRecursive,
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
            mode: NonRecursive,
          },
          WatchPattern {
            path: InternedPath::from(dir_c.clone()),
            mode: NonRecursive,
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
          mode: Recursive,
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
          mode: NonRecursive,
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
