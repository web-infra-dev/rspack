use std::{
  ffi::OsStr,
  fmt::Debug,
  ops::Deref,
  path::{Path, PathBuf},
  sync::Arc,
  time::SystemTime,
};

use dashmap::{mapref::one::RefMut, setref::multiple::RefMulti};
use rspack_error::Result;
use rspack_paths::{
  InternedPath, InternedPathDashMap, InternedPathDashSet, InternedPathIndexSet, InternedPathSet,
};
use rspack_util::{
  fx_hash::FxHashMap,
  time::{current_time, system_time_to_millis},
};

use super::{FsWatcherIgnored, TimeInfoEntries, TimeInfoEntry, ignored::IgnoredMatcher};
use crate::{
  observation::{Mutation, Observations, Scan, ScanEpoch},
  time::{fs_accuracy, mtime_accuracy, mtime_safe_time, mtime_to_millis},
  watcher_manager::{ContextChange, MemberKind, WatcherManager},
};

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

/// `PathUpdater` is used to update collections of registered paths (files, directories, and missing paths)
/// by specifying which paths have been added and which have been removed. It holds vectors of paths to be
/// added and removed, and provides functionality to apply these changes to a path tracker. This struct
/// facilitates batch updates to the path sets, ensuring that additions and removals are processed efficiently.
struct PathUpdater<'a> {
  pub added: Vec<InternedPath>,
  pub removed: Vec<InternedPath>,
  base_dir: PathBuf,
  contexts: Option<&'a WatcherManager>,
}

impl<Added, Removed> From<(Added, Removed)> for PathUpdater<'_>
where
  Added: Iterator<Item = InternedPath>,
  Removed: Iterator<Item = InternedPath>,
{
  fn from((added, removed): (Added, Removed)) -> Self {
    Self {
      added: added.collect(),
      removed: removed.collect(),
      base_dir: std::env::current_dir().unwrap_or_default(),
      contexts: None,
    }
  }
}

impl<'a> PathUpdater<'a> {
  fn with_contexts(mut self, contexts: &'a WatcherManager) -> Self {
    self.contexts = Some(contexts);
    self
  }

  /// Update the paths in the given set. Returns the paths it newly holds and
  /// those it no longer does.
  async fn update(
    self,
    watch_tracker: &PathTracker,
    ignored: &IgnoredMatcher,
  ) -> Result<(Vec<InternedPath>, Vec<InternedPath>)> {
    let added_paths = self.added;
    let removed_paths = self.removed;
    let mut registered = Vec::new();
    let mut unregistered = Vec::new();

    for added in added_paths {
      // Absolutize before the ignored check so registration filters on the
      // same absolute path the watcher reports for events.
      let added = if added.is_absolute() {
        added
      } else {
        InternedPath::from(self.base_dir.join(added.as_ref()))
      };

      // Skip ignored paths AND anything inside an ignored directory.
      if ignored
        .is_ignored(added.to_str().expect("Path should be valid UTF-8"))
        .await
      {
        continue;
      }

      if watch_tracker.add(added.clone()) {
        if let Some(contexts) = self.contexts {
          contexts.subscribe_context(&added);
        }
        registered.push(added);
      }
    }

    for removed in removed_paths {
      let removed = if removed.is_absolute() {
        removed
      } else {
        InternedPath::from(self.base_dir.join(removed.as_ref()))
      };
      if watch_tracker.remove(removed.clone()) {
        if let Some(contexts) = self.contexts {
          contexts.unsubscribe_context(&removed);
        }
        unregistered.push(removed);
      }
    }
    Ok((registered, unregistered))
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

/// One value of watchpack's `files` map, as written by `setFileTime`: the mtime
/// a path was last seen with, plus the safe time and accuracy fixed at that
/// observation.
#[derive(Debug, Clone, Copy)]
pub(crate) struct FileTime {
  mtime: SystemTime,
  safe_time: u64,
  accuracy: u64,
}

impl FileTime {
  /// watchpack `setFileTime(initial = true)`: the path was found on disk by a
  /// scan, so only its mtime says when it changed — the safe time is that
  /// mtime padded by the filesystem's timestamp accuracy.
  fn initial(mtime: SystemTime) -> Self {
    let mtime_ms = mtime_to_millis(mtime);
    let accuracy = mtime_accuracy(mtime);
    Self {
      mtime,
      safe_time: current_time().min(mtime_ms).saturating_add(accuracy),
      accuracy,
    }
  }

  /// watchpack's `fixupEntryAccuracy`: a record taken while the filesystem
  /// accuracy was coarser is read with the accuracy learned since.
  fn narrowed(self) -> Self {
    let accuracy = fs_accuracy();
    if self.accuracy <= accuracy {
      return self;
    }
    Self {
      safe_time: self.safe_time - self.accuracy + accuracy,
      accuracy,
      ..self
    }
  }

  /// watchpack `setFileTime(initial = false)`: a live event just reported the
  /// path. The change happened no later than now, whatever mtime the new
  /// content carries (a restored backup, a timestamp-preserving copy), so the
  /// safe time is now, exactly.
  fn observed(mtime: SystemTime) -> Self {
    mtime_accuracy(mtime);
    Self {
      mtime,
      safe_time: current_time(),
      accuracy: 0,
    }
  }
}

#[derive(Clone, Copy, Default, PartialEq, Eq)]
enum ReportedBaseline {
  #[default]
  Unknown,
  Present(SystemTime),
  Absent,
}

#[derive(Default)]
struct FileRecord {
  /// Last observation exported by `collect_time_info_entries`.
  time: Option<FileTime>,
  /// State the build already knows from registration or a reported event.
  reported: ReportedBaseline,
}

impl FileRecord {
  fn set_time(&mut self, mtime: SystemTime, initial: bool, ignore_when_equal: bool) -> bool {
    // Seeing the file again clears a reported absence. A scan that only
    // records its time must still let the file's first event through.
    self.clear_absent();
    if ignore_when_equal && self.time.is_some_and(|time| time.mtime == mtime) {
      return false;
    }
    self.time = Some(if initial {
      FileTime::initial(mtime)
    } else {
      FileTime::observed(mtime)
    });
    true
  }

  fn observe_member(&mut self, mtime: SystemTime, origin: &ContextObservation) -> bool {
    match origin {
      ContextObservation::Scan { initial, .. } => {
        self.set_time(mtime, *initial, true);
      }
      ContextObservation::Event => {
        if self.time.is_some_and(|old| {
          old.mtime == mtime && mtime_to_millis(mtime) + mtime_accuracy(mtime) < current_time()
        }) {
          return false;
        }
        self.set_time(mtime, false, false);
      }
      ContextObservation::Reported => {}
    }
    true
  }

  fn clear_present(&mut self) {
    self.time = None;
    if matches!(self.reported, ReportedBaseline::Present(_)) {
      self.reported = ReportedBaseline::Unknown;
    }
  }

  fn clear_absent(&mut self) {
    if self.reported == ReportedBaseline::Absent {
      self.reported = ReportedBaseline::Unknown;
    }
  }
}

#[derive(Default)]
struct FileTimes {
  records: InternedPathDashMap<FileRecord>,
}

impl FileTimes {
  fn get(&self, path: &InternedPath) -> Option<FileTime> {
    self.records.get(path)?.time
  }

  fn reported(&self, path: &InternedPath) -> ReportedBaseline {
    self
      .records
      .get(path)
      .map_or(ReportedBaseline::Unknown, |record| record.reported)
  }

  fn record(&self, path: &InternedPath) -> RefMut<'_, InternedPath, FileRecord> {
    self.records.entry(path.clone()).or_default()
  }

  fn clear(&self, path: &InternedPath, clear: impl FnOnce(&mut FileRecord)) {
    self.records.remove_if_mut(path, |_, record| {
      clear(record);
      record.time.is_none() && record.reported == ReportedBaseline::Unknown
    });
  }
}

enum ContextObservation {
  Scan {
    initial: bool,
    changed_since: Option<u64>,
  },
  Event,
  Reported,
}

/// `PathManager` is responsible for managing the set of files, directories, and missing paths.
#[derive(Default)]
pub(crate) struct PathManager {
  files: PathTracker,
  directories: PathTracker,
  missing: PathTracker,
  ignored: IgnoredMatcher,
  /// Observations of registered files, missing dependencies and context
  /// members. Only explicit dependencies also need an event baseline.
  /// See: https://gist.github.com/stormslowly/ed758500de6f23211fd63b39eba5ed07
  file_times: FileTimes,
  /// watchpack's per-`DirectoryWatcher` `lastWatchEvent`: when an event last
  /// reached each registered or discovered context (its own or a
  /// descendant's), in epoch millis. Like watchpack it starts at 0 (absent)
  /// and only events advance it; a context's safe time is the max of it and
  /// the safe times of the files below.
  last_watch_events: InternedPathDashMap<u64>,
  pub(super) watchers: WatcherManager,
  /// Every directory above a registered path: a removal of one of them may
  /// have taken registered records, or a registered context, with it. Each
  /// counts the registrations directly in it plus its subdirectories here,
  /// and goes once that count is back to zero.
  registered_ancestors: InternedPathDashMap<usize>,
  /// Every path holding time info (in `file_times`, the directory watchers or
  /// `last_watch_events`) under its parent directory, ancestors included up
  /// to the first one already indexed: a removed directory's records are
  /// found by walking its subtree, not by scanning every map. A directory
  /// left with no children and no record of its own is dropped.
  recorded_children: InternedPathDashMap<InternedPathSet>,
  /// Serializes changes to `recorded_children`'s links: a walk taking a
  /// directory out on one thread must not interleave with a walk linking it
  /// in on another, or a linked subtree is left cut off from its parent.
  index_lock: std::sync::Mutex<()>,
  /// Registered contexts known to be absent from disk: absent when registered,
  /// or removed since. The only directories `collect_time_info_entries`
  /// stats, to see one that came back without an event of its own.
  absent_directories: InternedPathDashSet,
  /// The `followSymlinks` watch option: the context scan descends into
  /// symlinked directories, as watchpack does.
  follow_symlinks: bool,
}

impl PathManager {
  /// Create a new `PathManager` with an optional ignored paths filter.
  pub fn new(ignored: FsWatcherIgnored) -> Self {
    Self {
      files: PathTracker::default(),
      directories: PathTracker::default(),
      missing: PathTracker::default(),
      ignored: IgnoredMatcher::new(ignored),
      file_times: FileTimes::default(),
      last_watch_events: InternedPathDashMap::default(),
      watchers: WatcherManager::default(),
      registered_ancestors: InternedPathDashMap::default(),
      absent_directories: InternedPathDashSet::default(),
      recorded_children: InternedPathDashMap::default(),
      index_lock: std::sync::Mutex::default(),
      follow_symlinks: false,
    }
  }

  // A directory walk can still update shared members while any original context covers them.
  fn scan_has_subscription(
    &self,
    state: &Observations,
    scan: &Scan,
    directory: Option<&Path>,
  ) -> bool {
    let current = |context: &InternedPath| {
      self.watchers.is_explicit_context(context) && state.same_subscription(scan, context)
    };
    current(&scan.root)
      || directory.is_some_and(|dir| {
        dir
          .ancestors()
          .map(InternedPath::from)
          .any(|context| current(&context))
      })
  }

  fn can_scan_children(&self, scan: &Scan, path: &InternedPath) -> bool {
    let state = self.watchers.lock_observations();
    self.scan_has_subscription(&state, scan, Some(path)) && state.covers_subtree(scan, path)
  }

  // Check and commit together, after reading metadata and awaiting ignored.
  pub(super) fn observe<R>(
    &self,
    path: &InternedPath,
    scan: Option<&Scan>,
    write: impl FnOnce() -> (R, Mutation),
  ) -> Option<R> {
    self.observe_in_directory(path, scan, None, |_| write())
  }

  fn observe_in_directory<R>(
    &self,
    path: &InternedPath,
    scan: Option<&Scan>,
    directory: Option<&Path>,
    write: impl FnOnce(&Observations) -> (R, Mutation),
  ) -> Option<R> {
    let mut state = self.watchers.lock_observations();
    if scan
      .is_some_and(|s| !self.scan_has_subscription(&state, s, directory) || !state.accepts(s, path))
    {
      return None;
    }
    let (result, mutation) = write(&state);
    state.observe(path, scan, mutation);
    Some(result)
  }

  pub fn with_follow_symlinks(mut self, follow_symlinks: bool) -> Self {
    self.follow_symlinks = follow_symlinks;
    self
  }

  /// Reset the per-`watch()` diff. File records live with their subscriptions
  /// so a delayed event can still report a write made between watch cycles.
  pub fn reset(&self) {
    self.files.reset();
    self.directories.reset();
    self.missing.reset();
  }

  /// Record an initial baseline mtime for `path`, but only if no present
  /// baseline already exists. A reported absence counts as no
  /// baseline because the file is back. This is the "incremental" form used by
  /// `FsWatcher::wait_for_event`: on the first `watch()` call we record
  /// mtimes for all newly-registered files; on subsequent `watch()` calls
  /// we MUST skip files that already have a baseline, otherwise we risk
  /// snapshotting a mtime that the user has just bumped via `writeFile`.
  ///
  /// Use `has_mtime_changed` (which atomically reads-and-updates the
  /// baseline) to advance the recorded mtime in response to real events.
  pub fn set_file_time_if_absent(&self, path: InternedPath, mtime: SystemTime) {
    self.record(&path, || {
      let mut record = self.file_times.record(&path);
      if !matches!(record.reported, ReportedBaseline::Present(_)) {
        record.reported = ReportedBaseline::Present(mtime);
      }
      record.time.get_or_insert_with(|| FileTime::initial(mtime));
    });
  }

  /// The scanner reported `path` as changed: the build will know this mtime,
  /// so it becomes the deduplication baseline as well as the time info.
  pub fn set_reported_file_time(&self, path: &InternedPath, mtime: SystemTime) {
    self.record(path, || {
      let mut record = self.file_times.record(path);
      record.reported = ReportedBaseline::Present(mtime);
      record.set_time(mtime, true, true);
    });
  }

  /// Whether a removal was already reported for this path. The scanner must
  /// still report an absence with no prior record.
  pub fn is_known_absent(&self, path: &InternedPath) -> bool {
    self.file_times.reported(path) == ReportedBaseline::Absent
  }

  /// Record a removal the caller is about to deliver. The scanner relies on
  /// this marker to mean the build was told about the absence.
  fn set_reported_file_removal(&self, path: &InternedPath) {
    self.file_times.record(path).reported = ReportedBaseline::Absent;
    if !self.files.all.contains(path) && !self.missing.all.contains(path) {
      self.file_times.clear(path, FileRecord::clear_absent);
    }
  }

  pub(super) fn set_missing(
    &self,
    path: &InternedPath,
    scan: Option<&Scan>,
    report_absence: bool,
  ) -> bool {
    self
      .observe(path, scan, || {
        let subtree = self.may_contain_records(path);
        self.remove_file_time(path, false);
        if subtree {
          self.remove_file_times_under(path, false);
        }
        if report_absence {
          self.set_reported_file_removal(path);
        }
        ((), Mutation::Subtree)
      })
      .is_some()
  }

  /// Drop a removed path's time info and present baseline. A reported absence
  /// survives later removals until the path reappears or is unregistered.
  fn remove_file_time(&self, path: &InternedPath, keep_file: bool) {
    let _index = self.lock_index();
    if !keep_file {
      self.file_times.clear(path, FileRecord::clear_present);
    }
    if self.watchers.remove_member(path).directory {
      self.last_watch_events.remove(path);
    }
    self.unlink_unrecorded(path);
  }

  /// Drop the records of every path inside the directory `dir` (watchpack's
  /// `onDirectoryRemoved` marks them all missing), walking only its subtree.
  fn remove_file_times_under(&self, dir: &InternedPath, keep_root_file: bool) {
    let _index = self.lock_index();
    let mut pending = vec![dir.clone()];
    while let Some(path) = pending.pop() {
      if let Some((_, children)) = self.recorded_children.remove(&path) {
        pending.extend(children);
      }
      if !keep_root_file || &path != dir {
        self.file_times.clear(&path, FileRecord::clear_present);
      }
      self.watchers.remove_member(&path);
      if !self.watchers.is_explicit_context(&path) {
        self.last_watch_events.remove(&path);
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
    written
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

  fn lock_index(&self) -> std::sync::MutexGuard<'_, ()> {
    self.index_lock.lock().expect("record index lock poisoned")
  }

  fn holds_record(&self, path: &InternedPath) -> bool {
    self.file_times.get(path).is_some()
      || self.watchers.has_member(path, |_| true)
      || self.last_watch_events.contains_key(path)
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

  /// watchpack's initial scan of a `DirectoryWatcher`, for this cycle's newly
  /// registered contexts: index every file and subdirectory below them, so
  /// `collect_time_info_entries` reports them like watchpack does. A path the
  /// live watch already observed with the same mtime keeps its record.
  ///
  /// Returns the files that changed since `start_time` (watchpack's
  /// `checkStartTime`): they changed before the watch was active, so no event
  /// will come for them.
  pub async fn scan_contexts(&self, start_time: SystemTime) -> Vec<(InternedPath, ContextChange)> {
    let contexts: Vec<InternedPath> = self.directories.added.iter().map(|p| p.clone()).collect();
    self
      .scan_below(contexts, false, Some(system_time_to_millis(start_time)))
      .await
  }

  /// Index every file and subdirectory below `roots`, minus what `ignored`
  /// excludes (watchpack's `DirectoryWatcher` initial scan). Roots that are
  /// `recovered` contexts coming back are rescanned like watchpack's
  /// `doScan(false)`: the files directly in them are stamped with the
  /// observation time, since one changed while away may have kept its mtime.
  /// Returns the files whose safe time is at or after `changed_since`.
  async fn scan_below(
    &self,
    roots: Vec<InternedPath>,
    recovered: bool,
    changed_since: Option<u64>,
  ) -> Vec<(InternedPath, ContextChange)> {
    let mut changed = Vec::new();
    if roots.is_empty() {
      return changed;
    }
    let epoch = ScanEpoch::start(&self.watchers.observations);
    // Each directory carries the real paths of the directories above it: with
    // `followSymlinks`, a link back into its own ancestry is a loop, while two
    // links to one directory are two aliases, each scanned.
    let mut pending: Vec<_> = roots
      .into_iter()
      .filter_map(|dir| {
        let root = dir
          .ancestors()
          .map(InternedPath::from)
          .find(|p| self.watchers.is_explicit_context(p))?;
        let scan = Scan {
          epoch: Arc::clone(&epoch),
          root,
        };
        Some((dir, Vec::new(), recovered, scan))
      })
      .collect();
    while let Some((dir, mut ancestry, observed, scan)) = pending.pop() {
      if !self.can_scan_children(&scan, &dir) {
        continue;
      }
      if self.follow_symlinks {
        let Ok(real) = std::fs::canonicalize(&dir) else {
          continue;
        };
        if ancestry.contains(&real) {
          continue;
        }
        ancestry.push(real);
      }
      let Ok(entries) = std::fs::read_dir(&dir) else {
        continue;
      };
      for entry in entries.flatten() {
        let path = InternedPath::from(entry.path());
        if self.is_ignored_path(path.as_ref()).await {
          continue;
        }
        let Ok((metadata, symlink)) = Self::scan_metadata(ancestry.last(), &path) else {
          continue;
        };
        let Some((_, change)) = self.observe_member(
          &path,
          &metadata,
          symlink,
          Some(&scan),
          ContextObservation::Scan {
            initial: !observed,
            changed_since,
          },
        ) else {
          if metadata.is_dir() && self.watchers.has_member(&path, |member| member.directory) {
            pending.push((path, ancestry.clone(), false, scan.clone()));
          }
          continue;
        };
        if let Some(change) = change {
          changed.push((path.clone(), change));
        }
        if metadata.is_dir() {
          pending.push((path, ancestry.clone(), false, scan.clone()));
        }
      }
    }
    changed.retain(|(_, change)| change.pending.has_subscribers());
    changed
  }

  fn observe_member(
    &self,
    path: &InternedPath,
    metadata: &std::fs::Metadata,
    symlink: bool,
    scan: Option<&Scan>,
    origin: ContextObservation,
  ) -> Option<(bool, Option<ContextChange>)> {
    let directory = match origin {
      ContextObservation::Scan { .. } => path.parent(),
      _ => None,
    };
    self.observe_in_directory(path, scan, directory, |state| {
      if scan.is_none() && !self.is_below_context(path) {
        return ((false, None), Mutation::Entry);
      }
      let directory = metadata.is_dir();
      // An initial scan can follow a directory symlink whose live event records the link as a file.
      let replaced = !symlink
        && self
          .watchers
          .has_member(path, |old| if directory { old.file } else { old.directory });
      if replaced {
        // A registered file's baseline has already advanced for this event.
        let keep_file = matches!(origin, ContextObservation::Reported)
          && (self.files.all.contains(path) || self.missing.all.contains(path));
        if directory {
          self.remove_file_time(path, keep_file);
        } else {
          self.remove_file_times_under(path, keep_file);
        }
      }
      let mtime = metadata.modified().or_else(|_| metadata.created()).ok();
      let updated = self.record(path, || {
        if directory {
          if let Some(mtime) = mtime {
            mtime_accuracy(mtime);
          }
          return self.watchers.add_member(path, MemberKind::Directory);
        }
        self.watchers.add_member(path, MemberKind::File);
        matches!(origin, ContextObservation::Reported)
          || mtime.is_none_or(|mtime| self.file_times.record(path).observe_member(mtime, &origin))
      });
      let change = match (&origin, scan) {
        (
          ContextObservation::Scan {
            changed_since: Some(since),
            ..
          },
          Some(scan),
        ) => {
          let changed_at = if directory {
            metadata.created().ok().or(mtime)
          } else if self.files.all.contains(path) || self.missing.all.contains(path) {
            None
          } else {
            mtime
          };
          changed_at
            .filter(|time| mtime_safe_time(*time) >= *since)
            .and_then(|_| {
              self
                .watchers
                .queue_context_change(path, directory, scan, state)
            })
        }
        _ => None,
      };
      (
        (updated, change),
        if !directory && !updated && !replaced {
          Mutation::Unchanged
        } else if replaced {
          Mutation::Subtree
        } else {
          Mutation::Entry
        },
      )
    })
  }

  /// An event named `path` below a registered context: index it the way the
  /// scan would have (watchpack's `DirectoryWatcher.setFileTime` /
  /// `setDirectory` on a watch event). A file's record becomes the live
  /// observation; returns false, so the event is dropped, when its mtime is
  /// unchanged and older than the filesystem accuracy, like watchpack: only
  /// its attributes changed (an xattr, a `chmod`). A directory is scanned,
  /// since one moved in arrives as a single event with no events for what it
  /// already contains. Registered contexts absent until now that came back at
  /// or below `path` are rescanned, even when only an ancestor had the event.
  /// An event already `judged` a change — a file the scan found changed since
  /// the start time, or a registered path past its baseline check — skips the
  /// attribute-only check and keeps the record already written for it.
  pub async fn set_context_entry(
    &self,
    path: &InternedPath,
    judged: bool,
    scan: Option<&Scan>,
  ) -> bool {
    let mut recovered: Vec<_> = self
      .absent_directories
      .iter()
      .filter(|context| context.starts_with(path.as_ref() as &Path))
      .map(|context| context.clone())
      .collect();
    recovered.retain(|context| context.is_dir());
    if !recovered.is_empty() {
      for context in &recovered {
        self.absent_directories.remove(context);
      }
      self.scan_below(recovered, true, None).await;
    }

    let registered_directory = self.watchers.is_explicit_context(path);
    if !registered_directory && !self.is_below_context(path) {
      return true;
    }
    // Like watchpack's live events, `lstat`: a link is a file entry here even
    // with `followSymlinks`; only the scan descends into links.
    let Ok(metadata) = std::fs::symlink_metadata(path) else {
      return true;
    };
    if metadata.is_dir()
      && let Ok(mtime) = metadata.modified()
    {
      mtime_accuracy(mtime);
    }
    if registered_directory {
      if !metadata.is_file() {
        return true;
      }
      if self
        .observe(path, scan, || {
          if !self.absent_directories.contains(path) {
            let keep_file =
              judged && (self.files.all.contains(path) || self.missing.all.contains(path));
            self.remove_file_times_under(path, keep_file);
          }
          ((), Mutation::Subtree)
        })
        .is_none()
      {
        return false;
      }
      if !self.is_below_context(path) {
        return true;
      }
    }
    let Some((changed, _)) = self.observe_member(
      path,
      &metadata,
      metadata.is_symlink(),
      scan,
      if judged {
        ContextObservation::Reported
      } else {
        ContextObservation::Event
      },
    ) else {
      return false;
    };
    // A known directory can emit an attribute-only event; only a new one needs a scan.
    if metadata.is_dir() && changed {
      self.scan_below(vec![path.clone()], false, None).await;
    }
    metadata.is_dir() || changed
  }

  /// A scanned entry's metadata. `real_dir`, the real path of the directory
  /// being scanned, is known only with `followSymlinks`; a link is then
  /// followed like watchpack's scan does, except a dangling link, one into its
  /// own directory, or one back above itself: those are recorded as
  /// themselves (`lstat`).
  fn scan_metadata(
    real_dir: Option<&PathBuf>,
    path: &InternedPath,
  ) -> std::io::Result<(std::fs::Metadata, bool)> {
    let own = std::fs::symlink_metadata(path)?;
    let symlink = own.is_symlink();
    let Some(real_dir) = real_dir.filter(|_| own.file_type().is_symlink()) else {
      return Ok((own, symlink));
    };
    let Ok(target) = std::fs::canonicalize(path) else {
      return Ok((own, symlink));
    };
    let into_own_directory = target.parent() == Some(real_dir.as_path());
    let above_itself = path
      .file_name()
      .is_some_and(|name| real_dir.join(name).starts_with(&target));
    if into_own_directory || above_itself {
      return Ok((own, symlink));
    }
    // Like watchpack, only a link to a directory takes the target's metadata;
    // a link to a file keeps its own mtime.
    match std::fs::metadata(&target) {
      Ok(metadata) if metadata.is_dir() => Ok((metadata, symlink)),
      _ => Ok((own, symlink)),
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

  /// watchpack's `setFileTime(filePath, mtime, initial, ignoreWhenEqual)`:
  /// record `path` as seen with `mtime` — from a scan (`initial`, safe time
  /// derived from the mtime) or from a live event (safe time = now). With
  /// `ignore_when_equal`, a record already carrying this mtime is kept as is.
  /// Returns whether a record was written.
  pub fn set_file_time(
    &self,
    path: &InternedPath,
    mtime: SystemTime,
    initial: bool,
    ignore_when_equal: bool,
  ) -> bool {
    self.record(path, || {
      self
        .file_times
        .record(path)
        .set_time(mtime, initial, ignore_when_equal)
    })
  }

  /// An event reached `path` or something below it: advance the
  /// `lastWatchEvent` of `path` and of every registered or discovered
  /// context above it.
  pub fn set_last_watch_events(&self, path: &InternedPath) {
    let now = current_time();
    for ancestor in path.ancestors() {
      let ancestor = InternedPath::from(ancestor);
      if self.watchers.is_watched_directory(&ancestor) {
        self.last_watch_events.insert(ancestor, now);
      }
    }
  }

  /// Check whether a watched path's mtime can represent a change,
  /// advancing the baseline when it does. Covers registered files and
  /// registered-missing paths (on disk by the time an event names them);
  /// other paths, files found below a context included, pass through
  /// unfiltered.
  /// Equal mtimes are only stale once they are past the filesystem accuracy window.
  #[cfg(test)]
  pub fn has_mtime_changed(&self, path: &InternedPath) -> bool {
    self.observe_mtime_change(path, None)
  }

  pub(super) fn observe_mtime_change(&self, path: &InternedPath, scan: Option<&Scan>) -> bool {
    if !self.files.all.contains(path) && !self.missing.all.contains(path) {
      return true;
    }

    let Some(current_mtime) = disk_mtime(path) else {
      return true;
    };
    let mtime_ms = mtime_to_millis(current_mtime);
    let present = ReportedBaseline::Present(current_mtime);
    let aged = mtime_ms.saturating_add(mtime_accuracy(current_mtime)) < current_time();
    self
      .observe(path, scan, || {
        if aged && self.file_times.reported(path) == present {
          return (false, Mutation::Unchanged);
        }
        self.record(path, || {
          let mut record = self.file_times.record(path);
          record.reported = present;
          record.time = Some(FileTime::observed(current_mtime));
        });
        (true, Mutation::Entry)
      })
      .unwrap_or(false)
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
      self.retain_registered_ancestors(path);
    }
    for path in &unregistered {
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
        self.last_watch_events.remove(dir);
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
        self.last_watch_events.remove(path);
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

pub(super) struct RegisteredTimeInfo {
  file_paths: InternedPathIndexSet,
  dir_paths: InternedPathIndexSet,
  file_timestamps: TimeInfoEntries,
  file_safe_times: Vec<(InternedPath, u64)>,
}

fn path_to_string(path: &Path) -> String {
  path
    .to_str()
    .map_or_else(|| path.to_string_lossy().into_owned(), str::to_owned)
}

impl PathManager {
  /// A file's `file_times` record (no syscall), falling back to a fresh stat
  /// for a registered file with no record yet (e.g. one whose record was
  /// dropped when it was removed).
  fn file_time(&self, path: &InternedPath) -> Option<FileTime> {
    self
      .file_times
      .get(path)
      .or_else(|| disk_mtime(path).map(FileTime::initial))
  }

  fn entry(time: FileTime) -> TimeInfoEntry {
    let time = time.narrowed();
    TimeInfoEntry::Entry {
      safe_time: time.safe_time,
      timestamp: mtime_to_millis(time.mtime),
      accuracy: time.accuracy,
    }
  }

  /// watchpack's `collectTimeInfoEntries(fileTimestamps, directoryTimestamps)`
  /// over every registered path, returned as `(fileTimestamps,
  /// directoryTimestamps)`.
  ///
  /// Files — registered, or found below a context — reuse their `file_times`
  /// record. A registered-missing path reads
  /// as `null` until an event (or the scan's backfill) has recorded it on
  /// disk — deliberately no stat fallback: the missing set is every resolver
  /// miss, and stat'ing it on each aggregate would be a syscall storm.
  ///
  /// A directory that exists gets, like a nested-watching `DirectoryWatcher`,
  /// an `ExistenceOnlyTimeEntry` in `fileTimestamps` and an
  /// `OnlySafeTimeEntry` in `directoryTimestamps` whose safe time is the max
  /// of its `lastWatchEvent` and the safe times of every file beneath it.
  /// Absent on disk reads as `null` in both. Only a context known to be
  /// absent is stat'ed, to see whether it came back.
  pub fn collect_time_info_entries(&self) -> (TimeInfoEntries, TimeInfoEntries) {
    self.collect_context_time_info(self.collect_registered_time_info())
  }

  pub(super) fn collect_registered_time_info(&self) -> RegisteredTimeInfo {
    let accessor = self.access();
    let file_paths: InternedPathIndexSet = accessor.files().0.iter().map(|p| p.clone()).collect();
    let mut file_timestamps = TimeInfoEntries::with_capacity(file_paths.len());
    let mut file_safe_times: Vec<(InternedPath, u64)> = Vec::with_capacity(file_paths.len());
    for path in &file_paths {
      let entry = self
        .file_time(path)
        .map_or(TimeInfoEntry::Null, Self::entry);
      if let TimeInfoEntry::Entry { safe_time, .. } = entry {
        file_safe_times.push((path.clone(), safe_time));
      }
      file_timestamps.push((path_to_string(path.as_path()), entry));
    }
    // webpack registers `files` and `missing` as disjoint sets, so a null
    // missing entry never clobbers a real file entry sharing the same key.
    for path in accessor.missing().0.iter() {
      let entry = self
        .file_times
        .get(&path)
        .map_or(TimeInfoEntry::Null, Self::entry);
      if let TimeInfoEntry::Entry { safe_time, .. } = entry {
        file_safe_times.push((path.clone(), safe_time));
      }
      file_timestamps.push((path_to_string(path.as_path()), entry));
    }
    RegisteredTimeInfo {
      file_paths,
      dir_paths: accessor.directories().0.iter().map(|p| p.clone()).collect(),
      file_timestamps,
      file_safe_times,
    }
  }

  pub(super) fn collect_context_time_info(
    &self,
    registered: RegisteredTimeInfo,
  ) -> (TimeInfoEntries, TimeInfoEntries) {
    let RegisteredTimeInfo {
      file_paths,
      mut dir_paths,
      mut file_timestamps,
      mut file_safe_times,
    } = registered;
    for watcher in &self.watchers.directories {
      for (path, member) in &watcher.members {
        if member.directory {
          dir_paths.insert(path.clone());
        }
        if member.file
          && !file_paths.contains(path)
          && let Some(time) = self.file_times.get(path)
        {
          file_safe_times.push((path.clone(), time.narrowed().safe_time));
          file_timestamps.push((path_to_string(path.as_path()), Self::entry(time)));
        }
      }
    }
    let mut dir_safe_times: FxHashMap<InternedPath, Option<u64>> =
      FxHashMap::with_capacity_and_hasher(dir_paths.len(), Default::default());
    for dir in &dir_paths {
      let last_watch_event = self.last_watch_events.get(dir).map(|time| *time);
      // An absent registered context reads null, where watchpack reports `{}`
      // or `{ safeTime: 0 }`: webpack then sees it as missing and rebuilds
      // once it appears, instead of trusting a time for a directory not there.
      let safe_time = if self.absent_directories.contains(dir) {
        disk_mtime(dir)
          .map(mtime_safe_time)
          .map(|own| last_watch_event.map_or(own, |event| own.max(event)))
      } else {
        Some(last_watch_event.unwrap_or_default())
      };
      dir_safe_times.insert(dir.clone(), safe_time);
    }
    // Raise each ancestor directory that exists by its descendant files' safe
    // times; a record cannot resurrect a directory gone from disk. Files are
    // folded into their parent first, so each ancestor chain is walked once.
    if !dir_safe_times.is_empty() {
      let mut parent_indices: FxHashMap<&OsStr, usize> = FxHashMap::default();
      let mut raw_parent_safe_times: Vec<(&Path, u64)> = Vec::new();
      for (file, safe_time) in &file_safe_times {
        if let Some(parent) = file.parent() {
          let index = *parent_indices.entry(parent.as_os_str()).or_insert_with(|| {
            raw_parent_safe_times.push((parent, 0));
            raw_parent_safe_times.len() - 1
          });
          let latest = &mut raw_parent_safe_times[index].1;
          *latest = (*latest).max(*safe_time);
        }
      }
      // Preserve the first spelling of Path-equal parents when merging the raw groups.
      let mut parent_safe_times: FxHashMap<&Path, u64> = FxHashMap::default();
      for (parent, safe_time) in raw_parent_safe_times {
        let latest = parent_safe_times.entry(parent).or_default();
        *latest = (*latest).max(safe_time);
      }
      for (parent, safe_time) in parent_safe_times {
        for dir in parent.ancestors() {
          if let Some(Some(current)) = dir_safe_times.get_mut(&InternedPath::from(dir)) {
            *current = (*current).max(safe_time);
          }
        }
      }
    }
    let mut directory_timestamps = TimeInfoEntries::with_capacity(dir_paths.len());
    for dir in &dir_paths {
      let path = path_to_string(dir.as_path());
      match dir_safe_times.get(dir).copied().flatten() {
        Some(safe_time) => {
          file_timestamps.push((path.clone(), TimeInfoEntry::ExistenceOnlyTimeEntry));
          directory_timestamps.push((path, TimeInfoEntry::OnlySafeTimeEntry { safe_time }));
        }
        None => {
          file_timestamps.push((path.clone(), TimeInfoEntry::Null));
          directory_timestamps.push((path, TimeInfoEntry::Null));
        }
      }
    }

    (file_timestamps, directory_timestamps)
  }
}

pub(crate) fn disk_mtime(path: &InternedPath) -> Option<SystemTime> {
  path
    .metadata()
    .and_then(|m| m.modified().or_else(|_| m.created()))
    .ok()
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
