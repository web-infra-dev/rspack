use std::{
  fmt::Debug,
  ops::Deref,
  path::{Path, PathBuf},
  time::SystemTime,
};

use dashmap::setref::multiple::RefMulti;
use rspack_error::Result;
use rspack_paths::{InternedPath, InternedPathDashMap, InternedPathDashSet};
use rspack_util::{
  fx_hash::FxHashMap,
  time::{current_time, fs_accuracy, mtime_accuracy, mtime_safe_time, system_time_to_millis},
};

use super::{FsWatcherIgnored, TimeInfoEntries, TimeInfoEntry, ignored::IgnoredMatcher};

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
#[derive(Debug)]
struct PathUpdater {
  pub added: Vec<InternedPath>,
  pub removed: Vec<InternedPath>,
  base_dir: PathBuf,
}

impl<Added, Removed> From<(Added, Removed)> for PathUpdater
where
  Added: Iterator<Item = InternedPath>,
  Removed: Iterator<Item = InternedPath>,
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
  async fn update(self, watch_tracker: &PathTracker, ignored: &IgnoredMatcher) -> Result<()> {
    let added_paths = self.added;
    let removed_paths = self.removed;

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

      watch_tracker.add(added);
    }

    for removed in removed_paths {
      if removed.is_absolute() {
        watch_tracker.remove(removed);
        continue;
      }

      let removed_absolute_path = self.base_dir.join(removed.as_ref());

      watch_tracker.remove(InternedPath::from(removed_absolute_path));
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
  added: InternedPathDashSet,
  removed: InternedPathDashSet,
  all: InternedPathDashSet,
}

impl PathTracker {
  fn reset(&self) {
    self.added.clear();
    self.removed.clear();
  }

  /// Adds a path to the tracker.
  fn add(&self, path: InternedPath) {
    self.added.insert(path.clone());
    self.all.insert(path);
  }

  /// Removes a path from the tracker.
  fn remove(&self, path: InternedPath) {
    self.all.remove(&path);
    self.removed.insert(path);
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
    let mtime_ms = system_time_to_millis(mtime);
    Self {
      mtime,
      safe_time: mtime_safe_time(mtime_ms),
      accuracy: mtime_accuracy(mtime_ms),
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
    Self {
      mtime,
      safe_time: current_time(),
      accuracy: 0,
    }
  }
}

/// `PathManager` is responsible for managing the set of files, directories, and missing paths.
#[derive(Default)]
pub(crate) struct PathManager {
  files: PathTracker,
  directories: PathTracker,
  missing: PathTracker,
  ignored: IgnoredMatcher,
  /// watchpack's `files` map: the last observation of each registered file,
  /// each registered-missing path found on disk, and each file found below a
  /// context. It feeds `collect_time_info_entries` without a syscall; stale
  /// FSEvents are filtered by `baselines` instead.
  /// See: https://gist.github.com/stormslowly/ed758500de6f23211fd63b39eba5ed07
  file_times: InternedPathDashMap<FileTime>,
  /// The mtime the build already knows for each registered file or missing
  /// path — through a delivered event, or because it was on disk when it was
  /// registered. Only this deduplicates a later event carrying the same mtime;
  /// `file_times` is time info only and anything may write it.
  baselines: InternedPathDashMap<SystemTime>,
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
  /// Every directory above a registered path, ever: a removal of one of them
  /// may have taken registered records, or a registered context, with it. Only grows; a
  /// stale entry just costs an unneeded sweep.
  registered_ancestors: InternedPathDashSet,
  /// Every path holding a record (in `file_times`, the context sets or
  /// `last_watch_events`) under its parent directory, ancestors included up
  /// to the first one already indexed: a removed directory's records are
  /// found by walking its subtree, not by scanning every map.
  recorded_children: InternedPathDashMap<InternedPathDashSet>,
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
      file_times: InternedPathDashMap::default(),
      baselines: InternedPathDashMap::default(),
      last_watch_events: InternedPathDashMap::default(),
      context_files: InternedPathDashSet::default(),
      context_directories: InternedPathDashSet::default(),
      registered_ancestors: InternedPathDashSet::default(),
      absent_directories: InternedPathDashSet::default(),
      recorded_children: InternedPathDashMap::default(),
      follow_symlinks: false,
    }
  }

  pub fn with_follow_symlinks(mut self, follow_symlinks: bool) -> Self {
    self.follow_symlinks = follow_symlinks;
    self
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
  pub fn set_file_time_if_absent(&self, path: InternedPath, mtime: SystemTime) {
    self.index_record(&path);
    self.baselines.entry(path.clone()).or_insert(mtime);
    self
      .file_times
      .entry(path)
      .or_insert_with(|| FileTime::initial(mtime));
  }

  /// The scanner reported `path` as changed: the build will know this mtime,
  /// so it becomes the deduplication baseline as well as the time info.
  pub fn set_reported_file_time(&self, path: &InternedPath, mtime: SystemTime) {
    self.baselines.insert(path.clone(), mtime);
    self.set_file_time(path, mtime, true, true);
  }

  /// Drop the record for a path that is no longer being watched (or no longer
  /// on disk), so the map does not grow unboundedly across watch cycles.
  pub fn remove_file_time(&self, path: &InternedPath) {
    self.file_times.remove(path);
    self.baselines.remove(path);
    self.context_files.remove(path);
    if self.context_directories.remove(path).is_some() {
      self.last_watch_events.remove(path);
    }
    self.unindex_record(path);
  }

  /// Drop the records of every path inside the directory `dir` (watchpack's
  /// `onDirectoryRemoved` marks them all missing), walking only its subtree.
  pub fn remove_file_times_under(&self, dir: &InternedPath) {
    let mut pending = vec![dir.clone()];
    while let Some(path) = pending.pop() {
      if let Some((_, children)) = self.recorded_children.remove(&path) {
        pending.extend(children);
      }
      self.file_times.remove(&path);
      self.baselines.remove(&path);
      self.context_files.remove(&path);
      self.context_directories.remove(&path);
      if !self.directories.all.contains(&path) {
        self.last_watch_events.remove(&path);
      }
    }
    self.unindex_record(dir);
    for context in self.directories.all.iter() {
      if context.starts_with(dir.as_ref() as &Path) {
        self.absent_directories.insert(context.clone());
      }
    }
  }

  /// `path` now holds a record: index it, and its ancestors not indexed yet.
  fn index_record(&self, path: &InternedPath) {
    let mut child = path.clone();
    while let Some(parent) = child.parent().map(InternedPath::from) {
      let indexed = !self
        .recorded_children
        .entry(parent.clone())
        .or_default()
        .insert(child);
      if indexed {
        return;
      }
      child = parent;
    }
  }

  /// `path` no longer holds a record of its own.
  fn unindex_record(&self, path: &InternedPath) {
    if let Some(parent) = path.parent().map(InternedPath::from)
      && let Some(siblings) = self.recorded_children.get(&parent)
    {
      siblings.remove(path);
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
  pub async fn scan_contexts(&self, start_time: SystemTime) -> Vec<InternedPath> {
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
  ) -> Vec<InternedPath> {
    let mut changed = Vec::new();
    // Each directory carries the real paths of the directories above it: with
    // `followSymlinks`, a link back into its own ancestry is a loop, while two
    // links to one directory are two aliases, each scanned.
    let mut pending: Vec<(InternedPath, Vec<PathBuf>, bool)> = roots
      .into_iter()
      .map(|root| (root, Vec::new(), recovered))
      .collect();
    while let Some((dir, mut ancestry, observed)) = pending.pop() {
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
        let Ok(metadata) = Self::scan_metadata(ancestry.last(), &path) else {
          continue;
        };
        if metadata.is_dir() {
          self.context_directories.insert(path.clone());
          self.index_record(&path);
          if self.forget_if_orphaned(&path) {
            break;
          }
          pending.push((path, ancestry.clone(), false));
        } else if let Ok(mtime) = metadata.modified().or_else(|_| metadata.created()) {
          self.context_files.insert(path.clone());
          self.set_file_time(&path, mtime, !observed, true);
          let registered = self.files.all.contains(&path) || self.missing.all.contains(&path);
          if self.forget_if_orphaned(&path) {
            break;
          }
          if !registered
            && changed_since
              .is_some_and(|since| mtime_safe_time(system_time_to_millis(mtime)) >= since)
          {
            changed.push(path);
          }
        }
      }
    }
    changed
  }

  /// The scan awaits the `ignored` predicate, so it can outlive the context it
  /// indexes: the next watch cycle may unregister it meanwhile, like watchpack
  /// closing a `DirectoryWatcher`. A record is written first and checked
  /// after, so it either sees the context gone here or is swept by the
  /// unregistering `update()`.
  fn forget_if_orphaned(&self, path: &InternedPath) -> bool {
    if self.is_below_context(path) {
      return false;
    }
    self.context_files.remove(path);
    self.context_directories.remove(path);
    if !self.files.all.contains(path) && !self.missing.all.contains(path) {
      self.file_times.remove(path);
    }
    if !self.directories.all.contains(path) {
      self.last_watch_events.remove(path);
    }
    self.unindex_record(path);
    true
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
  pub async fn set_context_entry(&self, path: &InternedPath, judged: bool) -> bool {
    let recovered: Vec<InternedPath> = self
      .absent_directories
      .iter()
      .filter(|context| context.starts_with(path.as_ref() as &Path) && context.is_dir())
      .map(|context| context.clone())
      .collect();
    if !recovered.is_empty() {
      for context in &recovered {
        self.absent_directories.remove(context);
      }
      self.scan_below(recovered, true, None).await;
    }

    if self.directories.all.contains(path) || !self.is_below_context(path) {
      return true;
    }
    let Ok(metadata) = self.entry_metadata(path) else {
      return true;
    };
    if metadata.is_dir() {
      // Like watchpack's `setDirectory`, only a directory not known before is
      // scanned: an event on a known one, e.g. a `chmod`, changes nothing below.
      if self.context_directories.insert(path.clone()) {
        self.index_record(path);
        self.scan_below(vec![path.clone()], false, None).await;
      }
      return true;
    }
    self.context_files.insert(path.clone());
    self.index_record(path);
    let Ok(mtime) = metadata.modified().or_else(|_| metadata.created()) else {
      return true;
    };
    if judged {
      return true;
    }
    let mtime_ms = system_time_to_millis(mtime);
    let untouched = self.file_times.get(path).is_some_and(|old| {
      old.mtime == mtime && mtime_ms + mtime_accuracy(mtime_ms) < current_time()
    });
    if untouched {
      return false;
    }
    self.set_file_time(path, mtime, false, false);
    true
  }

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
    std::fs::metadata(&target).or(Ok(own))
  }

  /// A path's metadata as the context scan sees it: through a symlink only
  /// with `followSymlinks`, otherwise the link itself (watchpack's `lstat`).
  /// A dangling link is the link itself either way.
  fn entry_metadata(&self, path: &InternedPath) -> std::io::Result<std::fs::Metadata> {
    if self.follow_symlinks {
      std::fs::metadata(path).or_else(|_| std::fs::symlink_metadata(path))
    } else {
      std::fs::symlink_metadata(path)
    }
  }

  /// Whether removing `path` can take recorded paths below it along: it is a
  /// registered or discovered context, or a directory above a registered
  /// path.
  pub fn may_contain_records(&self, path: &InternedPath) -> bool {
    self.directories.all.contains(path)
      || self.context_directories.contains(path)
      || self.registered_ancestors.contains(path)
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
    self.index_record(path);
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

  /// Check whether a watched path's mtime differs from its stored baseline,
  /// advancing the baseline when it does. Covers registered files and
  /// registered-missing paths (on disk by the time an event names them);
  /// other paths, files found below a context included, pass through
  /// unfiltered.
  /// Returns `true` if the event should pass through (mtime changed or no baseline).
  /// Returns `false` if the event should be suppressed (mtime unchanged = stale).
  pub fn has_mtime_changed(&self, path: &InternedPath) -> bool {
    if !self.files.all.contains(path) && !self.missing.all.contains(path) {
      return true;
    }

    let Some(current_mtime) = disk_mtime(path) else {
      return true;
    };
    if self.baselines.insert(path.clone(), current_mtime) == Some(current_mtime) {
      return false;
    }
    self.set_file_time(path, current_mtime, false, false);
    true
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
    PathUpdater::from(files)
      .update(&self.files, &self.ignored)
      .await?;
    PathUpdater::from(directories)
      .update(&self.directories, &self.ignored)
      .await?;
    PathUpdater::from(missing)
      .update(&self.missing, &self.ignored)
      .await?;

    for dir in self.directories.added.iter() {
      if dir.exists() {
        self.absent_directories.remove(&*dir);
      } else {
        self.absent_directories.insert(dir.clone());
      }
    }
    let added: Vec<InternedPath> = self
      .files
      .added
      .iter()
      .chain(self.missing.added.iter())
      .chain(self.directories.added.iter())
      .map(|p| p.clone())
      .collect();
    for path in &added {
      for ancestor in path.ancestors().skip(1) {
        if !self
          .registered_ancestors
          .insert(InternedPath::from(ancestor))
        {
          break;
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
    let removed_files: Vec<InternedPath> = self.files.removed.iter().map(|p| p.clone()).collect();
    for path in &removed_files {
      if !self.missing.all.contains(path) {
        self.baselines.remove(path);
      }
      if !self.is_below_context(path) {
        self.file_times.remove(path);
      }
    }
    let removed_missing: Vec<InternedPath> =
      self.missing.removed.iter().map(|p| p.clone()).collect();
    for path in &removed_missing {
      if self.files.all.contains(path) {
        continue;
      }
      self.baselines.remove(path);
      if !self.is_below_context(path) {
        self.file_times.remove(path);
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
        self.context_directories.insert(dir.clone());
        self.index_record(dir);
      } else {
        self.last_watch_events.remove(dir);
      }
      self.forget_context_entries_under(dir);
    }

    Ok(())
  }

  /// A context was unregistered: forget what its scan found below it, except
  /// what another still-registered context also covers.
  fn forget_context_entries_under(&self, dir: &InternedPath) {
    let mut subtree = Vec::new();
    let mut pending = vec![dir.clone()];
    while let Some(path) = pending.pop() {
      if let Some(children) = self.recorded_children.get(&path) {
        pending.extend(children.iter().map(|child| child.clone()));
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
        self.unindex_record(path);
      }
      if self.context_directories.remove(path).is_some() {
        self.last_watch_events.remove(path);
        self.unindex_record(path);
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

impl PathManager {
  /// A file's `file_times` record (no syscall), falling back to a fresh stat
  /// for a registered file with no record yet (e.g. one whose record was
  /// dropped when it was removed).
  fn file_time(&self, path: &InternedPath) -> Option<FileTime> {
    if let Some(time) = self.file_times.get(path) {
      return Some(*time);
    }
    disk_mtime(path).map(FileTime::initial)
  }

  fn stat_mtime_ms(path: &InternedPath) -> Option<u64> {
    disk_mtime(path).map(system_time_to_millis)
  }

  fn entry(time: FileTime) -> TimeInfoEntry {
    let time = time.narrowed();
    TimeInfoEntry::Entry {
      safe_time: time.safe_time,
      timestamp: system_time_to_millis(time.mtime),
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
    let accessor = self.access();

    let file_paths: Vec<InternedPath> = accessor.files().0.iter().map(|p| p.clone()).collect();
    let mut file_timestamps = TimeInfoEntries::with_capacity(file_paths.len());
    let mut file_safe_times: Vec<(InternedPath, u64)> = Vec::with_capacity(file_paths.len());
    for path in &file_paths {
      let entry = self
        .file_time(path)
        .map_or(TimeInfoEntry::Null, Self::entry);
      if let TimeInfoEntry::Entry { safe_time, .. } = entry {
        file_safe_times.push((path.clone(), safe_time));
      }
      file_timestamps.push((path.to_string_lossy().to_string(), entry));
    }
    // webpack registers `files` and `missing` as disjoint sets, so a null
    // missing entry never clobbers a real file entry sharing the same key.
    for path in accessor.missing().0.iter() {
      let entry = self
        .file_times
        .get(&path)
        .map_or(TimeInfoEntry::Null, |time| Self::entry(*time));
      if let TimeInfoEntry::Entry { safe_time, .. } = entry {
        file_safe_times.push((path.clone(), safe_time));
      }
      file_timestamps.push((path.to_string_lossy().to_string(), entry));
    }
    // Files found below contexts, like watchpack's `DirectoryWatcher.files`.
    for path in self.context_files.iter() {
      if accessor.files().0.contains(&path) {
        continue;
      }
      let Some(time) = self.file_times.get(&path).map(|time| *time) else {
        continue;
      };
      file_safe_times.push((path.clone(), time.narrowed().safe_time));
      file_timestamps.push((path.to_string_lossy().to_string(), Self::entry(time)));
    }

    let mut dir_paths: Vec<InternedPath> =
      accessor.directories().0.iter().map(|p| p.clone()).collect();
    dir_paths.extend(
      self
        .context_directories
        .iter()
        .filter(|dir| !accessor.directories().0.contains(dir))
        .map(|dir| dir.clone()),
    );
    let mut dir_safe_times: FxHashMap<InternedPath, Option<u64>> =
      FxHashMap::with_capacity_and_hasher(dir_paths.len(), Default::default());
    for dir in &dir_paths {
      let last_watch_event = self.last_watch_events.get(dir).map(|time| *time);
      let safe_time = if self.absent_directories.contains(dir) {
        Self::stat_mtime_ms(dir)
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
      let mut parent_safe_times: FxHashMap<&Path, u64> = FxHashMap::default();
      for (file, safe_time) in &file_safe_times {
        if let Some(parent) = file.parent() {
          let latest = parent_safe_times.entry(parent).or_default();
          *latest = (*latest).max(*safe_time);
        }
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
      let path = dir.to_string_lossy().to_string();
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
