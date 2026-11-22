use std::{
  ffi::OsStr,
  fmt::Debug,
  future::Future,
  ops::Deref,
  path::{Path, PathBuf},
  pin::Pin,
  sync::Mutex,
  time::SystemTime,
};

use dashmap::setref::multiple::RefMulti;
use rspack_error::Result;
use rspack_paths::{InternedPath, InternedPathDashMap, InternedPathDashSet, InternedPathSet};
use rspack_util::{
  fx_hash::FxHashMap,
  time::{current_time, system_time_to_millis},
};

use super::{FsWatcherIgnored, TimeInfoEntries, TimeInfoEntry, ignored::IgnoredMatcher};
use crate::{
  subscription::{
    LogicalKind, MatchedContext, MatchedTarget, PendingAdvance, ReportedBaseline, SubId,
    SubscriptionGraph, WaitingStepResult,
  },
  time::{fs_accuracy, mtime_accuracy, mtime_safe_time, mtime_to_millis},
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

/// `PathAccessor` provides access to the physical watch target sets for files, directories, and missing paths.
pub(crate) struct PathAccessor<'a> {
  files: &'a PathTracker,
  directories: &'a PathTracker,
  missing: &'a PathTracker,
}

impl<'a> PathAccessor<'a> {
  /// Creates a new `PathAccessor` with references to the physical watch target sets.
  fn new(path_manager: &'a PathManager) -> Self {
    Self {
      files: &path_manager.watch_files,
      directories: &path_manager.watch_directories,
      missing: &path_manager.watch_missing,
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
#[derive(Debug)]
struct PathUpdater {
  pub added: Vec<InternedPath>,
  pub removed: Vec<InternedPath>,
  base_dir: PathBuf,
}

impl PathUpdater {
  fn new((added, removed): (Vec<InternedPath>, Vec<InternedPath>)) -> Self {
    Self {
      added,
      removed,
      base_dir: std::env::current_dir().unwrap_or_default(),
    }
  }
}

#[cfg(test)]
impl<Added, Removed> From<(Added, Removed)> for PathUpdater
where
  Added: Iterator<Item = InternedPath>,
  Removed: Iterator<Item = InternedPath>,
{
  fn from((added, removed): (Added, Removed)) -> Self {
    Self::new((added.collect::<Vec<_>>(), removed.collect::<Vec<_>>()))
  }
}

impl PathUpdater {
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

/// `PathManager` coordinates logical compiler dependencies, the per-directory
/// [`SubscriptionGraph`], and the lock-free time-info tables read from JS.
#[derive(Default)]
pub(crate) struct PathManager {
  files: PathTracker,
  directories: PathTracker,
  missing: PathTracker,
  watch_files: PathTracker,
  watch_directories: PathTracker,
  watch_missing: PathTracker,
  ignored: IgnoredMatcher,
  /// watchpack's `files` map: the last observation of each registered file,
  /// each registered-missing path found on disk, and each file found below a
  /// context. It feeds `collect_time_info_entries` without a syscall; stale
  /// FSEvents are filtered by `SubscriptionGraph::baselines` instead.
  /// See: https://gist.github.com/stormslowly/ed758500de6f23211fd63b39eba5ed07
  file_times: InternedPathDashMap<FileTime>,
  /// watchpack's per-`DirectoryWatcher` `lastWatchEvent`: when an event last
  /// reached each registered or discovered context (its own or a
  /// descendant's), in epoch millis. Like watchpack it starts at 0 (absent)
  /// and only events advance it; a context's safe time is the max of it and
  /// the safe times of the files below.
  last_watch_events: InternedPathDashMap<u64>,
  /// watchpack's `DirectoryWatcher.files`: every file found below a registered
  /// context by its initial scan or reported there by an event. Their records
  /// live in `file_times`; this set is what `collect_time_info_entries` walks.
  context_files: InternedPathDashSet,
  /// watchpack's nested `DirectoryWatcher.directories`: every subdirectory
  /// found below a registered context.
  context_directories: InternedPathDashSet,
  /// Every path holding a record (in `file_times`, the context sets or
  /// `last_watch_events`) under its parent directory, ancestors included up
  /// to the first one already indexed: a removed directory's records are
  /// found by walking its subtree, not by scanning every map. A directory
  /// left with no children and no record of its own is dropped.
  recorded_children: InternedPathDashMap<InternedPathSet>,
  /// Serializes changes to `recorded_children`'s links: a walk taking a
  /// directory out on one thread must not interleave with a walk linking it
  /// in on another, or a linked subtree is left cut off from its parent.
  index_lock: Mutex<()>,
  /// Registered contexts known to be absent from disk: absent when registered,
  /// or removed since. The only directories `collect_time_info_entries`
  /// stats, to see one that came back without an event of its own.
  absent_directories: InternedPathDashSet,
  /// Per-directory forward subscription graph and per-dependency reported baselines.
  graph: Mutex<SubscriptionGraph>,
  /// The `followSymlinks` watch option: the context scan descends into
  /// symlinked directories, as watchpack does.
  follow_symlinks: bool,
}

impl PathManager {
  /// Create a new `PathManager` with an optional ignored paths filter.
  pub fn new(ignored: FsWatcherIgnored) -> Self {
    Self {
      ignored: IgnoredMatcher::new(ignored),
      ..Default::default()
    }
  }

  pub fn with_follow_symlinks(mut self, follow_symlinks: bool) -> Self {
    self.follow_symlinks = follow_symlinks;
    self
  }

  #[cfg(feature = "testing")]
  pub fn bind_custom_target(&self, logical_path: InternedPath, target_path: InternedPath) {
    let base_dir = std::env::current_dir().unwrap_or_default();
    let logical = if logical_path.is_absolute() {
      logical_path
    } else {
      InternedPath::from(base_dir.join(logical_path.as_ref()))
    };
    let target = if target_path.is_absolute() {
      target_path
    } else {
      InternedPath::from(base_dir.join(target_path.as_ref()))
    };
    self.lock_graph().set_custom_target(logical, target);
  }

  fn watch_tracker(&self, kind: LogicalKind) -> &PathTracker {
    match kind {
      LogicalKind::File => &self.watch_files,
      LogicalKind::Directory => &self.watch_directories,
      LogicalKind::Missing => &self.watch_missing,
    }
  }

  fn retain_watch_target(
    &self,
    graph: &mut SubscriptionGraph,
    kind: LogicalKind,
    path: InternedPath,
  ) {
    if graph.retain_watch_ref(kind, path.clone()) {
      let tracker = self.watch_tracker(kind);
      tracker.removed.remove(&path);
      tracker.added.insert(path.clone());
      tracker.all.insert(path);
    }
  }

  fn release_watch_target(
    &self,
    graph: &mut SubscriptionGraph,
    kind: LogicalKind,
    path: &InternedPath,
  ) {
    if graph.release_watch_ref(kind, path) {
      let tracker = self.watch_tracker(kind);
      tracker.added.remove(path);
      tracker.all.remove(path);
      tracker.removed.insert(path.clone());
    }
  }

  /// Reset the per-`watch()`-call diff state (added / removed sets) without
  /// touching the long-lived mtime baselines.
  ///
  /// `file_times` and `SubscriptionGraph::baselines` are intentionally NOT
  /// cleared here: the mtime baselines are tied to the lifetime of the
  /// registered files, not to the lifetime of a single `FsWatcher::watch()`
  /// invocation. Each call to `watch()` from rspack's rebuild cycle
  /// (aggregate → pause → rebuild → rewatch) must NOT re-snapshot the baseline,
  /// otherwise the snapshot can capture a post-user-write mtime and then
  /// `check_and_advance_target_baseline` silently suppresses the very FSEvent
  /// that the user is waiting for. Stale entries for files that have actually
  /// been unregistered are pruned by `update_vecs()`.
  pub fn reset(&self) {
    self.files.reset();
    self.directories.reset();
    self.missing.reset();
    self.watch_files.reset();
    self.watch_directories.reset();
    self.watch_missing.reset();
  }

  pub fn target_path_of(&self, kind: LogicalKind, logical_path: &InternedPath) -> InternedPath {
    let graph = self.lock_graph();
    graph
      .target_of(kind, logical_path)
      .unwrap_or_else(|| graph.resolve_target(logical_path))
  }

  pub fn added_files(&self) -> Vec<InternedPath> {
    self.files.added.iter().map(|p| p.clone()).collect()
  }

  pub fn take_added_contexts(&self) -> Vec<InternedPath> {
    self.directories.added.iter().map(|p| p.clone()).collect()
  }

  pub fn added_for_scan(
    &self,
  ) -> Option<(Vec<InternedPath>, Vec<InternedPath>, Vec<InternedPath>)> {
    if self.files.added.is_empty()
      && self.directories.added.is_empty()
      && self.missing.added.is_empty()
    {
      return None;
    }
    Some((
      self.files.added.iter().map(|p| p.clone()).collect(),
      self.directories.added.iter().map(|p| p.clone()).collect(),
      self.missing.added.iter().map(|p| p.clone()).collect(),
    ))
  }

  pub fn is_registered_missing(&self, path: &InternedPath) -> bool {
    self.missing.all.contains(path)
  }

  /// Record an initial baseline mtime for `path`, but only if no present
  /// baseline already exists. A reported absence counts as no baseline because
  /// the file is back. This is the "incremental" form used by
  /// `FsWatcher::wait_for_event`: on the first `watch()` call we record
  /// mtimes for all newly-registered files; on subsequent `watch()` calls
  /// we MUST skip files that already have a baseline, otherwise we risk
  /// snapshotting a mtime that the user has just bumped via `writeFile`.
  ///
  /// Use `check_and_advance_target_baseline` (which atomically reads-and-updates
  /// the baseline) to advance the recorded mtime in response to real events.
  pub fn set_file_time_if_absent(&self, path: InternedPath, mtime: SystemTime) {
    {
      let mut graph = self.lock_graph();
      let entry = graph
        .baselines
        .entry(path.clone())
        .or_insert(ReportedBaseline::Unreported);
      if *entry == ReportedBaseline::Unreported || entry.is_absent() {
        *entry = ReportedBaseline::Present(mtime);
      }
    }
    self.record(&path, || {
      self
        .file_times
        .entry(path.clone())
        .or_insert_with(|| FileTime::initial(mtime));
    });
  }

  /// The scanner reported `path` as changed: the build will know this mtime,
  /// so it becomes the deduplication baseline as well as the time info.
  pub fn set_reported_file_time(&self, path: &InternedPath, mtime: SystemTime) {
    {
      let mut graph = self.lock_graph();
      graph
        .baselines
        .insert(path.clone(), ReportedBaseline::Present(mtime));
    }
    self.set_file_time(path, mtime, true, true);
  }

  pub fn commit_recovered_entry(
    &self,
    owner_path: &InternedPath,
    _target_path: &InternedPath,
    mtime: Option<SystemTime>,
  ) {
    if let Some(m) = mtime {
      {
        let mut graph = self.lock_graph();
        graph
          .baselines
          .insert(owner_path.clone(), ReportedBaseline::Present(m));
      }
      self.set_file_time(owner_path, m, false, false);
    } else {
      let mut graph = self.lock_graph();
      if matches!(graph.baselines.get(owner_path), Some(b) if b.is_absent()) {
        graph
          .baselines
          .insert(owner_path.clone(), ReportedBaseline::Unreported);
      }
    }
  }

  pub fn mark_initial_missing_absent(&self, path: &InternedPath) {
    let mut graph = self.lock_graph();
    let entry = graph
      .baselines
      .entry(path.clone())
      .or_insert(ReportedBaseline::InitialMissingAbsent);
    if *entry == ReportedBaseline::Unreported {
      *entry = ReportedBaseline::InitialMissingAbsent;
    }
  }

  pub fn record_initial_missing_present(
    &self,
    path: &InternedPath,
    _target: &InternedPath,
    mtime: SystemTime,
  ) {
    {
      let mut graph = self.lock_graph();
      if graph.baselines.get(path) == Some(&ReportedBaseline::ReportedRemoved) {
        graph
          .baselines
          .insert(path.clone(), ReportedBaseline::Unreported);
      }
    }
    self.set_file_time(path, mtime, true, true);
  }

  /// Whether a removal was already reported for this path. The scanner must
  /// still report an absence with no prior record.
  pub fn is_known_absent(&self, path: &InternedPath) -> bool {
    self.lock_graph().baselines.get(path) == Some(&ReportedBaseline::ReportedRemoved)
  }

  /// Record a removal the caller is about to deliver. The scanner relies on
  /// this marker to mean the build was told about the absence.
  pub fn set_reported_file_removal(&self, path: &InternedPath) {
    let registered = self.files.all.contains(path) || self.missing.all.contains(path);
    let mut graph = self.lock_graph();
    let target = graph
      .target_of(LogicalKind::File, path)
      .or_else(|| graph.target_of(LogicalKind::Missing, path))
      .unwrap_or_else(|| path.clone());
    if registered && std::fs::symlink_metadata(&target).is_err() {
      graph
        .baselines
        .insert(path.clone(), ReportedBaseline::ReportedRemoved);
    } else if !registered {
      graph.baselines.remove(path);
    } else if matches!(graph.baselines.get(path), Some(b) if b.is_absent()) {
      graph
        .baselines
        .insert(path.clone(), ReportedBaseline::Unreported);
    }
  }

  fn clear_present_baseline_in_graph(&self, graph: &mut SubscriptionGraph, path: &InternedPath) {
    if matches!(
      graph.baselines.get(path),
      Some(ReportedBaseline::Present(_))
    ) {
      if self.files.all.contains(path) || self.missing.all.contains(path) {
        graph
          .baselines
          .insert(path.clone(), ReportedBaseline::Unreported);
      } else {
        graph.baselines.remove(path);
      }
    }
  }

  /// Drop a removed path's time info and present baseline. A reported absence
  /// survives later removals until the path reappears or is unregistered.
  pub fn remove_file_time(&self, path: &InternedPath) {
    self.file_times.remove(path);
    self.context_files.remove(path);
    let released_ctx_dir = self.context_directories.remove(path).is_some();
    if released_ctx_dir {
      self.last_watch_events.remove(path);
    }
    self.unindex_record(path);
    {
      let mut graph = self.lock_graph();
      self.clear_present_baseline_in_graph(&mut graph, path);
      if released_ctx_dir {
        self.release_watch_target(&mut graph, LogicalKind::Directory, path);
      }
    }
  }

  /// Drop the records of every path inside the directory `dir` (watchpack's
  /// `onDirectoryRemoved` marks them all missing), walking only its subtree.
  #[inline(never)]
  pub fn remove_file_times_under(&self, dir: &InternedPath) {
    let mut cleared_paths = Vec::new();
    let mut released_dirs = Vec::new();
    {
      let _index = self.lock_index();
      let mut pending = vec![dir.clone()];
      while let Some(path) = pending.pop() {
        if let Some((_, children)) = self.recorded_children.remove(&path) {
          pending.extend(children);
        }
        self.file_times.remove(&path);
        self.context_files.remove(&path);
        if self.context_directories.remove(&path).is_some() {
          released_dirs.push(path.clone());
        }
        if !self.directories.all.contains(&path) {
          self.last_watch_events.remove(&path);
        }
        cleared_paths.push(path);
      }
      self.unlink_unrecorded(dir);
    }
    {
      let mut graph = self.lock_graph();
      for path in &cleared_paths {
        self.clear_present_baseline_in_graph(&mut graph, path);
      }
      for d in &released_dirs {
        self.release_watch_target(&mut graph, LogicalKind::Directory, d);
      }
    }
    for context in self.directories.all.iter() {
      if context.starts_with(dir.as_ref() as &Path) {
        self.absent_directories.insert(context.clone());
      }
    }
  }

  pub fn handle_disk_directory_removed(&self, disk_dir: &InternedPath) {
    let (affected_files, affected_contexts) = self.lock_graph().handle_directory_removed(disk_dir);
    for file in affected_files {
      self.remove_file_time(&file);
    }
    for ctx in affected_contexts {
      self.absent_directories.insert(ctx.clone());
      self.remove_file_time(&ctx);
      self.remove_file_times_under(&ctx);
    }
    if self.may_contain_records(disk_dir) {
      self.remove_file_times_under(disk_dir);
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
    (recorded, self.lock_graph().directory_count())
  }

  fn lock_index(&self) -> std::sync::MutexGuard<'_, ()> {
    self.index_lock.lock().expect("record index lock poisoned")
  }

  fn lock_graph(&self) -> std::sync::MutexGuard<'_, SubscriptionGraph> {
    self.graph.lock().expect("subscription graph lock poisoned")
  }

  fn holds_record(&self, path: &InternedPath) -> bool {
    self.file_times.contains_key(path)
      || self.context_files.contains(path)
      || self.context_directories.contains(path)
      || self.last_watch_events.contains_key(path)
  }

  pub fn lookup_disk_event(
    &self,
    disk_path: &InternedPath,
  ) -> (Vec<MatchedTarget>, Vec<MatchedContext>, Vec<PendingAdvance>) {
    self.lock_graph().lookup_disk_event(disk_path)
  }

  pub fn collect_subtree_targets(&self, root_dir: &InternedPath) -> Vec<MatchedTarget> {
    self.lock_graph().collect_subtree_targets(root_dir)
  }

  pub fn commit_waiting_advance(
    &self,
    sub_id: SubId,
    expected_epoch: u64,
    new_dir: InternedPath,
    step: WaitingStepResult,
  ) -> bool {
    let mut graph = self.lock_graph();
    graph.commit_waiting_advance(sub_id, expected_epoch, new_dir, step)
  }

  pub fn is_registration_current(
    &self,
    kind: LogicalKind,
    path: &InternedPath,
    epoch: u64,
  ) -> bool {
    self.lock_graph().is_registration_current(kind, path, epoch)
  }

  pub fn registration_epoch(&self, kind: LogicalKind, path: &InternedPath) -> Option<u64> {
    self.lock_graph().registration_epoch(kind, path)
  }

  pub fn is_registered_file_or_missing(&self, path: &InternedPath) -> bool {
    self.files.all.contains(path) || self.missing.all.contains(path)
  }

  pub fn mark_directory_absent(&self, path: &InternedPath) {
    self.absent_directories.insert(path.clone());
  }

  pub fn clear_directory_absent(&self, path: &InternedPath) -> bool {
    self.absent_directories.remove(path).is_some()
  }

  /// watchpack's initial scan of a `DirectoryWatcher`, for this cycle's newly
  /// registered contexts: index every file and subdirectory below them, so
  /// `collect_time_info_entries` reports them like watchpack does. A path the
  /// live watch already observed with the same mtime keeps its record.
  ///
  /// Returns the files that changed since `start_time` (watchpack's
  /// `checkStartTime`): they changed before the watch was active, so no event
  /// will come for them.
  pub async fn scan_contexts(
    &self,
    contexts: Vec<InternedPath>,
    start_time: SystemTime,
  ) -> Vec<InternedPath> {
    let mut roots = Vec::with_capacity(contexts.len());
    for logical_ctx in contexts {
      if let Some(epoch) = self.registration_epoch(LogicalKind::Directory, &logical_ctx) {
        let target_ctx = self.target_path_of(LogicalKind::Directory, &logical_ctx);
        roots.push((target_ctx, logical_ctx.clone(), logical_ctx, epoch));
      }
    }
    self
      .scan_below(roots, false, Some(system_time_to_millis(start_time)))
      .await
  }

  pub async fn recover_context(&self, disk_dir: InternedPath, root_ctx: InternedPath) {
    if let Some(epoch) = self.registration_epoch(LogicalKind::Directory, &root_ctx) {
      self
        .scan_below(
          vec![(disk_dir, root_ctx.clone(), root_ctx, epoch)],
          true,
          None,
        )
        .await;
    }
  }

  /// Index every file and subdirectory below `roots`, minus what `ignored`
  /// excludes (watchpack's `DirectoryWatcher` initial scan). Roots that are
  /// `recovered` contexts coming back are rescanned like watchpack's
  /// `doScan(false)`: the files directly in them are stamped with the
  /// observation time, since one changed while away may have kept its mtime.
  /// Returns the files whose safe time is at or after `changed_since`.
  fn scan_below(
    &self,
    roots: Vec<(InternedPath, InternedPath, InternedPath, u64)>,
    recovered: bool,
    changed_since: Option<u64>,
  ) -> Pin<Box<dyn Future<Output = Vec<InternedPath>> + Send + '_>> {
    Box::pin(async move {
      let mut changed = Vec::new();
      // Each directory carries the real paths of the directories above it: with
      // `followSymlinks`, a link back into its own ancestry is a loop, while two
      // links to one directory are two aliases, each scanned.
      let mut pending: Vec<(
        InternedPath,
        InternedPath,
        InternedPath,
        u64,
        Vec<PathBuf>,
        bool,
      )> = roots
        .into_iter()
        .map(|(disk_dir, logical_dir, root_ctx, epoch)| {
          (
            disk_dir,
            logical_dir,
            root_ctx,
            epoch,
            Vec::new(),
            recovered,
          )
        })
        .collect();

      while let Some((disk_dir, logical_dir, root_ctx, epoch, mut ancestry, observed)) =
        pending.pop()
      {
        if !self.is_registration_current(LogicalKind::Directory, &root_ctx, epoch) {
          continue;
        }
        if self.follow_symlinks {
          let Ok(real) = std::fs::canonicalize(&disk_dir) else {
            continue;
          };
          if ancestry.contains(&real) {
            continue;
          }
          ancestry.push(real);
        }
        let Ok(entries) = std::fs::read_dir(&disk_dir) else {
          continue;
        };

        for entry_res in entries {
          let Ok(entry) = entry_res else {
            continue;
          };
          let entry_name = entry.file_name();
          let disk_entry_path = InternedPath::from(entry.path());
          let logical_entry_path = InternedPath::from(logical_dir.as_path().join(&entry_name));
          if self.is_ignored_path(logical_entry_path.as_ref()).await {
            continue;
          }
          if !self.is_registration_current(LogicalKind::Directory, &root_ctx, epoch) {
            break;
          }
          let Ok(metadata) = Self::scan_metadata(ancestry.last(), &disk_entry_path) else {
            continue;
          };
          if metadata.is_dir() {
            let newly_added = self.record(&logical_entry_path, || {
              self.context_directories.insert(logical_entry_path.clone())
            });
            if newly_added {
              let mut graph = self.lock_graph();
              self.retain_watch_target(&mut graph, LogicalKind::Directory, disk_entry_path.clone());
            }
            if self.forget_if_orphaned(&logical_entry_path) {
              break;
            }
            if !self.follow_symlinks {
              self.lock_graph().attach_child_context(
                &root_ctx,
                epoch,
                disk_entry_path.clone(),
                logical_entry_path.clone(),
              );
            }
            if let Ok(mtime) = metadata.modified() {
              mtime_accuracy(mtime);
            }
            // watchpack's initial `setDirectory` checks the birthtime.
            if changed_since.is_some_and(|since| {
              metadata
                .created()
                .or_else(|_| metadata.modified())
                .is_ok_and(|birthtime| mtime_safe_time(birthtime) >= since)
            }) {
              changed.push(logical_entry_path.clone());
            }
            pending.push((
              disk_entry_path,
              logical_entry_path,
              root_ctx.clone(),
              epoch,
              ancestry.clone(),
              false,
            ));
          } else if let Ok(mtime) = metadata.modified().or_else(|_| metadata.created()) {
            {
              let mut graph = self.lock_graph();
              if graph.baselines.get(&logical_entry_path)
                == Some(&ReportedBaseline::ReportedRemoved)
              {
                graph
                  .baselines
                  .insert(logical_entry_path.clone(), ReportedBaseline::Unreported);
              }
            }
            self.record(&logical_entry_path, || {
              self.context_files.insert(logical_entry_path.clone());
              self.write_file_time(&logical_entry_path, mtime, !observed, true)
            });
            let registered = self.files.all.contains(&logical_entry_path)
              || self.missing.all.contains(&logical_entry_path);
            if self.forget_if_orphaned(&logical_entry_path) {
              break;
            }
            if !registered && changed_since.is_some_and(|since| mtime_safe_time(mtime) >= since) {
              changed.push(logical_entry_path);
            }
          }
        }
      }
      changed
    })
  }

  /// The scan awaits the `ignored` predicate, so it can outlive the context it
  /// indexes: the next watch cycle may unregister it meanwhile, like watchpack
  /// closing a `DirectoryWatcher`. A record is written first and checked
  /// after, so it either sees the context gone here or is swept by the
  /// unregistering `update_vecs()`.
  fn forget_if_orphaned(&self, path: &InternedPath) -> bool {
    if self.is_below_context(path) {
      return false;
    }
    self.context_files.remove(path);
    let released_dir = self.context_directories.remove(path).is_some();
    if !self.files.all.contains(path) && !self.missing.all.contains(path) {
      self.file_times.remove(path);
    }
    if !self.directories.all.contains(path) {
      self.last_watch_events.remove(path);
    }
    self.unindex_record(path);
    if released_dir {
      let mut graph = self.lock_graph();
      self.release_watch_target(&mut graph, LogicalKind::Directory, path);
    }
    true
  }

  /// An event named `disk_path` below a registered context: index it the way the
  /// scan would have (watchpack's `DirectoryWatcher.setFileTime` /
  /// `setDirectory` on a watch event). A file's record becomes the live
  /// observation; returns false, so the event is dropped, when its mtime is
  /// unchanged and older than the filesystem accuracy, like watchpack: only
  /// its attributes changed (an xattr, a `chmod`). A directory is scanned,
  /// since one moved in arrives as a single event with no events for what it
  /// already contains. Registered contexts absent until now that came back at
  /// or below `disk_path` are rescanned, even when only an ancestor had the event.
  /// An event already `judged` a change — a file the scan found changed since
  /// the start time, or a registered path past its baseline check — skips the
  /// attribute-only check and keeps the record already written for it.
  pub async fn set_context_entry_for_binding(
    &self,
    disk_path: &InternedPath,
    ctx: &MatchedContext,
    judged: bool,
  ) -> bool {
    let recovered: Vec<InternedPath> = self
      .absent_directories
      .iter()
      .filter(|context| {
        context.starts_with(ctx.logical_entry_path.as_ref() as &Path) && context.is_dir()
      })
      .map(|context| context.clone())
      .collect();
    if !recovered.is_empty() {
      for context in &recovered {
        self.absent_directories.remove(context);
        let target = self.target_path_of(LogicalKind::Directory, context);
        {
          let mut graph = self.lock_graph();
          if graph.directories.contains_key(context)
            || graph.registrations[LogicalKind::Directory as usize].contains_key(context)
          {
            graph.register(LogicalKind::Directory, context.clone());
          }
        }
        self.recover_context(target, context.clone()).await;
      }
    }

    // Like watchpack's live events, `lstat`: a link is a file entry here even
    // with `followSymlinks`; only the scan descends into links.
    let Ok(metadata) = std::fs::symlink_metadata(disk_path) else {
      return true;
    };
    if metadata.is_dir()
      && let Ok(mtime) = metadata.modified()
    {
      mtime_accuracy(mtime);
    }
    if ctx.is_root_self || self.directories.all.contains(&ctx.logical_entry_path) {
      return true;
    }
    if metadata.is_dir() {
      self.lock_graph().attach_child_context(
        &ctx.root_context,
        ctx.epoch,
        disk_path.clone(),
        ctx.logical_entry_path.clone(),
      );
      // Like watchpack's `setDirectory`, only a directory not known before is
      // scanned: an event on a known one, e.g. a `chmod`, changes nothing below.
      if self.record(&ctx.logical_entry_path, || {
        self
          .context_directories
          .insert(ctx.logical_entry_path.clone())
      }) {
        {
          let mut graph = self.lock_graph();
          self.retain_watch_target(&mut graph, LogicalKind::Directory, disk_path.clone());
        }
        self
          .scan_below(
            vec![(
              disk_path.clone(),
              ctx.logical_entry_path.clone(),
              ctx.root_context.clone(),
              ctx.epoch,
            )],
            false,
            None,
          )
          .await;
      }
      return true;
    }

    self.record(&ctx.logical_entry_path, || {
      self.context_files.insert(ctx.logical_entry_path.clone())
    });
    let Ok(mtime) = metadata.modified().or_else(|_| metadata.created()) else {
      return true;
    };
    if judged {
      return true;
    }
    let mtime_ms = mtime_to_millis(mtime);
    let accuracy = mtime_accuracy(mtime);
    let untouched = self
      .file_times
      .get(&ctx.logical_entry_path)
      .is_some_and(|old| old.mtime == mtime && mtime_ms + accuracy < current_time());
    if untouched {
      return false;
    }
    self.set_file_time(&ctx.logical_entry_path, mtime, false, false);
    true
  }
}

impl PathManager {
  /// A scanned entry's metadata. `real_dir`, the real path of the directory
  /// being scanned, is known only with `followSymlinks`; a link is then
  /// followed like watchpack's scan does, except a dangling link, one into its
  /// own directory, or one back above itself: those are recorded as
  /// themselves (`lstat`).
  fn scan_metadata(
    real_dir: Option<&PathBuf>,
    path: &InternedPath,
  ) -> std::io::Result<std::fs::Metadata> {
    let own = std::fs::symlink_metadata(path)?;
    let Some(real_dir) = real_dir.filter(|_| own.file_type().is_symlink()) else {
      return Ok(own);
    };
    let Ok(target) = std::fs::canonicalize(path) else {
      return Ok(own);
    };
    let into_own_directory = target.parent() == Some(real_dir.as_path());
    let above_itself = path
      .file_name()
      .is_some_and(|name| real_dir.join(name).starts_with(&target));
    if into_own_directory || above_itself {
      return Ok(own);
    }
    // Like watchpack, only a link to a directory takes the target's metadata;
    // a link to a file keeps its own mtime.
    match std::fs::metadata(&target) {
      Ok(metadata) if metadata.is_dir() => Ok(metadata),
      _ => Ok(own),
    }
  }

  /// Whether removing `path` can take recorded paths below it along: it is a
  /// registered or discovered context, or a directory indexing recorded descendants.
  pub fn may_contain_records(&self, path: &InternedPath) -> bool {
    self.directories.all.contains(path)
      || self.context_directories.contains(path)
      || self.recorded_children.contains_key(path)
  }

  fn is_below_context(&self, path: &InternedPath) -> bool {
    path
      .ancestors()
      .skip(1)
      .any(|ancestor| self.directories.all.contains(&InternedPath::from(ancestor)))
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
    // Seeing the file again clears a reported absence. A time-info-only write
    // must not seed a present baseline and suppress the file's own event.
    {
      let mut graph = self.lock_graph();
      if graph.baselines.get(path) == Some(&ReportedBaseline::ReportedRemoved) {
        graph
          .baselines
          .insert(path.clone(), ReportedBaseline::Unreported);
      }
    }
    self.record(path, || {
      self.write_file_time(path, mtime, initial, ignore_when_equal)
    })
  }

  /// [`Self::set_file_time`]'s write, for a caller already inside
  /// [`Self::record`].
  fn write_file_time(
    &self,
    path: &InternedPath,
    mtime: SystemTime,
    initial: bool,
    ignore_when_equal: bool,
  ) -> bool {
    if ignore_when_equal
      && self
        .file_times
        .get(path)
        .is_some_and(|current| current.mtime == mtime)
    {
      return false;
    }
    let time = if initial {
      FileTime::initial(mtime)
    } else {
      FileTime::observed(mtime)
    };
    self.file_times.insert(path.clone(), time);
    true
  }

  /// An event reached `path` or something below it: advance the
  /// `lastWatchEvent` of `path` and of every registered or discovered
  /// context above it.
  pub fn set_last_watch_events(&self, path: &InternedPath) {
    let now = current_time();
    for ancestor in path.ancestors() {
      let ancestor = InternedPath::from(ancestor);
      if self.directories.all.contains(&ancestor) || self.context_directories.contains(&ancestor) {
        self.last_watch_events.insert(ancestor, now);
      }
    }
  }

  #[cfg(test)]
  pub fn has_mtime_changed(&self, path: &InternedPath) -> bool {
    let target = {
      let graph = self.lock_graph();
      graph
        .target_of(LogicalKind::File, path)
        .or_else(|| graph.target_of(LogicalKind::Missing, path))
        .unwrap_or_else(|| path.clone())
    };
    self.check_and_advance_target_baseline(path, &target)
  }

  /// Check whether a watched path's mtime can represent a change,
  /// advancing the baseline when it does. Covers registered files and
  /// registered-missing paths (on disk by the time an event names them);
  /// other paths, files found below a context included, pass through
  /// unfiltered.
  /// Equal mtimes are only stale once they are past the filesystem accuracy window.
  pub fn check_and_advance_target_baseline(
    &self,
    logical_path: &InternedPath,
    target_path: &InternedPath,
  ) -> bool {
    if !self.files.all.contains(logical_path) && !self.missing.all.contains(logical_path) {
      return true;
    }

    let Some(current_mtime) = disk_mtime(target_path) else {
      return true;
    };
    let mtime_ms = mtime_to_millis(current_mtime);
    let previous = self.lock_graph().baselines.insert(
      logical_path.clone(),
      ReportedBaseline::Present(current_mtime),
    );
    if previous == Some(ReportedBaseline::Present(current_mtime))
      && mtime_ms.saturating_add(mtime_accuracy(current_mtime)) < current_time()
    {
      return false;
    }
    self.set_file_time(logical_path, current_mtime, false, false);
    true
  }

  #[cfg(any(test, feature = "testing"))]
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
    self
      .update_vecs(
        (files.0.collect(), files.1.collect()),
        (directories.0.collect(), directories.1.collect()),
        (missing.0.collect(), missing.1.collect()),
      )
      .await
  }

  /// Update the paths, directories, and missing paths in the `PathManager`.
  #[inline(never)]
  pub async fn update_vecs(
    &self,
    files: (Vec<InternedPath>, Vec<InternedPath>),
    directories: (Vec<InternedPath>, Vec<InternedPath>),
    missing: (Vec<InternedPath>, Vec<InternedPath>),
  ) -> Result<()> {
    let mut diffs: [(Vec<InternedPath>, Vec<InternedPath>); 3] = Default::default();
    for (kind, input, tracker) in [
      (LogicalKind::File, files, &self.files),
      (LogicalKind::Directory, directories, &self.directories),
      (LogicalKind::Missing, missing, &self.missing),
    ] {
      diffs[kind as usize] = PathUpdater::new(input)
        .update(tracker, &self.ignored)
        .await?;
    }

    {
      let mut graph = self.lock_graph();
      for kind in [
        LogicalKind::File,
        LogicalKind::Directory,
        LogicalKind::Missing,
      ] {
        for path in &diffs[kind as usize].1 {
          if let Some(target) = graph.unregister(kind, path) {
            self.release_watch_target(&mut graph, kind, &target);
          }
        }
      }
      for kind in [
        LogicalKind::File,
        LogicalKind::Directory,
        LogicalKind::Missing,
      ] {
        for path in &diffs[kind as usize].0 {
          let target = graph.register(kind, path.clone());
          if kind == LogicalKind::Directory {
            if target.exists() {
              self.absent_directories.remove(path);
            } else {
              self.absent_directories.insert(path.clone());
            }
          }
          self.retain_watch_target(&mut graph, kind, target);
        }
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
    for path in &diffs[LogicalKind::File as usize].1 {
      let still_missing = self.missing.all.contains(path);
      if !self.is_below_context(path) {
        self.file_times.remove(path);
        if !still_missing {
          self.unindex_record(path);
        }
      }
    }
    for path in &diffs[LogicalKind::Missing as usize].1 {
      if self.files.all.contains(path) {
        continue;
      }
      if !self.is_below_context(path) {
        self.file_times.remove(path);
        self.unindex_record(path);
      }
    }
    for dir in &diffs[LogicalKind::Directory as usize].1 {
      // Still below a registered context and on disk: it stays reported, as a
      // directory found there, and keeps its event clock. Absent, it is
      // forgotten, so it is scanned as a new directory when it comes back.
      let absent = self.absent_directories.remove(dir).is_some();
      if !absent && self.is_below_context(dir) {
        if self.record(dir, || self.context_directories.insert(dir.clone())) {
          let mut graph = self.lock_graph();
          self.retain_watch_target(&mut graph, LogicalKind::Directory, dir.clone());
        }
      } else {
        self.last_watch_events.remove(dir);
      }
      self.forget_context_entries_under(dir);
    }

    Ok(())
  }

  /// A context was unregistered: forget what its scan found below it, except
  /// what another still-registered context also covers.
  #[inline(never)]
  fn forget_context_entries_under(&self, dir: &InternedPath) {
    let mut released_dirs = Vec::new();
    {
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
        if self.is_below_context(path) || self.directories.all.contains(path) {
          continue;
        }
        if self.context_files.remove(path).is_some()
          && !self.files.all.contains(path)
          && !self.missing.all.contains(path)
        {
          self.file_times.remove(path);
          self.unlink_unrecorded(path);
        }
        if self.context_directories.remove(path).is_some() {
          released_dirs.push(path.clone());
          self.last_watch_events.remove(path);
          self.unlink_unrecorded(path);
        }
      }
    }
    if !released_dirs.is_empty() {
      let mut graph = self.lock_graph();
      for d in &released_dirs {
        self.release_watch_target(&mut graph, LogicalKind::Directory, d);
      }
    }
  }

  /// Create a new `PathAccessor` to access the current state of physical watch paths, directories, and missing paths.
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

impl PathManager {
  /// A file's `file_times` record (no syscall), falling back to a fresh stat
  /// for a registered file with no record yet (e.g. one whose record was
  /// dropped when it was removed).
  fn file_time(&self, path: &InternedPath) -> Option<FileTime> {
    if let Some(time) = self.file_times.get(path) {
      return Some(*time);
    }
    let target = self.target_path_of(LogicalKind::File, path);
    disk_mtime(&target).map(FileTime::initial)
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
    let path_to_string = |path: &Path| {
      path
        .to_str()
        .map_or_else(|| path.to_string_lossy().into_owned(), str::to_owned)
    };

    let file_paths: Vec<InternedPath> = self.files.all.iter().map(|p| p.clone()).collect();
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
    for path in self.missing.all.iter() {
      let entry = self
        .file_times
        .get(&path)
        .map_or(TimeInfoEntry::Null, |time| Self::entry(*time));
      if let TimeInfoEntry::Entry { safe_time, .. } = entry {
        file_safe_times.push((path.clone(), safe_time));
      }
      file_timestamps.push((path_to_string(path.as_path()), entry));
    }
    // Files found below contexts, like watchpack's `DirectoryWatcher.files`.
    for path in self.context_files.iter() {
      if self.files.all.contains(&path) {
        continue;
      }
      let Some(time) = self.file_times.get(&path).map(|time| *time) else {
        continue;
      };
      file_safe_times.push((path.clone(), time.narrowed().safe_time));
      file_timestamps.push((path_to_string(path.as_path()), Self::entry(time)));
    }

    let mut dir_paths: Vec<InternedPath> = self.directories.all.iter().map(|p| p.clone()).collect();
    dir_paths.extend(
      self
        .context_directories
        .iter()
        .filter(|dir| !self.directories.all.contains(dir))
        .map(|dir| dir.clone()),
    );
    let mut dir_safe_times: FxHashMap<InternedPath, Option<u64>> =
      FxHashMap::with_capacity_and_hasher(dir_paths.len(), Default::default());
    for dir in &dir_paths {
      let last_watch_event = self.last_watch_events.get(dir).map(|time| *time);
      // An absent registered context reads null, where watchpack reports `{}`
      // or `{ safeTime: 0 }`: webpack then sees it as missing and rebuilds
      // once it appears, instead of trusting a time for a directory not there.
      let safe_time = if self.absent_directories.contains(dir) {
        let target = self.target_path_of(LogicalKind::Directory, dir);
        disk_mtime(&target)
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

pub(crate) fn symlink_mtime(path: &InternedPath) -> Option<SystemTime> {
  std::fs::symlink_metadata(path)
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
