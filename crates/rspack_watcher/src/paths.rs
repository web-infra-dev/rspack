mod links;

use std::{
  ffi::OsStr,
  fmt::Debug,
  ops::Deref,
  path::{Path, PathBuf},
  time::SystemTime,
};

use dashmap::setref::multiple::RefMulti;
pub(crate) use links::{
  DepKind, EntrySubscriber, LinkMappings, LinkResolver, LinkUpdate, PhysicalSet, RebindRequest,
};
use rspack_error::Result;
use rspack_paths::{InternedPath, InternedPathDashMap, InternedPathDashSet, InternedPathSet};
use rspack_util::{
  fx_hash::FxHashMap,
  time::{current_time, system_time_to_millis},
};

use super::{FsWatcherIgnored, TimeInfoEntries, TimeInfoEntry, ignored::IgnoredMatcher};
use crate::time::{fs_accuracy, mtime_accuracy, mtime_safe_time, mtime_to_millis};

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

/// The three sets of one [`PathTracker`] or one physical set.
#[derive(Clone, Copy)]
struct SetRefs<'a> {
  all: &'a InternedPathDashSet,
  added: &'a InternedPathDashSet,
  removed: &'a InternedPathDashSet,
}

/// `PathAccessor` provides access to the sets of files, directories, and missing paths.
pub(crate) struct PathAccessor<'a> {
  files: SetRefs<'a>,
  directories: SetRefs<'a>,
  missing: SetRefs<'a>,
}

impl<'a> PathAccessor<'a> {
  /// Creates a new `PathAccessor` with references to the sets of files, directories, and missing paths.
  fn new(files: SetRefs<'a>, directories: SetRefs<'a>, missing: SetRefs<'a>) -> Self {
    Self {
      files,
      directories,
      missing,
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
    (self.files.all, self.files.added, self.files.removed)
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
      self.directories.all,
      self.directories.added,
      self.directories.removed,
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
    (self.missing.all, self.missing.added, self.missing.removed)
  }

  /// Returns an iterator that combines all files, directories, and missing paths into a single sequence.
  pub fn all(&self) -> impl Iterator<Item = InternedPath> + '_ {
    All::new(self.files.all, self.directories.all, self.missing.all)
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

/// A directory's filesystem identity: `(device, inode)` on Unix, the volume
/// serial and file index on Windows. Two observations with different
/// identities are two different directories, however equal their mtimes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct DirectoryIdentity {
  device: u64,
  index: u64,
}

impl DirectoryIdentity {
  fn of(metadata: &std::fs::Metadata) -> Option<Self> {
    #[cfg(unix)]
    {
      use std::os::unix::fs::MetadataExt;
      Some(Self {
        device: metadata.dev(),
        index: metadata.ino(),
      })
    }
    #[cfg(windows)]
    {
      use std::os::windows::fs::MetadataExt;
      Some(Self {
        device: metadata.volume_serial_number()?.into(),
        index: metadata.file_index()?,
      })
    }
    #[cfg(not(any(unix, windows)))]
    {
      let _ = metadata;
      None
    }
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

/// What one observed entry event means for the subscriptions above it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum EntryChange {
  /// Nothing changed — an attribute-only file event, or an event on an existing
  /// directory whose identity is unchanged.
  Unchanged,
  /// The directory still exists and is the same one: only a subscription to the
  /// directory itself needs the event, its parent context does not.
  ExistingDirectory,
  Changed,
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
  /// registered. `None` records a reported removal, distinct from a path not
  /// observed yet. Only this deduplicates a later event carrying the same mtime;
  /// `file_times` is time info only and anything may write it.
  baselines: InternedPathDashMap<Option<SystemTime>>,
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
  /// Every directory found below a registered context, with the filesystem
  /// identity it was first seen with. A directory can be replaced without ever
  /// being observed absent (a rename pair, a `mv` over it), so its identity —
  /// not its mtime — decides whether its recorded children still belong to it.
  context_directories: InternedPathDashSet,
  /// The filesystem identity each context directory was last observed with.
  directory_identities: InternedPathDashMap<Option<DirectoryIdentity>>,
  /// Every directory above a registered path: a removal of one of them may
  /// have taken registered records, or a registered context, with it. Each
  /// counts the registrations directly in it plus its subdirectories here,
  /// and goes once that count is back to zero.
  registered_ancestors: InternedPathDashMap<usize>,
  /// Every path holding a record (in `file_times`, the context sets or
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
  /// Physical paths behind the logical registrations. Only the OS layer needs
  /// them; events are mapped back to logical paths through this table.
  links: std::sync::RwLock<LinkMappings>,
  /// The physical sets handed to the analyzer while any mapping is live.
  physical_files: PhysicalSet,
  physical_directories: PhysicalSet,
  physical_missing: PhysicalSet,
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
      directory_identities: InternedPathDashMap::default(),
      registered_ancestors: InternedPathDashMap::default(),
      absent_directories: InternedPathDashSet::default(),
      recorded_children: InternedPathDashMap::default(),
      index_lock: std::sync::Mutex::default(),
      follow_symlinks: false,
      links: std::sync::RwLock::default(),
      physical_files: PhysicalSet::default(),
      physical_directories: PhysicalSet::default(),
      physical_missing: PhysicalSet::default(),
    }
  }

  pub fn with_follow_symlinks(mut self, follow_symlinks: bool) -> Self {
    self.follow_symlinks = follow_symlinks;
    self.links = std::sync::RwLock::new(LinkMappings::new());
    self
  }

  pub(crate) fn follows_symlinks(&self) -> bool {
    self.follow_symlinks
  }

  fn lock_links(&self) -> std::sync::RwLockWriteGuard<'_, LinkMappings> {
    self
      .links
      .write()
      .unwrap_or_else(std::sync::PoisonError::into_inner)
  }

  fn read_links(&self) -> std::sync::RwLockReadGuard<'_, LinkMappings> {
    self
      .links
      .read()
      .unwrap_or_else(std::sync::PoisonError::into_inner)
  }

  /// The logical subscribers of a physical entry path.
  pub(crate) fn entry_subscribers(
    &self,
    disk_path: &InternedPath,
  ) -> smallvec::SmallVec<[EntrySubscriber; 2]> {
    self.read_links().entry_subscribers(disk_path)
  }

  /// The `(root context, logical path)` pairs a physical path belongs to.
  pub(crate) fn context_subscribers(
    &self,
    disk_path: &InternedPath,
  ) -> smallvec::SmallVec<[(InternedPath, InternedPath, InternedPath); 2]> {
    self.read_links().context_subscribers(disk_path)
  }

  /// The dependencies waiting for a missing component to appear.
  pub(crate) fn pending_waiters(
    &self,
    disk_path: &InternedPath,
  ) -> smallvec::SmallVec<[RebindRequest; 2]> {
    self.read_links().pending_waiters(disk_path)
  }

  /// The logical dependencies whose physical chain includes this link.
  pub(crate) fn link_waiters(&self, link: &InternedPath) -> smallvec::SmallVec<[RebindRequest; 2]> {
    self.read_links().link_waiters(link)
  }

  /// Whether `path` is a link whose state the mappings track.
  pub(crate) fn tracks_link(&self, path: &InternedPath) -> bool {
    self.read_links().tracks_link(path)
  }

  /// Whether `path` is a link whose target is ignored.
  #[allow(dead_code)]
  pub(crate) fn is_unfollowed(&self, path: &InternedPath) -> bool {
    self.read_links().is_unfollowed(path)
  }

  /// Whether any ancestor of `path` is a link whose target is ignored: the
  /// event came from behind that link and must not be reported.
  pub(crate) fn is_behind_unfollowed_link(&self, path: &InternedPath) -> bool {
    let links = self.read_links();
    if links.unfollowed_is_empty() {
      return false;
    }
    path
      .ancestors()
      .skip(1)
      .map(InternedPath::from)
      .any(|ancestor| links.is_unfollowed(&ancestor))
  }

  /// Resolve the physical target of a link a context scan found, and remember
  /// it so events on that target map back to the logical link path.
  pub(crate) async fn bind_context_link(
    &self,
    root_context: &InternedPath,
    logical_path: &InternedPath,
    directory: bool,
  ) -> bool {
    if !self.follow_symlinks || !logical_path.is_symlink() {
      return false;
    }
    let mut resolver = LinkResolver::new(self.follow_symlinks);
    let resolved = if directory {
      resolver.resolve_directory(logical_path)
    } else {
      resolver.resolve_entry(logical_path)
    };
    if resolved.is_identity(logical_path) {
      return false;
    }
    // A link that resolves inside its own context is already covered by that
    // context's watch: subscribing the target again would report it twice.
    if resolved.target.starts_with(root_context.as_ref() as &Path) {
      return false;
    }
    let mut links = self.lock_links();
    let mut files = links.files.clone();
    let mut directories = links.directories.clone();
    links.bind_context_target(
      root_context,
      logical_path,
      &resolved,
      directory,
      &mut files,
      &mut directories,
    );
    links.files = files;
    links.directories = directories;
    true
  }

  /// Rebuild the physical subscription of every logical dependency and hand the
  /// resulting physical sets to the analyzer.
  pub(crate) async fn refresh_links(&self) -> LinkUpdate {
    let files: InternedPathSet = self.files.all.iter().map(|path| path.clone()).collect();
    let directories: InternedPathSet = self
      .directories
      .all
      .iter()
      .map(|path| path.clone())
      .collect();
    let missing: InternedPathSet = self.missing.all.iter().map(|path| path.clone()).collect();
    // A target the caller ignores is not watched: only the link itself is,
    // so redirecting it is still seen while the target stays quiet.
    let mut ignored_targets = InternedPathSet::default();
    let mut unfollowed_links = InternedPathSet::default();
    if self.follow_symlinks {
      let mut resolver = LinkResolver::new(self.follow_symlinks);
      for logical in files.iter().chain(directories.iter()).chain(missing.iter()) {
        let resolved = if directories.contains(logical) {
          resolver.resolve_directory(logical)
        } else {
          resolver.resolve_entry(logical)
        };
        if resolved.target == *logical {
          continue;
        }
        if !ignored_targets.contains(&resolved.target)
          && self.is_ignored_path(resolved.target.as_ref()).await
        {
          ignored_targets.insert(resolved.target.clone());
        }
        if ignored_targets.contains(&resolved.target) {
          unfollowed_links.insert(logical.clone());
        }
        let _ = &resolved.target;
      }
    }
    let mut resolver = LinkResolver::new(self.follow_symlinks);
    let mut links = self.lock_links();
    let update = links.rebuild(
      &mut resolver,
      &files,
      &directories,
      &missing,
      &ignored_targets,
      &unfollowed_links,
    );
    let next_files = links.files.clone();
    let next_directories = links.directories.clone();
    let next_missing = links.missing.clone();
    drop(links);
    self.physical_files.replace(&next_files);
    self.physical_directories.replace(&next_directories);
    self.physical_missing.replace(&next_missing);
    update
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

  /// Record an initial baseline mtime for `path`, but only if no present
  /// baseline already exists. A reported absence (`None`) counts as no
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
      self
        .baselines
        .entry(path.clone())
        .or_default()
        .get_or_insert(mtime);
      self
        .file_times
        .entry(path.clone())
        .or_insert_with(|| FileTime::initial(mtime));
    });
  }

  /// The scanner reported `path` as changed: the build will know this mtime,
  /// so it becomes the deduplication baseline as well as the time info.
  pub fn set_reported_file_time(&self, path: &InternedPath, mtime: SystemTime) {
    self.baselines.insert(path.clone(), Some(mtime));
    self.set_file_time(path, mtime, true, true);
  }

  /// Whether a removal was already reported for this path. The scanner must
  /// still report an absence with no prior record.
  pub fn is_known_absent(&self, path: &InternedPath) -> bool {
    self.baselines.get(path).is_some_and(|time| time.is_none())
  }

  /// Record a removal the caller is about to deliver. The scanner relies on
  /// this marker to mean the build was told about the absence.
  pub fn set_reported_file_removal(&self, path: &InternedPath) {
    self.baselines.insert(path.clone(), None);
    if (!self.files.all.contains(path) && !self.missing.all.contains(path))
      || std::fs::symlink_metadata(path).is_ok()
    {
      self.baselines.remove_if(path, |_, time| time.is_none());
    }
  }

  /// Drop a removed path's time info and present baseline. A reported absence
  /// survives later removals until the path reappears or is unregistered.
  pub fn remove_file_time(&self, path: &InternedPath) {
    self.file_times.remove(path);
    self.baselines.remove_if(path, |_, time| time.is_some());
    self.context_files.remove(path);
    if self.context_directories.remove(path).is_some() {
      self.last_watch_events.remove(path);
      self.directory_identities.remove(path);
    }
    self.unindex_record(path);
  }

  /// Drop the records of every path inside the directory `dir` (watchpack's
  /// `onDirectoryRemoved` marks them all missing), walking only its subtree.
  pub fn remove_file_times_under(&self, dir: &InternedPath) {
    let _index = self.lock_index();
    let mut pending = vec![dir.clone()];
    while let Some(path) = pending.pop() {
      if let Some((_, children)) = self.recorded_children.remove(&path) {
        pending.extend(children);
      }
      self.file_times.remove(&path);
      self.baselines.remove_if(&path, |_, time| time.is_some());
      self.context_files.remove(&path);
      self.context_directories.remove(&path);
      self.directory_identities.remove(&path);
      if !self.directories.all.contains(&path) {
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
    self.file_times.contains_key(path)
      || self.context_files.contains(path)
      || self.context_directories.contains(path)
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
    let mut pending: Vec<(InternedPath, Vec<PathBuf>, bool, InternedPath)> = roots
      .into_iter()
      .map(|root| {
        let context = root.clone();
        (root, Vec::new(), recovered, context)
      })
      .collect();
    while let Some((dir, mut ancestry, observed, root_context)) = pending.pop() {
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
        // A link to somewhere outside the context needs its physical target
        // watched and mapped back to this logical link path.
        if self.follow_symlinks && path.is_symlink() {
          self
            .bind_context_link(&root_context, &path, metadata.is_dir())
            .await;
        }
        if metadata.is_dir() {
          self.record(&path, || {
            self.context_directories.insert(path.clone());
            self
              .directory_identities
              .insert(path.clone(), DirectoryIdentity::of(&metadata));
          });
          if self.forget_if_orphaned(&path) {
            break;
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
            changed.push(path.clone());
          }
          pending.push((path, ancestry.clone(), false, root_context.clone()));
        } else if let Ok(mtime) = metadata.modified().or_else(|_| metadata.created()) {
          // A file behind a link whose target is ignored is not reported: only
          // the link itself is a dependency.
          if self.is_behind_unfollowed_link(&path) {
            continue;
          }
          self.record(&path, || {
            self.context_files.insert(path.clone());
            self.write_file_time(&path, mtime, !observed, true)
          });
          let registered = self.files.all.contains(&path) || self.missing.all.contains(&path);
          if self.forget_if_orphaned(&path) {
            break;
          }
          if !registered && changed_since.is_some_and(|since| mtime_safe_time(mtime) >= since) {
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
  pub async fn set_context_entry(&self, disk_path: &InternedPath, judged: bool) -> EntryChange {
    // A disk event names the physical target of a followed link. Everything
    // recorded here is keyed by the logical path the caller registered, so
    // `collect_time_info_entries` reports the link, not its target.
    let logical = self.logical_entry(disk_path);
    let path = &logical;
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

    let registered_directory = self.directories.all.contains(path);
    if !registered_directory && !self.is_below_context(path) {
      // A registered file reached through a link: record the target's time
      // under the logical path, so its time info follows the file behind it.
      if self.files.all.contains(path)
        && let Ok(target) = std::fs::metadata(path)
        && let Ok(mtime) = target.modified().or_else(|_| target.created())
      {
        self.set_file_time(path, mtime, false, false);
      }
      return EntryChange::Changed;
    }
    // Like watchpack's live events, `lstat`: a link is a file entry here even
    // with `followSymlinks`; only the scan descends into links.
    let Ok(metadata) = std::fs::symlink_metadata(path) else {
      return EntryChange::Changed;
    };
    if metadata.is_dir()
      && let Ok(mtime) = metadata.modified()
    {
      mtime_accuracy(mtime);
    }
    if registered_directory {
      // A registered context reports its own event, but only its parent
      // context needs to hear about it when it was replaced by another
      // directory. An unchanged directory (a `chmod`, a repeated Create for a
      // path already there) leaves everything below it in place.
      let identity = DirectoryIdentity::of(&metadata);
      let previous = self.directory_identities.get(path).map(|old| *old);
      let replaced = match (identity, previous) {
        (Some(now), Some(Some(was))) => now != was,
        // First observation of a registered context: its parent has not been
        // told about this directory yet, so the event is a change.
        (Some(_), None) => true,
        _ => false,
      };
      self.directory_identities.insert(path.clone(), identity);
      return if replaced {
        EntryChange::Changed
      } else {
        EntryChange::ExistingDirectory
      };
    }
    if metadata.is_dir() {
      // Like watchpack's `setDirectory`, only a directory not known before is
      // scanned: an event on a known one, e.g. a `chmod`, changes nothing below.
      // A directory replaced under the same path — a rename pair, a `mv` over
      // it — is a new directory even though the path was known: its old
      // children belong to another directory, so it is scanned again.
      let identity = DirectoryIdentity::of(&metadata);
      let replaced = match (
        identity,
        self.directory_identities.get(path).map(|old| *old),
      ) {
        (Some(now), Some(Some(was))) => now != was,
        _ => false,
      };
      if replaced {
        self.remove_file_times_under(path);
      }
      self.directory_identities.insert(path.clone(), identity);
      let known = self.record(path, || self.context_directories.insert(path.clone()));
      if known || replaced {
        self.scan_below(vec![path.clone()], false, None).await;
      }
      return if known || replaced {
        EntryChange::Changed
      } else {
        // A directory that is still the same one: its own subscription needs
        // the event, its parent context does not — nothing below it moved.
        EntryChange::ExistingDirectory
      };
    }
    self.record(path, || self.context_files.insert(path.clone()));
    // A file reached through a link reports its target's time when the event
    // named the target; an event on the link itself keeps the link's own time,
    // so a newly made link to an old file is still a change.
    let reached_through_link = logical != *disk_path;
    let file_metadata =
      if reached_through_link && self.follow_symlinks && metadata.file_type().is_symlink() {
        std::fs::metadata(path).unwrap_or(metadata)
      } else {
        metadata
      };
    let Ok(mtime) = file_metadata
      .modified()
      .or_else(|_| file_metadata.created())
    else {
      return EntryChange::Changed;
    };
    if judged {
      return EntryChange::Changed;
    }
    let mtime_ms = mtime_to_millis(mtime);
    let accuracy = mtime_accuracy(mtime);
    let untouched = self
      .file_times
      .get(path)
      .is_some_and(|old| old.mtime == mtime && mtime_ms + accuracy < current_time());
    if untouched {
      return EntryChange::Unchanged;
    }
    self.set_file_time(path, mtime, false, false);
    // Every other registration sharing this physical target moved with it.
    for other in self.logical_entries(disk_path) {
      if other != *path {
        self.set_file_time(&other, mtime, false, false);
      }
    }
    EntryChange::Changed
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
    // Like watchpack, only a link to a directory takes the target's metadata;
    // a link to a file keeps its own mtime.
    match std::fs::metadata(&target) {
      Ok(metadata) if metadata.is_dir() => Ok(metadata),
      _ => Ok(own),
    }
  }

  /// Whether removing `path` can take recorded paths below it along: it is a
  /// registered or discovered context, or a directory above a registered
  /// path.
  pub fn may_contain_records(&self, path: &InternedPath) -> bool {
    self.directories.all.contains(path)
      || self.context_directories.contains(path)
      || self.registered_ancestors.contains_key(path)
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
  ///
  /// A physical path reached through a link is recorded under the logical path
  /// the caller registered: `collect_time_info_entries` reports logical paths.
  pub fn set_file_time(
    &self,
    path: &InternedPath,
    mtime: SystemTime,
    initial: bool,
    ignore_when_equal: bool,
  ) -> bool {
    let logical = self.logical_entry(path);
    self.record(&logical, || {
      self.write_file_time(&logical, mtime, initial, ignore_when_equal)
    })
  }

  /// Whether `path` was explicitly registered as a context.
  pub(crate) fn is_registered_directory(&self, path: &InternedPath) -> bool {
    self.directories.all.contains(path)
  }

  /// Every logical registration a physical path serves.
  pub(crate) fn logical_entries(&self, path: &InternedPath) -> Vec<InternedPath> {
    let mut logical: Vec<InternedPath> = self
      .entry_subscribers(path)
      .iter()
      .map(|subscriber| subscriber.logical_path().clone())
      .collect();
    logical.extend(
      self
        .context_subscribers(path)
        .into_iter()
        .map(|(_, logical, _)| logical),
    );
    logical.sort_by(|a, b| a.as_ref().cmp(b.as_ref()));
    logical.dedup();
    logical
  }

  /// The logical registration a physical path serves, or the path itself.
  fn logical_entry(&self, path: &InternedPath) -> InternedPath {
    if self.files.all.contains(path)
      || self.missing.all.contains(path)
      || self.context_files.contains(path)
    {
      return path.clone();
    }
    let mut logical: Vec<InternedPath> = self
      .entry_subscribers(path)
      .iter()
      .map(|subscriber| subscriber.logical_path().clone())
      .collect();
    // A file found below a context link: the physical target maps back to the
    // logical path under the link.
    logical.extend(
      self
        .context_subscribers(path)
        .into_iter()
        .map(|(_, logical, _)| logical),
    );
    logical.sort_by(|a, b| a.as_ref().cmp(b.as_ref()));
    logical.first().cloned().unwrap_or_else(|| path.clone())
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
    // Seeing the file again clears a reported absence. A time-info-only write
    // must not seed a present baseline and suppress the file's own event.
    self.baselines.remove_if(path, |_, time| time.is_none());
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

  /// Check whether a watched path's mtime can represent a change,
  /// advancing the baseline when it does. Covers registered files and
  /// registered-missing paths (on disk by the time an event names them);
  /// other paths, files found below a context included, pass through
  /// unfiltered.
  /// Equal mtimes are only stale once they are past the filesystem accuracy window.
  pub fn has_mtime_changed(&self, path: &InternedPath) -> bool {
    // A disk event names the physical target of a followed link; the baseline
    // belongs to the logical dependency, which is what the build knows.
    let logical = self.entry_subscribers(path).first().map_or_else(
      || path.clone(),
      |subscriber| subscriber.logical_path().clone(),
    );
    let logical = &logical;
    if !self.files.all.contains(logical) && !self.missing.all.contains(logical) {
      return true;
    }

    // A watched link's own record is the link's `lstat`, not its target's.
    if self.tracks_link(path) {
      return true;
    }
    let Some(current_mtime) = disk_mtime(logical) else {
      return true;
    };
    let mtime_ms = mtime_to_millis(current_mtime);
    if self.baselines.insert(logical.clone(), Some(current_mtime)) == Some(Some(current_mtime))
      && mtime_ms.saturating_add(mtime_accuracy(current_mtime)) < current_time()
    {
      return false;
    }
    self.set_file_time(logical, current_mtime, false, false);
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
    let mut registered = Vec::new();
    let mut unregistered = Vec::new();
    for (paths, tracker) in [
      (PathUpdater::from(files), &self.files),
      (PathUpdater::from(directories), &self.directories),
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
        // Record the identity a registered context is observed with, so a
        // later event on the same directory is not mistaken for a replacement.
        if let Ok(metadata) = std::fs::symlink_metadata(dir.as_path()) {
          self
            .directory_identities
            .insert(dir.clone(), DirectoryIdentity::of(&metadata));
        }
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
      let still_missing = self.missing.all.contains(path);
      if !still_missing {
        self.baselines.remove(path);
      }
      if !self.is_below_context(path) {
        self.file_times.remove(path);
        if !still_missing {
          self.unindex_record(path);
        }
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
        self.unindex_record(path);
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
        self.record(dir, || self.context_directories.insert(dir.clone()));
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
        self.last_watch_events.remove(path);
        self.unlink_unrecorded(path);
      }
    }
  }

  /// Create a new `PathAccessor` to access the current state of paths, directories, and missing paths.
  ///
  /// When a symbolic link maps a logical dependency onto another physical
  /// path, the analyzer must watch that physical path instead. Everything else
  /// — event routing, time info, `ignored` — keeps using the logical sets.
  pub fn access(&self) -> PathAccessor<'_> {
    if self.read_links().is_empty() {
      return self.logical_access();
    }
    PathAccessor::new(
      SetRefs {
        all: &self.physical_files.all,
        added: &self.physical_files.added,
        removed: &self.physical_files.removed,
      },
      SetRefs {
        all: &self.physical_directories.all,
        added: &self.physical_directories.added,
        removed: &self.physical_directories.removed,
      },
      SetRefs {
        all: &self.physical_missing.all,
        added: &self.physical_missing.added,
        removed: &self.physical_missing.removed,
      },
    )
  }

  /// Create a `PathAccessor` over the logical registrations, regardless of the
  /// physical mapping.
  pub(crate) fn logical_access(&self) -> PathAccessor<'_> {
    PathAccessor::new(
      SetRefs {
        all: &self.files.all,
        added: &self.files.added,
        removed: &self.files.removed,
      },
      SetRefs {
        all: &self.directories.all,
        added: &self.directories.added,
        removed: &self.directories.removed,
      },
      SetRefs {
        all: &self.missing.all,
        added: &self.missing.added,
        removed: &self.missing.removed,
      },
    )
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
    let accessor = self.logical_access();
    let path_to_string = |path: &Path| {
      path
        .to_str()
        .map_or_else(|| path.to_string_lossy().into_owned(), str::to_owned)
    };

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
      file_timestamps.push((path_to_string(path.as_path()), entry));
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
      file_timestamps.push((path_to_string(path.as_path()), entry));
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
      file_timestamps.push((path_to_string(path.as_path()), Self::entry(time)));
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
    // A directory registered through a symbolic link reports the time of the
    // directory it resolves to, like watchpack's `DirectoryWatcher` for the
    // link: the link itself is only a name for that directory.
    let mut link_directory_times: FxHashMap<InternedPath, SystemTime> = FxHashMap::default();
    for dir in &dir_paths {
      if !dir.is_symlink() {
        continue;
      }
      if let Ok(mtime) = dir
        .metadata()
        .and_then(|metadata| metadata.modified().or_else(|_| metadata.created()))
      {
        link_directory_times.insert(dir.clone(), mtime);
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
    // A directory registered through a symbolic link is reported as a file
    // entry carrying its time, like watchpack's `DirectoryWatcher` for the
    // link: the link itself is a directory entry, and its time info is what a
    // consumer uses to decide the link still resolves where it did.
    let link_directories: InternedPathSet = link_directory_times.keys().cloned().collect();
    for (dir, mtime) in link_directory_times {
      let time = FileTime::observed(mtime);
      file_timestamps.push((path_to_string(dir.as_path()), Self::entry(time)));
    }

    let mut directory_timestamps = TimeInfoEntries::with_capacity(dir_paths.len());
    for dir in &dir_paths {
      let path = path_to_string(dir.as_path());
      // A link to a directory reports its own time as a file entry, like
      // watchpack; its directory entry still takes the same safe time as any
      // context.
      if link_directories.contains(dir) {
        match dir_safe_times.get(dir).copied().flatten() {
          Some(safe_time) => {
            directory_timestamps.push((path, TimeInfoEntry::OnlySafeTimeEntry { safe_time }));
          }
          None => {
            file_timestamps.push((path.clone(), TimeInfoEntry::Null));
            directory_timestamps.push((path, TimeInfoEntry::Null));
          }
        }
        continue;
      }
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

    let accessor = path_manager.logical_access();
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
