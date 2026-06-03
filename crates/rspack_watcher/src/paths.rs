use std::{collections::HashMap, fmt::Debug, ops::Deref, path::PathBuf, time::SystemTime};

use dashmap::setref::multiple::RefMulti;
use rspack_error::Result;
use rspack_paths::{ArcPath, ArcPathDashMap, ArcPathDashSet};

use super::FsWatcherIgnored;
use crate::time_info::{
  TimeInfoEntry, TimeInfoTables, ensure_fs_accuracy, fixup_entry_accuracy, fs_accuracy,
  initial_entry, now_millis, system_time_to_millis,
};

/// An iterator that chains together references to all files, directories, and missing paths
/// stored in the [`PathTracker`]. This allows iteration over all registered paths as a single sequence.
pub(crate) struct All<'a> {
  inner: Box<dyn Iterator<Item = RefMulti<'a, ArcPath>> + 'a>,
}

impl<'a> All<'a> {
  /// Creates a new `All` iterator from the given sets of files, directories, and missing paths.
  fn new(
    files: &'a ArcPathDashSet,
    directories: &'a ArcPathDashSet,
    missing: &'a ArcPathDashSet,
  ) -> Self {
    let files_iter = files.iter();
    let directories_iter = directories.iter();
    let missing_iter = missing.iter();
    let chain = files_iter.chain(directories_iter).chain(missing_iter);

    Self {
      inner: Box::new(chain),
    }
  }
}

impl<'a> Iterator for All<'a> {
  type Item = ArcPath;

  fn next(&mut self) -> Option<Self::Item> {
    self.inner.next().map(|v| v.deref().clone())
  }
}

/// `PathAccessor` provides access to the sets of files, directories, and missing paths.
pub(crate) struct PathAccessor<'a> {
  files: &'a PathTracker,
  directories: &'a PathTracker,
  missing: &'a PathTracker,
}

impl<'a> PathAccessor<'a> {
  /// Creates a new `PathAccessor` with references to the sets of files, directories, and missing paths.
  fn new(path_manager: &'a PathManager) -> Self {
    Self {
      files: &path_manager.files,
      directories: &path_manager.directories,
      missing: &path_manager.missing,
    }
  }

  /// Returns references to the sets of files, including added and removed files.
  pub fn files(&self) -> (&'a ArcPathDashSet, &'a ArcPathDashSet, &'a ArcPathDashSet) {
    (&self.files.all, &self.files.added, &self.files.removed)
  }

  /// Returns references to the set of directories, including added and removed directories.
  pub fn directories(&self) -> (&'a ArcPathDashSet, &'a ArcPathDashSet, &'a ArcPathDashSet) {
    (
      &self.directories.all,
      &self.directories.added,
      &self.directories.removed,
    )
  }

  /// Returns references to the set of missing paths, including added and removed missing paths.
  pub fn missing(&self) -> (&'a ArcPathDashSet, &'a ArcPathDashSet, &'a ArcPathDashSet) {
    (
      &self.missing.all,
      &self.missing.added,
      &self.missing.removed,
    )
  }

  /// Returns an iterator that combines all files, directories, and missing paths into a single sequence.
  pub fn all(&self) -> impl Iterator<Item = ArcPath> + '_ {
    All::new(&self.files.all, &self.directories.all, &self.missing.all)
  }
}

/// `PathUpdater` is used to update collections of registered paths (files, directories, and missing paths)
/// by specifying which paths have been added and which have been removed. It holds vectors of paths to be
/// added and removed, and provides functionality to apply these changes to a path tracker. This struct
/// facilitates batch updates to the path sets, ensuring that additions and removals are processed efficiently.
#[derive(Debug)]
struct PathUpdater {
  pub added: Vec<ArcPath>,
  pub removed: Vec<ArcPath>,
  base_dir: PathBuf,
}

impl<Added, Removed> From<(Added, Removed)> for PathUpdater
where
  Added: Iterator<Item = ArcPath>,
  Removed: Iterator<Item = ArcPath>,
{
  fn from((added, removed): (Added, Removed)) -> Self {
    Self {
      added: added.collect(),
      removed: removed.collect(),
      base_dir: std::env::current_dir().unwrap_or_default(),
    }
  }
}

impl PathUpdater {
  /// Update the paths in the given set.
  fn update(self, watch_tracker: &PathTracker, ignored: &FsWatcherIgnored) -> Result<()> {
    let added_paths = self.added;
    let removed_paths = self.removed;

    for added in added_paths {
      if ignored.should_be_ignored(added.to_str().expect("Path should be valid UTF-8")) {
        continue; // Skip ignored paths
      }

      if added.is_absolute() {
        watch_tracker.add(added);
        continue;
      }

      let added_absolute_path = self.base_dir.join(added.as_ref());

      watch_tracker.add(ArcPath::from(added_absolute_path));
    }

    for removed in removed_paths {
      if removed.is_absolute() {
        watch_tracker.remove(removed);
        continue;
      }

      let removed_absolute_path = self.base_dir.join(removed.as_ref());

      watch_tracker.remove(ArcPath::from(removed_absolute_path));
    }
    Ok(())
  }
}

#[derive(Debug, Default)]
/// `PathTracker` is responsible for tracking the state of file system paths for the watcher.
///
/// It maintains three sets:
/// - `added`: Paths that have been recently added and are being watched.
/// - `removed`: Paths that have been removed from watching.
/// - `all`: All currently watched paths.
///
/// This struct enables efficient updates and queries for the file system watcher,
/// ensuring that changes to the set of watched paths are tracked and managed correctly.
struct PathTracker {
  added: ArcPathDashSet,
  removed: ArcPathDashSet,
  all: ArcPathDashSet,
}

impl PathTracker {
  fn reset(&self) {
    self.added.clear();
    self.removed.clear();
  }

  /// Adds a path to the tracker.
  fn add(&self, path: ArcPath) {
    self.added.insert(path.clone());
    self.all.insert(path);
  }

  /// Removes a path from the tracker.
  fn remove(&self, path: ArcPath) {
    self.all.remove(&path);
    self.removed.insert(path);
  }
}

/// `PathManager` is responsible for managing the set of files, directories, and missing paths.
#[derive(Default)]
pub(crate) struct PathManager {
  files: PathTracker,
  directories: PathTracker,
  missing: PathTracker,
  pub ignored: FsWatcherIgnored,
  /// Baseline mtime for registered files, captured at scan time.
  /// Used to filter stale FSEvents that arrive for files not actually modified.
  /// See: https://gist.github.com/stormslowly/ed758500de6f23211fd63b39eba5ed07
  file_mtimes: ArcPathDashMap<SystemTime>,
  /// watchpack-style `{ safe_time, timestamp, accuracy }` time info per file,
  /// surfaced to webpack as `fileTimeInfoEntries`. See [`crate::time_info`].
  file_entries: ArcPathDashMap<TimeInfoEntry>,
  /// Directory-level time info, surfaced as `contextTimeInfoEntries`. Holds the
  /// directory's own change time; the value exposed to webpack additionally
  /// aggregates the `safe_time` of every registered child file (see
  /// [`PathManager::collect_time_info`]).
  dir_entries: ArcPathDashMap<TimeInfoEntry>,
}

impl PathManager {
  /// Create a new `PathManager` with an optional ignored paths filter.
  pub fn new(ignored: FsWatcherIgnored) -> Self {
    Self {
      files: PathTracker::default(),
      directories: PathTracker::default(),
      missing: PathTracker::default(),
      ignored,
      file_mtimes: ArcPathDashMap::default(),
      file_entries: ArcPathDashMap::default(),
      dir_entries: ArcPathDashMap::default(),
    }
  }

  /// Reset the per-`watch()`-call diff state (added / removed sets) without
  /// touching the long-lived mtime baselines.
  ///
  /// `file_mtimes` is intentionally NOT cleared here: the mtime baselines
  /// are tied to the lifetime of the registered files, not to the lifetime
  /// of a single `FsWatcher::watch()` invocation. Each call to `watch()`
  /// from rspack's rebuild cycle (aggregate → pause → rebuild → rewatch)
  /// must NOT re-snapshot the baseline, otherwise the snapshot can capture
  /// a post-user-write mtime and then `has_mtime_changed` silently
  /// suppresses the very FSEvent that the user is waiting for. Stale
  /// entries for files that have actually been unregistered are pruned by
  /// `update()` via `remove_file_mtime`.
  pub fn reset(&self) {
    self.files.reset();
    self.directories.reset();
    self.missing.reset();
  }

  /// Record an initial baseline mtime for `path`, but only if no baseline
  /// already exists. This is the "incremental" form used by
  /// `FsWatcher::wait_for_event`: on the first `watch()` call we record
  /// mtimes for all newly-registered files; on subsequent `watch()` calls
  /// we MUST skip files that already have a baseline, otherwise we risk
  /// snapshotting a mtime that the user has just bumped via `writeFile`.
  ///
  /// Use `has_mtime_changed` (which atomically reads-and-updates the
  /// baseline) to advance the recorded mtime in response to real events.
  pub fn set_file_mtime_if_absent(&self, path: ArcPath, mtime: SystemTime) {
    self.file_mtimes.entry(path).or_insert(mtime);
  }

  /// Drop the baseline for a path that is no longer being watched, so the
  /// map does not grow unboundedly across watch cycles.
  pub fn remove_file_mtime(&self, path: &ArcPath) {
    self.file_mtimes.remove(path);
  }

  /// Record an initial-scan time entry for a file, mirroring watchpack
  /// `setFileTime(initial=true)`. Only set if absent: a live event may have
  /// already advanced the entry across watch cycles and must not be clobbered
  /// by a re-stat (same reasoning as [`Self::set_file_mtime_if_absent`]).
  pub fn set_initial_file_time(&self, path: ArcPath, mtime: SystemTime) {
    self
      .file_entries
      .entry(path)
      .or_insert_with(|| initial_entry(system_time_to_millis(mtime)));
  }

  /// Record an initial-scan time entry for a directory (using its mtime as the
  /// birthtime), mirroring watchpack `setDirectory(initial=true)`.
  pub fn set_initial_dir_time(&self, path: ArcPath, mtime: SystemTime) {
    self
      .dir_entries
      .entry(path)
      .or_insert_with(|| initial_entry(system_time_to_millis(mtime)));
  }

  /// Record a real-time change for a file, mirroring watchpack
  /// `setFileTime(initial=false)`: `safe_time = now`, `accuracy = 0`. Skips the
  /// update for an attribute-only change — the same mtime already sitting
  /// outside the accuracy window, which cannot hide a content change.
  pub fn set_event_file_time(&self, path: &ArcPath, mtime: SystemTime) {
    let mtime_ms = system_time_to_millis(mtime);
    ensure_fs_accuracy(mtime_ms);
    let now = now_millis();
    // Read-and-drop the guard before inserting: DashMap would deadlock if a
    // read ref into the same shard were held across a write.
    let skip = {
      if let Some(old) = self.file_entries.get(path) {
        old.timestamp == mtime_ms && mtime_ms + fs_accuracy() < now
      } else {
        false
      }
    };
    if skip {
      return;
    }
    self.file_entries.insert(
      path.clone(),
      TimeInfoEntry {
        safe_time: now,
        timestamp: mtime_ms,
        accuracy: 0,
      },
    );
  }

  /// Bump a directory's change time to now when an event is associated with it,
  /// but only for registered directories. Captures directory-level changes
  /// (a child added/removed) that a remaining child file's entry can't reflect.
  pub fn touch_dir_if_registered(&self, path: &ArcPath) {
    if !self.directories.all.contains(path) {
      return;
    }
    let now = now_millis();
    self.dir_entries.insert(
      path.clone(),
      TimeInfoEntry {
        safe_time: now,
        timestamp: now,
        accuracy: 0,
      },
    );
  }

  /// Collect the full webpack-shaped time tables, mirroring watchpack's
  /// `collectTimeInfoEntries`. Returns `(file_entries, context_entries)` where
  /// each entry's accuracy is tightened on read ([`fixup_entry_accuracy`]) and a
  /// directory's `safe_time` is the max of its own change time and the
  /// `safe_time` of every registered descendant file.
  pub fn collect_time_info(&self) -> TimeInfoTables {
    let accuracy = fs_accuracy();

    // Seed directory aggregates with each directory's own change time.
    let mut dir_times: HashMap<ArcPath, u64> = HashMap::new();
    for dir in self.dir_entries.iter() {
      let mut entry = *dir.value();
      fixup_entry_accuracy(&mut entry, accuracy);
      dir_times.insert(dir.key().clone(), entry.safe_time);
    }

    let mut files = Vec::with_capacity(self.file_entries.len());
    for file in self.file_entries.iter() {
      let mut entry = *file.value();
      fixup_entry_accuracy(&mut entry, accuracy);
      let path = file.key();
      files.push((path.to_string_lossy().to_string(), entry));

      // Bubble this file's safe_time up to every registered ancestor directory.
      let mut current = path.parent();
      while let Some(parent) = current {
        let parent_arc = ArcPath::from(parent);
        if self.directories.all.contains(&parent_arc) {
          let slot = dir_times.entry(parent_arc).or_insert(0);
          *slot = (*slot).max(entry.safe_time);
        }
        current = parent.parent();
      }
    }

    let contexts = dir_times
      .into_iter()
      .map(|(dir, safe_time)| (dir.to_string_lossy().to_string(), safe_time))
      .collect();

    (files, contexts)
  }

  /// Check if a file's mtime has changed from the stored baseline.
  /// Returns `true` if the event should pass through (mtime changed or no baseline).
  /// Returns `false` if the event should be suppressed (mtime unchanged = stale).
  pub fn has_mtime_changed(&self, path: &ArcPath) -> bool {
    if !self.files.all.contains(path) {
      return true;
    }

    let current_mtime = match path
      .metadata()
      .and_then(|m| m.modified().or_else(|_| m.created()))
    {
      Ok(mtime) => mtime,
      Err(_) => return true,
    };

    match self.file_mtimes.get(path) {
      Some(baseline) => {
        if current_mtime != *baseline {
          drop(baseline);
          self.file_mtimes.insert(path.clone(), current_mtime);
          self.set_event_file_time(path, current_mtime);
          true
        } else {
          false
        }
      }
      None => {
        self.file_mtimes.insert(path.clone(), current_mtime);
        self.set_event_file_time(path, current_mtime);
        true
      }
    }
  }

  /// Update the paths, directories, and missing paths in the `PathManager`.
  pub fn update(
    &self,
    files: (impl Iterator<Item = ArcPath>, impl Iterator<Item = ArcPath>),
    directories: (impl Iterator<Item = ArcPath>, impl Iterator<Item = ArcPath>),
    missing: (impl Iterator<Item = ArcPath>, impl Iterator<Item = ArcPath>),
  ) -> Result<()> {
    PathUpdater::from(files).update(&self.files, &self.ignored)?;
    PathUpdater::from(directories).update(&self.directories, &self.ignored)?;
    PathUpdater::from(missing).update(&self.missing, &self.ignored)?;

    // Prune mtime baselines for files no longer being watched so the map
    // does not grow unboundedly across `watch()` cycles. `reset()` has
    // already cleared `self.files.removed` at the start of this `watch()`
    // call, so what we see here is only this cycle's removals.
    let removed_files: Vec<ArcPath> = self.files.removed.iter().map(|p| p.clone()).collect();
    for path in &removed_files {
      self.remove_file_mtime(path);
      self.file_entries.remove(path);
    }
    let removed_dirs: Vec<ArcPath> = self.directories.removed.iter().map(|p| p.clone()).collect();
    for path in &removed_dirs {
      self.dir_entries.remove(path);
    }

    Ok(())
  }

  /// Create a new `PathAccessor` to access the current state of paths, directories, and missing paths.
  pub fn access(&self) -> PathAccessor<'_> {
    PathAccessor::new(self)
  }
}

#[cfg(test)]
mod tests {
  use rspack_paths::Utf8Path;

  use super::*;

  #[test]
  fn test_updater() {
    let updater = PathUpdater::from((
      vec![
        ArcPath::from(Utf8Path::new("src/index.js")),
        ArcPath::from(Utf8Path::new("node_modules/.pnpm/axios/lib/index.js")),
        ArcPath::from(Utf8Path::new(".git/abc/")),
      ]
      .into_iter(),
      vec![].into_iter(),
    ));
    let ignored = FsWatcherIgnored::Paths(vec![
      "**/.git/**".to_owned(),
      "**/node_modules/**".to_owned(),
    ]);

    let path_tracker = PathTracker::default();

    updater.update(&path_tracker, &ignored).unwrap();

    let all = path_tracker.all;

    assert_eq!(all.len(), 1);
    assert!(
      all
        .iter()
        .any(|p| p.to_string_lossy().contains("src/index.js"))
    )
  }

  #[test]
  fn test_accessor() {
    let path_manager = PathManager::default();

    let files = (
      vec![ArcPath::from(Utf8Path::new("src/index.js"))].into_iter(),
      vec![].into_iter(),
    );
    let dirs = (
      vec![ArcPath::from(Utf8Path::new("src"))].into_iter(),
      vec![].into_iter(),
    );
    let missing = (
      vec![ArcPath::from(Utf8Path::new("src/page/index.ts"))].into_iter(),
      vec![].into_iter(),
    );

    path_manager.update(files, dirs, missing).unwrap();

    let accessor = PathAccessor::new(&path_manager);
    let mut all_paths = vec![];

    for path in accessor.all() {
      all_paths.push(path.to_string_lossy().to_string());
    }

    all_paths.sort();

    assert_eq!(all_paths.len(), 3);

    let should_exist_paths = vec!["src", "src/index.js", "src/page/index.ts"];

    for path in should_exist_paths {
      assert!(all_paths.iter().any(|p| p.ends_with(path)));
    }
  }

  #[test]
  fn test_manager() {
    let ignored = FsWatcherIgnored::Paths(vec![
      "**/node_modules/**".to_string(),
      "**/.git/**".to_string(),
    ]);
    let path_manager = PathManager::new(ignored);
    let files = (
      vec![ArcPath::from(Utf8Path::new("src/index.js"))].into_iter(),
      vec![].into_iter(),
    );
    let directories = (
      vec![
        ArcPath::from(Utf8Path::new("src/")),
        ArcPath::from(Utf8Path::new("node_modules/")),
      ]
      .into_iter(),
      vec![].into_iter(),
    );
    let missing = (
      vec![ArcPath::from(Utf8Path::new("src/page/index.ts"))].into_iter(),
      vec![].into_iter(),
    );

    path_manager.update(files, directories, missing).unwrap();

    let accessor = path_manager.access();
    let mut all_paths = accessor
      .all()
      .map(|p| p.to_string_lossy().to_string())
      .collect::<Vec<_>>();

    all_paths.sort();

    assert_eq!(all_paths.len(), 3);

    let should_exist_paths = vec!["src/", "src/index.js", "src/page/index.ts"];

    for path in should_exist_paths {
      assert!(all_paths.iter().any(|p| p.ends_with(path)));
    }
  }

  /// Regression for the FSEvents stale-event race: simulate two consecutive
  /// `FsWatcher::watch()` cycles with a real file write landing between
  /// them (the slow-runner case where the FSEvent is in the kernel queue
  /// but not yet delivered when the second cycle starts). The baseline
  /// for the already-registered file must survive the second `reset()`,
  /// so the delayed change event isn't suppressed as stale.
  #[test]
  fn test_baseline_persists_across_consecutive_watch_cycles() {
    use std::{thread::sleep, time::Duration};

    use tempfile::NamedTempFile;

    let tempfile = NamedTempFile::new().expect("create temp file");
    let path = ArcPath::from(tempfile.path());

    let pm = PathManager::default();
    pm.update(
      (std::iter::once(path.clone()), std::iter::empty()),
      (std::iter::empty(), std::iter::empty()),
      (std::iter::empty(), std::iter::empty()),
    )
    .expect("register file");

    // T0 — first `watch()` cycle records the baseline.
    let initial_mtime = tempfile
      .path()
      .metadata()
      .and_then(|m| m.modified())
      .expect("read initial mtime");
    pm.set_file_mtime_if_absent(path.clone(), initial_mtime);

    // T3 — rspack starts a rebuild; `reset()` runs ahead of the next `watch()`.
    pm.reset();

    // Invariant: `file_mtimes` must survive `reset()`. Without this the
    // baseline gets wiped on every watch cycle and the next bullet point
    // can no longer hold.
    assert_eq!(
      pm.file_mtimes.get(&path).map(|v| *v),
      Some(initial_mtime),
      "file_mtimes must persist across reset()",
    );

    // T4 — a real write lands while no watcher is attached. The sleep
    // covers 1s-resolution filesystems (HFS+, FAT); modern APFS / ext4 /
    // NTFS would not need it but this regression must hold everywhere.
    sleep(Duration::from_millis(1100));
    std::fs::write(tempfile.path(), b"v2").expect("rewrite tempfile");
    let post_write_mtime = tempfile
      .path()
      .metadata()
      .and_then(|m| m.modified())
      .expect("read post-write mtime");
    assert_ne!(
      post_write_mtime, initial_mtime,
      "test sanity: file mtime must advance after the write",
    );

    // T5 — second `watch()` cycle reaches `record_initial_file_mtimes`,
    // which delegates here. For an already-baselined path it must be a
    // no-op so the post-write mtime does NOT overwrite the original.
    pm.set_file_mtime_if_absent(path.clone(), post_write_mtime);
    assert_eq!(
      pm.file_mtimes.get(&path).map(|v| *v),
      Some(initial_mtime),
      "set_file_mtime_if_absent must not overwrite an existing baseline",
    );

    // T6 — the delayed FSEvent finally reaches `Trigger::on_event`, which
    // calls `has_mtime_changed`. Current disk mtime now differs from the
    // preserved baseline, so the event must NOT be suppressed.
    assert!(
      pm.has_mtime_changed(&path),
      "delayed change event must not be suppressed as stale",
    );

    // `has_mtime_changed` also atomically advances the baseline so a
    // subsequent duplicate FSEvent (e.g. re-delivery during rewatch)
    // can still be filtered correctly.
    assert_eq!(
      pm.file_mtimes.get(&path).map(|v| *v),
      Some(post_write_mtime),
      "has_mtime_changed should advance the baseline on a real change",
    );
  }

  /// Read a file's collected `safe_time`, or `None` if it has no entry.
  fn file_safe_time(pm: &PathManager, path: &ArcPath) -> Option<u64> {
    let key = path.to_string_lossy().to_string();
    let (files, _) = pm.collect_time_info();
    files
      .into_iter()
      .find(|(p, _)| *p == key)
      .map(|(_, e)| e.safe_time)
  }

  /// Read a directory's collected `safe_time`, or `None` if it has no entry.
  fn dir_safe_time(pm: &PathManager, path: &ArcPath) -> Option<u64> {
    let key = path.to_string_lossy().to_string();
    let (_, dirs) = pm.collect_time_info();
    dirs.into_iter().find(|(p, _)| *p == key).map(|(_, t)| t)
  }

  /// Ports watchpack "setFileTime without initial triggers change": a real-time
  /// event with a different mtime must advance `safe_time` to ~now.
  #[test]
  fn set_event_advances_safe_time_on_real_change() {
    use std::{
      path::Path,
      thread::sleep,
      time::{Duration, SystemTime},
    };

    let pm = PathManager::default();
    let path = ArcPath::from(Path::new("/x/file.js"));

    let t1 = SystemTime::now() - Duration::from_secs(20);
    let t2 = SystemTime::now() - Duration::from_secs(10);
    pm.set_event_file_time(&path, t1);
    let s1 = file_safe_time(&pm, &path).expect("entry after first event");
    // 50ms comfortably exceeds the ~15.6ms SystemTime granularity on Windows
    // so `now_millis()` reliably advances between the two events.
    sleep(Duration::from_millis(50));
    pm.set_event_file_time(&path, t2);
    let s2 = file_safe_time(&pm, &path).expect("entry after second event");

    assert!(
      s2 > s1,
      "a real change must advance safe_time ({s1} -> {s2})"
    );
  }

  /// Ports watchpack "setFileTime skips when timestamp is unchanged and old is
  /// stable": a repeated event carrying the same, already-stable mtime is an
  /// attribute-only change and must not advance `safe_time`.
  #[test]
  fn set_event_skips_attribute_only_change() {
    use std::{
      path::Path,
      thread::sleep,
      time::{Duration, SystemTime},
    };

    let pm = PathManager::default();
    let path = ArcPath::from(Path::new("/x/stable.js"));

    // An mtime well outside any plausible accuracy window (<= 2000ms).
    let stable = SystemTime::now() - Duration::from_secs(10);
    pm.set_event_file_time(&path, stable);
    let s1 = file_safe_time(&pm, &path).expect("entry after first event");
    sleep(Duration::from_millis(5));
    pm.set_event_file_time(&path, stable);
    let s2 = file_safe_time(&pm, &path).expect("entry after second event");

    assert_eq!(
      s1, s2,
      "attribute-only change (same stable mtime) must not advance safe_time",
    );
  }

  /// `set_initial_file_time` must not clobber an entry a live event already
  /// advanced — mirrors the if-absent contract of the mtime baseline.
  #[test]
  fn set_initial_does_not_overwrite_event_entry() {
    use std::{
      path::Path,
      time::{Duration, SystemTime},
    };

    let pm = PathManager::default();
    let path = ArcPath::from(Path::new("/x/live.js"));

    pm.set_event_file_time(&path, SystemTime::now() - Duration::from_secs(1));
    let after_event = file_safe_time(&pm, &path).expect("entry after event");

    // A later initial re-scan with an ancient mtime must be a no-op.
    pm.set_initial_file_time(path.clone(), SystemTime::now() - Duration::from_secs(100));
    let after_initial = file_safe_time(&pm, &path).expect("entry still present");

    assert_eq!(
      after_event, after_initial,
      "initial scan must not overwrite a live event entry",
    );
  }

  /// Ports watchpack's directory aggregation in `collectTimeInfoEntries`: a
  /// registered directory's `safe_time` is the max of every registered
  /// descendant file's `safe_time`.
  #[test]
  fn collect_time_info_aggregates_child_safe_times_to_dir() {
    use std::time::SystemTime;

    let pm = PathManager::default();
    // Build from an absolute temp-dir base: a Unix-style "/project" path is
    // NOT absolute on Windows, so `update()` would relativize the directory
    // (join cwd) and the file's parent chain would no longer match it.
    let base = std::env::temp_dir();
    let dir = ArcPath::from(base.join("rspack_wt_agg_src").as_path());
    let file = ArcPath::from(
      base
        .join("rspack_wt_agg_src")
        .join("nested")
        .join("a.js")
        .as_path(),
    );

    // Register the directory so it is an aggregation target.
    pm.update(
      (std::iter::empty(), std::iter::empty()),
      (std::iter::once(dir.clone()), std::iter::empty()),
      (std::iter::empty(), std::iter::empty()),
    )
    .expect("register dir");

    pm.set_event_file_time(&file, SystemTime::now());
    let file_st = file_safe_time(&pm, &file).expect("file entry");
    let dir_st = dir_safe_time(&pm, &dir).expect("dir entry aggregated from child");

    assert!(
      dir_st >= file_st,
      "dir safe_time ({dir_st}) must be >= descendant file safe_time ({file_st})",
    );
  }
}
