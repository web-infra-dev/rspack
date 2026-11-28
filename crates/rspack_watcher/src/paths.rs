mod directory_scan;
mod entries;
mod registration;
pub(crate) use registration::Registration;
mod scan;
mod scan_commit;
mod time_info;
mod update;

use std::{fmt::Debug, ops::Deref, path::Path, sync::Arc};

use dashmap::setref::multiple::RefMulti;
pub(crate) use directory_scan::{DirectoryScan, ScanInput, ScannedDirectory};
use rspack_error::Result;
use rspack_paths::{InternedPath, InternedPathDashMap, InternedPathDashSet, InternedPathSet};
pub(crate) use time_info::FileRecord;

use self::{
  time_info::{FileTimes, ReportedBaseline},
  update::PathUpdater,
};
use super::{FsWatcherIgnored, ignored::IgnoredMatcher};
use crate::watcher_manager::{MemberKind, WatcherManager};

/// An iterator that chains together references to all files, directories, and missing paths
/// stored in the [`PathTracker`]. This allows iteration over all registered paths as a single sequence.
pub(crate) struct All<'a> {
  inner: Box<dyn Iterator<Item = RefMulti<'a, InternedPath>> + 'a>,
}

impl<'a> All<'a> {
  /// Creates a new `All` iterator from the given sets of files, directories, and missing paths.
  fn new(
    files: &'a InternedPathDashSet,
    directories: &'a InternedPathDashSet,
    missing: &'a InternedPathDashSet,
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
  type Item = InternedPath;

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
  pub fn files(
    &self,
  ) -> (
    &'a InternedPathDashSet,
    &'a InternedPathDashSet,
    &'a InternedPathDashSet,
  ) {
    (&self.files.all, &self.files.added, &self.files.removed)
  }

  /// Returns references to the set of directories, including added and removed directories.
  pub fn directories(
    &self,
  ) -> (
    &'a InternedPathDashSet,
    &'a InternedPathDashSet,
    &'a InternedPathDashSet,
  ) {
    (
      &self.directories.all,
      &self.directories.added,
      &self.directories.removed,
    )
  }

  /// Returns references to the set of missing paths, including added and removed missing paths.
  pub fn missing(
    &self,
  ) -> (
    &'a InternedPathDashSet,
    &'a InternedPathDashSet,
    &'a InternedPathDashSet,
  ) {
    (
      &self.missing.all,
      &self.missing.added,
      &self.missing.removed,
    )
  }

  /// Returns an iterator that combines all files, directories, and missing paths into a single sequence.
  pub fn all(&self) -> impl Iterator<Item = InternedPath> + '_ {
    All::new(&self.files.all, &self.directories.all, &self.missing.all)
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
  added: InternedPathDashSet,
  removed: InternedPathDashSet,
  all: InternedPathDashSet,
}

impl PathTracker {
  fn reset(&self) {
    self.added.clear();
    self.removed.clear();
  }

  /// Adds a path to the tracker; whether it was not tracked before.
  fn add(&self, path: InternedPath) -> bool {
    self.added.insert(path.clone());
    self.all.insert(path)
  }

  /// Removes a path from the tracker; whether it was tracked.
  fn remove(&self, path: InternedPath) -> bool {
    let tracked = self.all.remove(&path).is_some();
    self.removed.insert(path);
    tracked
  }
}

/// `PathManager` is responsible for managing the set of files, directories, and missing paths.
pub(crate) struct PathManager {
  files: PathTracker,
  directories: PathTracker,
  missing: PathTracker,
  ignored: IgnoredMatcher,
  scans_closed: std::sync::atomic::AtomicBool,
  /// Observations of registered files, missing dependencies and context
  /// members. Only explicit dependencies also need an event baseline.
  /// See: https://gist.github.com/stormslowly/ed758500de6f23211fd63b39eba5ed07
  file_times: FileTimes,
  pub(super) watchers: WatcherManager,
  /// Every directory above a registered path: a removal of one of them may
  /// have taken registered records, or a registered context, with it. Each
  /// counts the registrations directly in it plus its subdirectories here,
  /// and goes once that count is back to zero.
  registered_ancestors: InternedPathDashMap<usize>,
  /// Every path holding time info under its parent directory, ancestors included
  /// up to the first one already indexed: a removed directory's records are
  /// found by walking its subtree, not by scanning every map. A directory
  /// left with no children and no record of its own is dropped.
  recorded_children: InternedPathDashMap<InternedPathSet>,
  /// Serializes changes to `recorded_children`'s links: a walk taking a
  /// directory out on one thread must not interleave with a walk linking it
  /// in on another, or a linked subtree is left cut off from its parent.
  index_lock: std::sync::Mutex<u64>,
  /// Registered contexts known to be absent from disk: absent when registered,
  /// or removed since. The only directories `collect_time_info_entries`
  /// stats, to see one that came back without an event of its own.
  absent_directories: InternedPathDashSet,
  /// The `followSymlinks` watch option: the context scan descends into
  /// symlinked directories, as watchpack does.
  follow_symlinks: bool,
}

impl Default for PathManager {
  fn default() -> Self {
    Self::new(FsWatcherIgnored::default())
  }
}

impl PathManager {
  /// Create a new `PathManager` with an optional ignored paths filter.
  pub fn new(ignored: FsWatcherIgnored) -> Self {
    let watchers = WatcherManager::default();
    Self {
      files: PathTracker::default(),
      directories: PathTracker::default(),
      missing: PathTracker::default(),
      ignored: IgnoredMatcher::new(ignored),
      scans_closed: Default::default(),
      file_times: FileTimes(Arc::clone(&watchers.directories)),
      watchers,
      registered_ancestors: InternedPathDashMap::default(),
      absent_directories: InternedPathDashSet::default(),
      recorded_children: InternedPathDashMap::default(),
      index_lock: std::sync::Mutex::default(),
      follow_symlinks: false,
    }
  }

  pub fn with_follow_symlinks(mut self, follow_symlinks: bool) -> Self {
    self.follow_symlinks = follow_symlinks;
    self
  }

  pub(crate) fn close_scans(&self) {
    let _index = self.lock_index();
    self
      .scans_closed
      .store(true, std::sync::atomic::Ordering::Release);
    for watcher in self.watchers.directories.iter() {
      if let Some(scan) = &watcher.scan {
        scan.invalidate();
      }
    }
  }

  /// Reset the per-`watch()` diff. File records live with their subscriptions
  /// so a delayed event can still report a write made between watch cycles.
  pub fn reset(&self) {
    self.files.reset();
    self.directories.reset();
    self.missing.reset();
  }

  /// Drop a removed path's time info and present baseline. A reported absence
  /// survives later removals until the path reappears or is unregistered.
  pub fn remove_file_time(&self, path: &InternedPath) {
    let _index = self.lock_index();
    self.remove_file_record(path);
  }

  fn remove_file_record(&self, path: &InternedPath) {
    self.file_times.clear(path, FileRecord::clear_present);
    if self.watchers.remove_member(path).directory {
      self.watchers.clear_event_time(path);
    }
    self.unlink_unrecorded(path);
  }

  /// Drop the records of every path inside the directory `dir` (watchpack's
  /// `onDirectoryRemoved` marks them all missing), walking only its subtree.
  pub fn remove_file_times_under(&self, dir: &InternedPath) {
    let _index = self.lock_index();
    self.remove_subtree_records(dir, false);
  }

  fn remove_subtree_records(&self, dir: &InternedPath, keep_root: bool) {
    let mut pending = vec![dir.clone()];
    while let Some(path) = pending.pop() {
      if let Some((_, children)) = self.recorded_children.remove(&path) {
        pending.extend(children);
      }
      if keep_root && &path == dir {
        continue;
      }
      self.file_times.clear(&path, FileRecord::clear_present);
      self.watchers.remove_member(&path);
      if !self.watchers.is_explicit_context(&path) {
        self.watchers.clear_event_time(&path);
      }
    }
    self.unlink_unrecorded(dir);
    for context in self.directories.all.iter() {
      if context.starts_with(dir.as_ref() as &Path) {
        self.absent_directories.insert(context.clone());
      }
    }
  }

  /// Writes a record for `path` with `write` and indexes it, and its ancestors
  /// not indexed yet, under the index lock: a sweep of an ancestor sees the
  /// record either both written and indexed, or not at all.
  fn record<R>(&self, path: &InternedPath, write: impl FnOnce() -> R) -> R {
    let _index = self.lock_index();
    let written = write();
    self.link_record(path);
    written
  }

  fn link_record(&self, path: &InternedPath) {
    let mut child = path.clone();
    while let Some(parent) = child.parent().map(InternedPath::from) {
      let indexed = !self
        .recorded_children
        .entry(parent.clone())
        .or_default()
        .insert(child);
      if indexed {
        break;
      }
      child = parent;
    }
  }

  /// `path` may no longer hold a record of its own: once it holds none and
  /// has nothing indexed below it, it leaves its parent, and so on up.
  fn unindex_record(&self, path: &InternedPath) {
    let _index = self.lock_index();
    self.unlink_unrecorded(path);
  }

  /// [`Self::unindex_record`] under the index lock already held.
  fn unlink_unrecorded(&self, path: &InternedPath) {
    let mut child = path.clone();
    while !self.holds_record(&child) && !self.recorded_children.contains_key(&child) {
      let Some(parent) = child.parent().map(InternedPath::from) else {
        return;
      };
      let emptied = match self.recorded_children.get_mut(&parent) {
        Some(mut siblings) => siblings.remove(&child) && siblings.is_empty(),
        None => false,
      };
      if !emptied {
        return;
      }
      self
        .recorded_children
        .remove_if(&parent, |_, siblings| siblings.is_empty());
      child = parent;
    }
  }

  pub fn index_sizes(&self) -> (usize, usize) {
    let recorded = self
      .recorded_children
      .iter()
      .map(|children| 1 + children.len())
      .sum();
    (recorded, self.registered_ancestors.len())
  }

  fn lock_index(&self) -> std::sync::MutexGuard<'_, u64> {
    self.index_lock.lock().expect("record index lock poisoned")
  }

  fn holds_record(&self, path: &InternedPath) -> bool {
    self.file_times.get(path).is_some()
      || self.watchers.member(path).is_some()
      || self.watchers.event_time(path).is_some()
  }

  /// `path` was registered: count it in its directory, and each directory
  /// above that is new here in its own parent.
  fn retain_registered_ancestors(&self, path: &InternedPath) {
    let mut child = path.clone();
    while let Some(parent) = child.parent().map(InternedPath::from) {
      let mut count = self.registered_ancestors.entry(parent.clone()).or_default();
      *count += 1;
      if *count > 1 {
        return;
      }
      drop(count);
      child = parent;
    }
  }

  /// `path` was unregistered: the reverse of [`Self::retain_registered_ancestors`].
  fn release_registered_ancestors(&self, path: &InternedPath) {
    let mut child = path.clone();
    while let Some(parent) = child.parent().map(InternedPath::from) {
      let released = self
        .registered_ancestors
        .get_mut(&parent)
        .is_some_and(|mut count| {
          *count -= 1;
          *count == 0
        });
      if !released {
        return;
      }
      self
        .registered_ancestors
        .remove_if(&parent, |_, count| *count == 0);
      child = parent;
    }
  }

  /// Whether removing `path` can take recorded paths below it along: it is a
  /// registered or discovered context, or a directory above a registered
  /// path.
  pub fn may_contain_records(&self, path: &InternedPath) -> bool {
    self.watchers.is_watched_directory(path) || self.registered_ancestors.contains_key(path)
  }

  fn is_below_context(&self, path: &InternedPath) -> bool {
    path.ancestors().skip(1).any(|ancestor| {
      self
        .watchers
        .is_explicit_context(&InternedPath::from(ancestor))
    })
  }

  /// Update the paths, directories, and missing paths in the `PathManager`.
  pub async fn update(
    &self,
    files: (
      impl Iterator<Item = InternedPath>,
      impl Iterator<Item = InternedPath>,
    ),
    directories: (
      impl Iterator<Item = InternedPath>,
      impl Iterator<Item = InternedPath>,
    ),
    missing: (
      impl Iterator<Item = InternedPath>,
      impl Iterator<Item = InternedPath>,
    ),
  ) -> Result<()> {
    let mut registered = Vec::new();
    let mut unregistered = Vec::new();
    for (paths, tracker) in [
      (PathUpdater::from(files), &self.files),
      (
        PathUpdater::from(directories).with_contexts(&self.watchers),
        &self.directories,
      ),
      (PathUpdater::from(missing), &self.missing),
    ] {
      let (added, removed) = paths.update(tracker, &self.ignored).await?;
      registered.extend(added);
      unregistered.extend(removed);
    }
    for path in &registered {
      self.watchers.register_child(path);
      self.retain_registered_ancestors(path);
    }
    for path in &unregistered {
      self.watchers.unregister_child(path);
      self.release_registered_ancestors(path);
    }

    for dir in self.directories.added.iter() {
      if dir.exists() {
        self.absent_directories.remove(&*dir);
      } else {
        self.absent_directories.insert(dir.clone());
      }
    }
    // Prune per-path state for paths no longer being watched so the maps do
    // not grow unboundedly across `watch()` cycles. `reset()` has already
    // cleared the `removed` sets at the start of this `watch()` call, so what
    // we see here is only this cycle's removals. A missing path that this
    // cycle re-registers as a file keeps its baseline: re-seeding it from a
    // fresh stat would reopen the rewatch-gap race `set_file_time_if_absent`
    // guards against. A path still below a registered context keeps its
    // record: the context reports it.
    let removed_files: Vec<InternedPath> = self.files.removed.iter().map(|p| p.clone()).collect();
    for path in &removed_files {
      self.prune_file_record(path, self.missing.all.contains(path));
    }
    let removed_missing: Vec<InternedPath> =
      self.missing.removed.iter().map(|p| p.clone()).collect();
    for path in &removed_missing {
      if !self.files.all.contains(path) {
        self.prune_file_record(path, false);
      }
    }
    let removed_dirs: Vec<InternedPath> =
      self.directories.removed.iter().map(|p| p.clone()).collect();
    for dir in &removed_dirs {
      // Still below a registered context and on disk: it stays reported, as a
      // directory found there, and keeps its event clock. Absent, it is
      // forgotten, so it is scanned as a new directory when it comes back.
      let absent = self.absent_directories.remove(dir).is_some();
      if !absent && self.is_below_context(dir) {
        self.record(dir, || self.watchers.add_member(dir, MemberKind::Directory));
      } else {
        self.watchers.clear_event_time(dir);
      }
      self.forget_context_entries_under(dir);
    }

    Ok(())
  }

  fn prune_file_record(&self, path: &InternedPath, keep_reported: bool) {
    let below_context = self.is_below_context(path);
    self.file_times.clear(path, |record| {
      if !keep_reported {
        record.reported = ReportedBaseline::Unknown;
      }
      if !below_context {
        record.time = None;
      }
    });
    if !below_context && !keep_reported {
      self.unindex_record(path);
    }
  }

  /// A context was unregistered: forget what its scan found below it, except
  /// what another still-registered context also covers.
  fn forget_context_entries_under(&self, dir: &InternedPath) {
    let _index = self.lock_index();
    let mut subtree = Vec::new();
    let mut pending = vec![dir.clone()];
    while let Some(path) = pending.pop() {
      if let Some(children) = self.recorded_children.get(&path) {
        pending.extend(children.iter().cloned());
      }
      subtree.push(path);
    }
    for path in &subtree {
      if self.is_below_context(path) {
        continue;
      }
      if self.watchers.is_explicit_context(path) {
        // A directory symlink can still export both its scanned file time and its own context.
        self.watchers.unsubscribe_parent(path);
        continue;
      }
      let member = self.watchers.remove_member(path);
      if member.file && !self.files.all.contains(path) && !self.missing.all.contains(path) {
        self.file_times.clear(path, |record| record.time = None);
        self.unlink_unrecorded(path);
      }
      if member.directory {
        self.watchers.clear_event_time(path);
        self.unlink_unrecorded(path);
      }
    }
  }

  /// Create a new `PathAccessor` to access the current state of paths, directories, and missing paths.
  pub fn access(&self) -> PathAccessor<'_> {
    PathAccessor::new(self)
  }

  /// Whether `path` is excluded from watching by the configured ignored
  /// patterns — directly, or by living inside an ignored directory.
  pub async fn is_ignored_path(&self, path: &Path) -> bool {
    match path.to_str() {
      Some(s) => self.ignored.is_ignored(s).await,
      None => false,
    }
  }
}

#[cfg(test)]
mod tests {
  use rspack_paths::Utf8Path;

  use super::*;

  #[tokio::test]
  async fn test_updater() {
    let updater = PathUpdater::from((
      vec![
        InternedPath::from(Utf8Path::new("src/index.js")),
        InternedPath::from(Utf8Path::new("node_modules/.pnpm/axios/lib/index.js")),
        InternedPath::from(Utf8Path::new(".git/abc/")),
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
      .await
      .unwrap();

    let all = path_tracker.all;

    assert_eq!(all.len(), 1);
    assert!(
      all
        .iter()
        .any(|p| p.to_string_lossy().contains("src/index.js"))
    )
  }

  #[tokio::test]
  async fn test_accessor() {
    let path_manager = PathManager::default();

    let files = (
      vec![InternedPath::from(Utf8Path::new("src/index.js"))].into_iter(),
      vec![].into_iter(),
    );
    let dirs = (
      vec![InternedPath::from(Utf8Path::new("src"))].into_iter(),
      vec![].into_iter(),
    );
    let missing = (
      vec![InternedPath::from(Utf8Path::new("src/page/index.ts"))].into_iter(),
      vec![].into_iter(),
    );

    path_manager.update(files, dirs, missing).await.unwrap();

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

  #[tokio::test]
  async fn test_manager() {
    let ignored = FsWatcherIgnored::Paths(vec![
      "**/node_modules/**".to_string(),
      "**/.git/**".to_string(),
    ]);
    let path_manager = PathManager::new(ignored);
    let files = (
      vec![InternedPath::from(Utf8Path::new("src/index.js"))].into_iter(),
      vec![].into_iter(),
    );
    let directories = (
      vec![
        InternedPath::from(Utf8Path::new("src/")),
        InternedPath::from(Utf8Path::new("node_modules/")),
      ]
      .into_iter(),
      vec![].into_iter(),
    );
    let missing = (
      vec![InternedPath::from(Utf8Path::new("src/page/index.ts"))].into_iter(),
      vec![].into_iter(),
    );

    path_manager
      .update(files, directories, missing)
      .await
      .unwrap();

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
  #[tokio::test]
  async fn test_baseline_persists_across_consecutive_watch_cycles() {
    use std::{thread::sleep, time::Duration};

    use tempfile::NamedTempFile;

    let tempfile = NamedTempFile::new().expect("create temp file");
    let path = InternedPath::from(tempfile.path());

    let pm = PathManager::default();
    pm.update(
      (std::iter::once(path.clone()), std::iter::empty()),
      (std::iter::empty(), std::iter::empty()),
      (std::iter::empty(), std::iter::empty()),
    )
    .await
    .expect("register file");

    // T0 — first `watch()` cycle records the baseline.
    let initial_mtime = tempfile
      .path()
      .metadata()
      .and_then(|m| m.modified())
      .expect("read initial mtime");
    pm.set_file_time_if_absent(path.clone(), initial_mtime);

    // T3 — rspack starts a rebuild; `reset()` runs ahead of the next `watch()`.
    pm.reset();

    // Invariant: `file_times` must survive `reset()`. Without this the
    // baseline gets wiped on every watch cycle and the next bullet point
    // can no longer hold.
    assert_eq!(
      pm.file_times.get(&path).map(|v| v.mtime),
      Some(initial_mtime),
      "file_times must persist across reset()",
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
    pm.set_file_time_if_absent(path.clone(), post_write_mtime);
    assert_eq!(
      pm.file_times.get(&path).map(|v| v.mtime),
      Some(initial_mtime),
      "set_file_time_if_absent must not overwrite an existing baseline",
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
      pm.file_times.get(&path).map(|v| v.mtime),
      Some(post_write_mtime),
      "has_mtime_changed should advance the baseline on a real change",
    );
  }
}
