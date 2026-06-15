use std::{
  fmt::Debug,
  ops::Deref,
  path::{Path, PathBuf},
  time::SystemTime,
};

use dashmap::setref::multiple::RefMulti;
use rspack_error::Result;
use rspack_paths::{ArcPath, ArcPathDashMap, ArcPathDashSet};

use super::{FsWatcherIgnored, TimeInfoEntry, ignored::IgnoredMatcher};
use crate::time_info;

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
  fn update(self, watch_tracker: &PathTracker, ignored: &IgnoredMatcher) -> Result<()> {
    let added_paths = self.added;
    let removed_paths = self.removed;

    for added in added_paths {
      // Absolutize before the ignored check so registration filters on the
      // same absolute path the watcher reports for events.
      let added = if added.is_absolute() {
        added
      } else {
        ArcPath::from(self.base_dir.join(added.as_ref()))
      };

      // Skip ignored paths AND anything inside an ignored directory.
      if ignored.is_ignored(added.to_str().expect("Path should be valid UTF-8")) {
        continue;
      }

      watch_tracker.add(added);
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
  ignored: IgnoredMatcher,
  /// Baseline mtime for registered files, captured at scan time.
  /// Used to filter stale FSEvents that arrive for files not actually modified.
  /// See: https://gist.github.com/stormslowly/ed758500de6f23211fd63b39eba5ed07
  file_mtimes: ArcPathDashMap<SystemTime>,
}

impl PathManager {
  /// Create a new `PathManager` with an optional ignored paths filter.
  pub fn new(ignored: FsWatcherIgnored) -> Self {
    Self {
      files: PathTracker::default(),
      directories: PathTracker::default(),
      missing: PathTracker::default(),
      ignored: IgnoredMatcher::new(ignored),
      file_mtimes: ArcPathDashMap::default(),
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
          true
        } else {
          false
        }
      }
      None => {
        self.file_mtimes.insert(path.clone(), current_mtime);
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
    }

    Ok(())
  }

  /// Create a new `PathAccessor` to access the current state of paths, directories, and missing paths.
  pub fn access(&self) -> PathAccessor<'_> {
    PathAccessor::new(self)
  }

  /// Whether `path` is excluded from watching by the configured ignored
  /// patterns — directly, or by living inside an ignored directory.
  pub fn is_ignored_path(&self, path: &Path) -> bool {
    match path.to_str() {
      Some(s) => self.ignored.is_ignored(s),
      None => false,
    }
  }
}

impl PathManager {
  /// Resolve a file's mtime in epoch-millis, preferring the cached
  /// `file_mtimes` baseline (no syscall) and falling back to a fresh stat for
  /// files with no baseline yet (e.g. registered-but-missing deps).
  fn file_mtime_ms(&self, path: &ArcPath) -> Option<u64> {
    if let Some(mtime) = self.file_mtimes.get(path) {
      return Some(time_info::system_time_to_millis(*mtime));
    }
    Self::stat_mtime_ms(path)
  }

  /// Read a path's mtime in epoch-millis with a fresh stat (used for
  /// directories, which have no baseline cache).
  fn stat_mtime_ms(path: &ArcPath) -> Option<u64> {
    path
      .metadata()
      .and_then(|m| m.modified().or_else(|_| m.created()))
      .ok()
      .map(time_info::system_time_to_millis)
  }

  /// Build the file and context time-info maps for all registered paths,
  /// mirroring watchpack's `fileTimeInfoEntries` / `contextTimeInfoEntries`.
  ///
  /// Files reuse the `file_mtimes` baseline. A directory's safe time is the max
  /// of its own mtime and the safe times of every registered file beneath it,
  /// so an in-directory content change — which does NOT bump the directory
  /// mtime — still advances it. Paths in `deleted` are forced to null, guarding
  /// the stale-baseline case where a just-removed file is still registered with
  /// an old cached mtime.
  pub(crate) fn collect_time_info(
    &self,
    deleted: &rspack_util::fx_hash::FxHashSet<String>,
  ) -> (Vec<TimeInfoEntry>, Vec<TimeInfoEntry>) {
    let accessor = self.access();

    // ---- files ----
    let file_paths: Vec<ArcPath> = accessor.files().0.iter().map(|p| p.clone()).collect();
    let mut file_entries = Vec::with_capacity(file_paths.len());
    let mut file_safe_times: Vec<(ArcPath, u64)> = Vec::new();
    for path in &file_paths {
      let key = path.to_string_lossy().to_string();
      let mtime_ms = if deleted.contains(&key) {
        None
      } else {
        self.file_mtime_ms(path)
      };
      match mtime_ms {
        Some(ms) => {
          let st = time_info::safe_time(ms);
          file_safe_times.push((path.clone(), st));
          file_entries.push(TimeInfoEntry {
            path: key,
            safe_time: Some(st),
            timestamp: Some(ms),
          });
        }
        None => file_entries.push(TimeInfoEntry {
          path: key,
          safe_time: None,
          timestamp: None,
        }),
      }
    }
    // Registered-missing dependencies read as null in the file map. webpack
    // registers `files` and `missing` as disjoint sets, so a null missing entry
    // never clobbers a real file entry sharing the same key in the final map.
    for path in accessor.missing().0.iter() {
      file_entries.push(TimeInfoEntry {
        path: path.to_string_lossy().to_string(),
        safe_time: None,
        timestamp: None,
      });
    }

    // ---- contexts ----
    let dir_paths: Vec<ArcPath> = accessor.directories().0.iter().map(|p| p.clone()).collect();
    // `dir_safe[i]` is the running safe time for `dir_paths[i]`, seeded from the
    // directory's own mtime.
    let mut dir_safe: Vec<Option<u64>> = dir_paths
      .iter()
      .map(|dir| Self::stat_mtime_ms(dir).map(time_info::safe_time))
      .collect();
    // Index registered directories by their raw path bytes so the ancestor walk
    // can probe them without allocating an `ArcPath` per step. Byte matching is
    // identical to the previous `ArcPath` keying, whose hash is `hash_path` over
    // exactly these bytes.
    let mut dir_index: rspack_util::fx_hash::FxHashMap<&std::ffi::OsStr, usize> =
      rspack_util::fx_hash::FxHashMap::with_capacity_and_hasher(
        dir_paths.len(),
        Default::default(),
      );
    for (i, dir) in dir_paths.iter().enumerate() {
      dir_index.insert(dir.as_os_str(), i);
    }
    // Raise each registered ancestor directory by its descendant files' safe
    // times (mirrors `Trigger::recurse_parent_directories`). On unix the
    // ancestors are byte prefixes of the path truncated at `/`, found with a
    // vectorized reverse scan — no per-step `Path::parent` component re-parse and
    // no allocation. Matching stays byte-identical to the `dir_index` keys.
    for (file, st) in &file_safe_times {
      let mut raise = |i: usize| {
        dir_safe[i] = Some(dir_safe[i].map_or(*st, |cur| cur.max(*st)));
      };

      #[cfg(unix)]
      {
        use std::{ffi::OsStr, os::unix::ffi::OsStrExt};
        let bytes = file.as_os_str().as_bytes();
        let mut end = bytes.len();
        while let Some(pos) = memchr::memrchr(b'/', &bytes[..end]) {
          if pos == 0 {
            // Reached the root; the parent is "/" unless the file IS "/".
            if end > 1
              && let Some(&i) = dir_index.get(OsStr::from_bytes(b"/"))
            {
              raise(i);
            }
            break;
          }
          if let Some(&i) = dir_index.get(OsStr::from_bytes(&bytes[..pos])) {
            raise(i);
          }
          end = pos;
        }
      }

      // Non-unix path semantics (drive prefixes, UNC, verbatim paths) are
      // intricate; keep the exact `Path::parent` walk there — correctness over
      // speed on a platform that is not the watcher's performance hot path.
      #[cfg(not(unix))]
      {
        let mut cursor = file.parent();
        while let Some(dir) = cursor {
          if let Some(&i) = dir_index.get(dir.as_os_str()) {
            raise(i);
          }
          cursor = dir.parent();
        }
      }
    }
    let context_entries = dir_paths
      .iter()
      .zip(&dir_safe)
      .map(|(dir, &safe_time)| TimeInfoEntry {
        path: dir.to_string_lossy().to_string(),
        safe_time,
        timestamp: None,
      })
      .collect();

    (file_entries, context_entries)
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

    updater
      .update(&path_tracker, &IgnoredMatcher::new(ignored))
      .unwrap();

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

  /// `collect_time_info` reports an existing file with both safe_time and
  /// timestamp, a registered-but-absent file as null (safe_time None), and a
  /// directory whose safe_time is at least the max of its contained files
  /// (watchpack's "max safeTime of children").
  #[test]
  fn collect_time_info_files_dirs_and_nulls() {
    use rspack_util::fx_hash::FxHashSet;

    let dir = tempfile::tempdir().expect("tempdir");
    let ctx = ArcPath::from(dir.path());
    let present = ArcPath::from(dir.path().join("present.js").as_path());
    let gone = ArcPath::from(dir.path().join("gone.js").as_path());
    std::fs::write(present.as_ref(), b"x").expect("write present");

    let pm = PathManager::default();
    pm.update(
      (
        vec![present.clone(), gone.clone()].into_iter(),
        std::iter::empty(),
      ),
      (vec![ctx.clone()].into_iter(), std::iter::empty()),
      (std::iter::empty(), std::iter::empty()),
    )
    .expect("register paths");
    // Seed the baseline the way a real `watch()` cycle does.
    if let Ok(mtime) = present.as_ref().metadata().and_then(|m| m.modified()) {
      pm.set_file_mtime_if_absent(present.clone(), mtime);
    }

    let deleted: FxHashSet<String> = FxHashSet::default();
    let (files, contexts) = pm.collect_time_info(&deleted);

    let find = |v: &[crate::TimeInfoEntry], p: &ArcPath| {
      v.iter().find(|e| e.path == p.to_string_lossy()).cloned()
    };

    let present_entry = find(&files, &present).expect("present file entry");
    assert!(
      present_entry.safe_time.is_some(),
      "present file has safe_time"
    );
    assert!(
      present_entry.timestamp.is_some(),
      "present file has timestamp"
    );

    let gone_entry = find(&files, &gone).expect("gone file entry");
    assert!(
      gone_entry.safe_time.is_none(),
      "absent file -> null safe_time"
    );

    let ctx_entry = find(&contexts, &ctx).expect("context entry");
    assert_eq!(
      ctx_entry.timestamp, None,
      "context entry carries no timestamp"
    );
    assert!(
      ctx_entry.safe_time.unwrap() >= present_entry.safe_time.unwrap(),
      "context safe_time >= contained file safe_time",
    );
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
}
