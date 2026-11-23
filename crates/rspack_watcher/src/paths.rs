pub(crate) mod links;

use std::{
  ffi::OsStr,
  fmt::Debug,
  future::Future,
  ops::Deref,
  path::{Path, PathBuf},
  pin::Pin,
  sync::{Arc, RwLock},
  time::SystemTime,
};

use dashmap::setref::multiple::RefMulti;
use links::{
  BoundDep, ContextMapping, DepKind, EntrySubscriber, LinkMappings, LinkResolver, LinkState,
  push_unique,
};
use rspack_error::Result;
use rspack_paths::{InternedPath, InternedPathDashSet, InternedPathMap, InternedPathSet};
use rspack_util::{
  fx_hash::{FxHashMap, FxHashSet},
  time::{current_time, system_time_to_millis},
};

use super::{FsWatcherIgnored, TimeInfoEntries, TimeInfoEntry, ignored::IgnoredMatcher};
use crate::time::{fs_accuracy, mtime_accuracy, mtime_safe_time, mtime_to_millis};

pub(crate) fn directory_identity(path: &Path, metadata: &std::fs::Metadata) -> Option<(u64, u64)> {
  #[cfg(unix)]
  {
    let _ = path;
    use std::os::unix::fs::MetadataExt;
    Some((metadata.dev(), metadata.ino()))
  }
  #[cfg(windows)]
  {
    use std::os::windows::fs::{MetadataExt, OpenOptionsExt};
    if let Some(vol) = metadata.volume_serial_number()
      && let Some(idx) = metadata.file_index()
    {
      return Some((vol.into(), idx));
    }
    const FILE_FLAG_BACKUP_SEMANTICS: u32 = 0x0200_0000;
    let file = std::fs::OpenOptions::new()
      .access_mode(0)
      .custom_flags(FILE_FLAG_BACKUP_SEMANTICS)
      .open(path)
      .ok()?;
    let handle_meta = file.metadata().ok()?;
    Some((
      handle_meta.volume_serial_number()?.into(),
      handle_meta.file_index()?,
    ))
  }
  #[cfg(not(any(unix, windows)))]
  {
    let _ = (path, metadata);
    None
  }
}

pub(crate) struct All<'a> {
  inner: Box<dyn Iterator<Item = RefMulti<'a, InternedPath>> + 'a>,
}

impl<'a> All<'a> {
  fn new(
    files: &'a InternedPathDashSet,
    directories: &'a InternedPathDashSet,
    missing: &'a InternedPathDashSet,
  ) -> Self {
    let chain = files.iter().chain(directories.iter()).chain(missing.iter());
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

pub(crate) struct PathAccessor<'a> {
  files: &'a PathTracker,
  directories: &'a PathTracker,
  missing: &'a PathTracker,
}

impl<'a> PathAccessor<'a> {
  fn new(path_manager: &'a PathManager) -> Self {
    Self {
      files: &path_manager.files,
      directories: &path_manager.directories,
      missing: &path_manager.missing,
    }
  }

  fn new_watched(path_manager: &'a PathManager) -> Self {
    Self {
      files: &path_manager.watched_files,
      directories: &path_manager.watched_directories,
      missing: &path_manager.watched_missing,
    }
  }

  pub fn files(
    &self,
  ) -> (
    &'a InternedPathDashSet,
    &'a InternedPathDashSet,
    &'a InternedPathDashSet,
  ) {
    (&self.files.all, &self.files.added, &self.files.removed)
  }

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

  pub fn all(&self) -> impl Iterator<Item = InternedPath> + '_ {
    All::new(&self.files.all, &self.directories.all, &self.missing.all)
  }
}

#[derive(Debug)]
struct PathUpdater {
  pub added: Vec<InternedPath>,
  pub removed: Vec<InternedPath>,
  base_dir: PathBuf,
}

impl PathUpdater {
  fn from_vecs((added, removed): (Vec<InternedPath>, Vec<InternedPath>)) -> Self {
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
    Self::from_vecs((added.collect(), removed.collect()))
  }
}

impl PathUpdater {
  async fn update(
    self,
    watch_tracker: &PathTracker,
    ignored: &IgnoredMatcher,
  ) -> Result<(Vec<InternedPath>, Vec<InternedPath>)> {
    let mut registered = Vec::new();
    let mut unregistered = Vec::new();

    for added in self.added {
      let added = if added.is_absolute() {
        added
      } else {
        InternedPath::from(self.base_dir.join(added.as_ref()))
      };

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

    for removed in self.removed {
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
pub(crate) struct PathTracker {
  added: InternedPathDashSet,
  removed: InternedPathDashSet,
  all: InternedPathDashSet,
}

impl PathTracker {
  pub fn reset(&self) {
    self.added.clear();
    self.removed.clear();
  }

  pub fn add(&self, path: InternedPath) -> bool {
    if self.removed.remove(&path).is_none() {
      self.added.insert(path.clone());
    }
    self.all.insert(path)
  }

  pub fn remove(&self, path: InternedPath) -> bool {
    let tracked = self.all.remove(&path).is_some();
    if self.added.remove(&path).is_none() {
      self.removed.insert(path);
    }
    tracked
  }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct FileTime {
  mtime: SystemTime,
  safe_time: u64,
  accuracy: u64,
}

impl FileTime {
  fn initial(mtime: SystemTime) -> Self {
    let mtime_ms = mtime_to_millis(mtime);
    let accuracy = mtime_accuracy(mtime);
    Self {
      mtime,
      safe_time: current_time().min(mtime_ms).saturating_add(accuracy),
      accuracy,
    }
  }

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

  fn observed(mtime: SystemTime) -> Self {
    mtime_accuracy(mtime);
    Self {
      mtime,
      safe_time: current_time(),
      accuracy: 0,
    }
  }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PhysicalBaseline {
  mtime: Option<SystemTime>,
  link: Option<LinkState>,
}

#[derive(Default, Clone)]
pub(crate) struct ScanPlan {
  pub files: Vec<InternedPath>,
  pub newly_added_files: Vec<InternedPath>,
  pub directories: Vec<InternedPath>,
  pub contexts_to_scan: Vec<(InternedPath, bool)>,
  pub missing: Vec<InternedPath>,
  pub replaced_physical_dirs: Vec<InternedPath>,
}

#[derive(Default)]
pub(crate) struct ContextScanOutput {
  pub changed_entries: Vec<(InternedPath, InternedPath)>,
  pub added_watched_paths: bool,
  pub attached_contexts: Vec<InternedPath>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ContextEntryOutcome {
  Unchanged,
  Changed,
  ExistingDirectory,
}

#[derive(Clone, Copy)]
pub(crate) struct LiveEntryFlags {
  pub judged: bool,
  pub is_disk: bool,
  pub renamed: bool,
}

#[derive(Default)]
struct RecordState {
  file_times: InternedPathMap<FileTime>,
  baselines: InternedPathMap<PhysicalBaseline>,
  last_watch_events: InternedPathMap<u64>,
  context_files: InternedPathMap<Vec<InternedPath>>,
  context_directories: InternedPathMap<Vec<InternedPath>>,
  directory_identities: InternedPathMap<(u64, u64)>,
  registered_ancestors: InternedPathMap<usize>,
  recorded_children: InternedPathMap<InternedPathSet>,
  absent_directories: InternedPathSet,
  pending_context_rescan: InternedPathSet,
}

impl RecordState {
  fn holds_record(&self, path: &InternedPath) -> bool {
    self.file_times.contains_key(path)
      || self.context_files.contains_key(path)
      || self.context_directories.contains_key(path)
      || self.last_watch_events.contains_key(path)
  }

  fn record<R>(&mut self, path: &InternedPath, write: impl FnOnce(&mut Self) -> R) -> R {
    let written = write(self);
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

  fn unlink_unrecorded(&mut self, path: &InternedPath) {
    let mut child = path.clone();
    while !self.holds_record(&child) && !self.recorded_children.contains_key(&child) {
      let Some(parent) = child.parent().map(InternedPath::from) else {
        return;
      };
      let emptied = match self.recorded_children.get_mut(&parent) {
        Some(siblings) => siblings.remove(&child) && siblings.is_empty(),
        None => false,
      };
      if !emptied {
        return;
      }
      self.recorded_children.remove(&parent);
      child = parent;
    }
  }

  fn write_file_time(
    &mut self,
    path: &InternedPath,
    mtime: SystemTime,
    initial: bool,
    ignore_when_equal: bool,
  ) -> bool {
    if self
      .baselines
      .get(path)
      .is_some_and(|b| b.mtime.is_none() && b.link.is_none())
    {
      self.baselines.remove(path);
    }
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

  fn retain_registered_ancestors(&mut self, path: &InternedPath) {
    let mut child = path.clone();
    while let Some(parent) = child.parent().map(InternedPath::from) {
      let count = self.registered_ancestors.entry(parent.clone()).or_default();
      *count += 1;
      if *count > 1 {
        return;
      }
      child = parent;
    }
  }

  fn release_registered_ancestors(&mut self, path: &InternedPath) {
    let mut child = path.clone();
    while let Some(parent) = child.parent().map(InternedPath::from) {
      let released = self
        .registered_ancestors
        .get_mut(&parent)
        .is_some_and(|count| {
          *count -= 1;
          *count == 0
        });
      if !released {
        return;
      }
      self.registered_ancestors.remove(&parent);
      child = parent;
    }
  }
}

#[cfg(test)]
#[derive(Default, Clone)]
struct FileTimeAccessor(Arc<RwLock<RecordState>>);

#[cfg(test)]
impl FileTimeAccessor {
  fn get(&self, path: &InternedPath) -> Option<FileTime> {
    self
      .0
      .read()
      .expect("records lock poisoned")
      .file_times
      .get(path)
      .copied()
  }
}

pub(crate) struct PathManager {
  files: PathTracker,
  directories: PathTracker,
  missing: PathTracker,
  watched_files: PathTracker,
  watched_directories: PathTracker,
  watched_missing: PathTracker,
  ignored: IgnoredMatcher,
  records: Arc<RwLock<RecordState>>,
  #[cfg(test)]
  file_times: FileTimeAccessor,
  links: RwLock<LinkMappings>,
  #[cfg(any(test, feature = "testing"))]
  pending_scan: std::sync::Mutex<Option<ScanPlan>>,
  follow_symlinks: bool,
}

impl Default for PathManager {
  fn default() -> Self {
    Self::new(FsWatcherIgnored::None)
  }
}

impl PathManager {
  pub fn new(ignored: FsWatcherIgnored) -> Self {
    let records = Arc::new(RwLock::new(RecordState::default()));
    Self {
      files: PathTracker::default(),
      directories: PathTracker::default(),
      missing: PathTracker::default(),
      watched_files: PathTracker::default(),
      watched_directories: PathTracker::default(),
      watched_missing: PathTracker::default(),
      ignored: IgnoredMatcher::new(ignored),
      #[cfg(test)]
      file_times: FileTimeAccessor(Arc::clone(&records)),
      records,
      links: RwLock::new(LinkMappings::default()),
      #[cfg(any(test, feature = "testing"))]
      pending_scan: std::sync::Mutex::new(None),
      follow_symlinks: false,
    }
  }

  pub fn with_follow_symlinks(mut self, follow_symlinks: bool) -> Self {
    self.follow_symlinks = follow_symlinks;
    self
  }

  pub fn follows_symlinks(&self) -> bool {
    self.follow_symlinks
  }

  fn watched_trackers(&self) -> [&PathTracker; 3] {
    [
      &self.watched_files,
      &self.watched_directories,
      &self.watched_missing,
    ]
  }

  pub fn reset(&self) {
    self.files.reset();
    self.directories.reset();
    self.missing.reset();
  }

  pub fn reset_watched_deltas(&self) {
    self.watched_files.reset();
    self.watched_directories.reset();
    self.watched_missing.reset();
  }

  pub fn access(&self) -> PathAccessor<'_> {
    PathAccessor::new(self)
  }

  pub fn watched_access(&self) -> PathAccessor<'_> {
    PathAccessor::new_watched(self)
  }

  pub fn watched_root_parents(&self) -> Vec<InternedPath> {
    let links = self.links.read().expect("links lock poisoned");
    let mut parents = InternedPathSet::default();
    for dir in links.root_dir_refs.keys() {
      if let Some(parent) = dir.parent().filter(|p| *p != dir.as_path() && p.is_dir()) {
        parents.insert(InternedPath::from(parent));
      }
    }
    parents.into_iter().collect()
  }

  #[cfg(any(test, feature = "testing"))]
  pub fn take_scan_plan(&self) -> Option<ScanPlan> {
    self
      .pending_scan
      .lock()
      .expect("scan plan lock poisoned")
      .take()
  }

  fn prune_evicted_physical_states(&self, evicted: &[InternedPath]) {
    if evicted.is_empty() {
      return;
    }
    let mut drop_baselines = Vec::new();
    let mut drop_identities = Vec::new();
    {
      let links = self.links.read().expect("links lock poisoned");
      for path in evicted {
        if !self.files.all.contains(path)
          && !self.missing.all.contains(path)
          && !links.is_physically_watched_file_or_missing(path)
        {
          drop_baselines.push(path.clone());
        }
        if !self.directories.all.contains(path) && !links.is_physically_watched_dir(path) {
          drop_identities.push(path.clone());
        }
      }
    }
    if drop_baselines.is_empty() && drop_identities.is_empty() {
      return;
    }
    let mut records = self.records.write().expect("records lock poisoned");
    for path in drop_baselines {
      records.baselines.remove(&path);
    }
    for path in drop_identities {
      if !records.context_directories.contains_key(&path) {
        records.directory_identities.remove(&path);
      }
    }
  }

  pub fn set_file_time_if_absent(&self, path: InternedPath, mtime: SystemTime) {
    let link = path.is_symlink().then(|| LinkState::read(path.as_path()));
    let mut records = self.records.write().expect("records lock poisoned");
    records.record(&path, |rec| {
      let entry = rec
        .baselines
        .entry(path.clone())
        .or_insert(PhysicalBaseline {
          mtime: None,
          link: link.clone(),
        });
      if entry.mtime.is_none() {
        entry.mtime = Some(mtime);
        entry.link = link;
      }
      rec
        .file_times
        .entry(path.clone())
        .or_insert_with(|| FileTime::initial(mtime));
    });
  }

  pub fn seed_physical_baseline_if_absent(&self, physical: &InternedPath) {
    if self
      .records
      .read()
      .expect("records lock poisoned")
      .baselines
      .get(physical)
      .is_some_and(|b| b.mtime.is_some() || b.link.is_some())
    {
      return;
    }
    let link = physical
      .is_symlink()
      .then(|| LinkState::read(physical.as_path()));
    let mtime = disk_mtime(physical);
    if mtime.is_some() || link.is_some() {
      self
        .records
        .write()
        .expect("records lock poisoned")
        .baselines
        .entry(physical.clone())
        .or_insert(PhysicalBaseline { mtime, link });
    }
  }

  pub fn clear_physical_baseline(&self, physical: &InternedPath) {
    self
      .records
      .write()
      .expect("records lock poisoned")
      .baselines
      .remove(physical);
  }

  pub fn set_reported_file_time(&self, path: &InternedPath, mtime: SystemTime) {
    if !self.files.all.contains(path) && !self.missing.all.contains(path) {
      return;
    }
    let link = path.is_symlink().then(|| LinkState::read(path.as_path()));
    let targets = self.retained_items(DepKind::File, path);
    let mut item_states = Vec::new();
    for item in targets {
      if item != *path {
        let item_mtime = std::fs::symlink_metadata(&item)
          .and_then(|m| m.modified().or_else(|_| m.created()))
          .ok();
        let item_link = item.is_symlink().then(|| LinkState::read(item.as_path()));
        item_states.push((
          item,
          PhysicalBaseline {
            mtime: item_mtime,
            link: item_link,
          },
        ));
      }
    }
    if !self.files.all.contains(path) && !self.missing.all.contains(path) {
      return;
    }
    let mut records = self.records.write().expect("records lock poisoned");
    if !self.files.all.contains(path) && !self.missing.all.contains(path) {
      return;
    }
    records.baselines.insert(
      path.clone(),
      PhysicalBaseline {
        mtime: Some(mtime),
        link,
      },
    );
    for (item, state) in item_states {
      records.baselines.insert(item, state);
    }
    records.record(path, |rec| rec.write_file_time(path, mtime, true, true));
  }

  pub fn is_known_absent(&self, path: &InternedPath) -> bool {
    self
      .records
      .read()
      .expect("records lock poisoned")
      .baselines
      .get(path)
      .is_some_and(|b| b.mtime.is_none() && b.link.is_none())
  }

  pub fn set_reported_file_removal(&self, path: &InternedPath) {
    let keep = (self.files.all.contains(path) || self.missing.all.contains(path))
      && !self.is_dep_present_on_disk(DepKind::File, path);
    let mut records = self.records.write().expect("records lock poisoned");
    if keep {
      records.baselines.insert(
        path.clone(),
        PhysicalBaseline {
          mtime: None,
          link: None,
        },
      );
    } else if records
      .baselines
      .get(path)
      .is_some_and(|b| b.mtime.is_none() && b.link.is_none())
    {
      records.baselines.remove(path);
    }
  }

  fn active_physically_watched_dirs(&self) -> InternedPathSet {
    let links = self.links.read().expect("links lock poisoned");
    let mut set: InternedPathSet = links.root_dir_refs.keys().cloned().collect();
    set.extend(
      links.watch_refs[DepKind::Directory as usize]
        .keys()
        .cloned(),
    );
    set
  }

  pub fn remove_file_time(&self, path: &InternedPath) {
    let keep_dir_identity = self.directories.all.contains(path)
      || self
        .links
        .read()
        .expect("links lock poisoned")
        .is_physically_watched_dir(path);
    let mut records = self.records.write().expect("records lock poisoned");
    records.file_times.remove(path);
    if records
      .baselines
      .get(path)
      .is_some_and(|b| b.mtime.is_some() || b.link.is_some())
    {
      records.baselines.remove(path);
    }
    records.context_files.remove(path);
    if records.context_directories.remove(path).is_some() {
      records.last_watch_events.remove(path);
    }
    if keep_dir_identity {
      if let Some(slot) = records.directory_identities.get_mut(path) {
        *slot = (0, 0);
      }
    } else {
      records.directory_identities.remove(path);
    }
    records.unlink_unrecorded(path);
  }

  pub fn remove_file_times_under(&self, dir: &InternedPath) {
    let active_phys_dirs = self.active_physically_watched_dirs();
    {
      let mut records = self.records.write().expect("records lock poisoned");
      let mut pending = vec![dir.clone()];
      while let Some(path) = pending.pop() {
        if let Some(children) = records.recorded_children.remove(&path) {
          pending.extend(children);
        }
        records.file_times.remove(&path);
        if records
          .baselines
          .get(&path)
          .is_some_and(|b| b.mtime.is_some() || b.link.is_some())
        {
          records.baselines.remove(&path);
        }
        records.context_files.remove(&path);
        records.context_directories.remove(&path);
        if self.directories.all.contains(&path) || active_phys_dirs.contains(&path) {
          if let Some(slot) = records.directory_identities.get_mut(&path) {
            *slot = (0, 0);
          }
        } else {
          records.directory_identities.remove(&path);
        }
        if !self.directories.all.contains(&path) {
          records.last_watch_events.remove(&path);
        }
      }
      records.unlink_unrecorded(dir);
    }

    let mut newly_absent = Vec::new();
    {
      let links = self.links.read().expect("links lock poisoned");
      for context in self.directories.all.iter() {
        let target_under_dir = links
          .get_bound(DepKind::Directory, &context)
          .is_some_and(|b| b.target_active && b.target.starts_with(dir.as_ref() as &Path));
        if context.starts_with(dir.as_ref() as &Path) || target_under_dir {
          newly_absent.push(context.clone());
        }
      }
    }
    if !newly_absent.is_empty() {
      let mut records = self.records.write().expect("records lock poisoned");
      records.absent_directories.extend(newly_absent);
    }
  }

  pub fn index_sizes(&self) -> (usize, usize) {
    let (recorded_tree, reg_ancestors) = {
      let records = self.records.read().expect("records lock poisoned");
      let rec_sum: usize = records
        .recorded_children
        .values()
        .map(|children| 1 + children.len())
        .sum();
      (rec_sum, records.registered_ancestors.len())
    };
    let links = self.links.read().expect("links lock poisoned");
    (
      recorded_tree + links.recorded_size(),
      reg_ancestors + links.registered_size(),
    )
  }

  pub fn has_file_time(&self, path: &InternedPath) -> bool {
    self
      .records
      .read()
      .expect("records lock poisoned")
      .file_times
      .contains_key(path)
  }

  pub fn is_registered(&self, kind: DepKind, path: &InternedPath) -> bool {
    match kind {
      DepKind::File => self.files.all.contains(path),
      DepKind::Directory => self.directories.all.contains(path),
      DepKind::Missing => self.missing.all.contains(path),
    }
  }

  pub fn is_any_registered_or_below_context(&self, path: &InternedPath) -> bool {
    self.files.all.contains(path)
      || self.directories.all.contains(path)
      || self.missing.all.contains(path)
      || self.is_below_context(path)
  }

  pub fn retained_items(&self, kind: DepKind, logical_path: &InternedPath) -> Vec<InternedPath> {
    let links = self.links.read().expect("links lock poisoned");
    if let Some(bound) = links.get_bound(kind, logical_path) {
      bound.retained_items()
    } else {
      vec![logical_path.clone()]
    }
  }

  pub fn bound_dep(&self, kind: DepKind, logical_path: &InternedPath) -> Option<BoundDep> {
    self
      .links
      .read()
      .expect("links lock poisoned")
      .get_bound(kind, logical_path)
      .cloned()
  }

  pub fn is_dep_present_on_disk(&self, kind: DepKind, logical_path: &InternedPath) -> bool {
    if kind == DepKind::Directory {
      return match self.bound_dep(DepKind::Directory, logical_path) {
        Some(bound) if !bound.target_active => bound
          .links
          .iter()
          .all(|l| std::fs::symlink_metadata(l).is_ok()),
        Some(bound) => {
          bound.target.exists()
            && bound
              .links
              .iter()
              .all(|l| std::fs::symlink_metadata(l).is_ok())
        }
        None => logical_path.exists(),
      };
    }
    let items = self.retained_items(kind, logical_path);
    if items.is_empty() {
      return std::fs::symlink_metadata(logical_path).is_ok();
    }
    items
      .iter()
      .all(|item| std::fs::symlink_metadata(item).is_ok())
  }

  pub fn context_filter_root(&self, root_ctx: &InternedPath) -> InternedPath {
    if self.follow_symlinks
      && let Some(bound) = self.bound_dep(DepKind::Directory, root_ctx)
    {
      bound.target
    } else {
      root_ctx.clone()
    }
  }

  pub fn filter_path_for_context(
    &self,
    root_ctx: &InternedPath,
    logical_path: &InternedPath,
  ) -> InternedPath {
    let links = self.links.read().expect("links lock poisoned");
    let mut best: Option<(&Path, &InternedPath)> = None;
    for mappings in links.dir_aliases.values() {
      for m in mappings {
        if m.root_context == *root_ctx
          && logical_path.starts_with(m.record_prefix.as_path())
          && best.is_none_or(|(prev, _)| m.record_prefix.as_os_str().len() > prev.as_os_str().len())
        {
          best = Some((m.record_prefix.as_path(), &m.filter_prefix));
        }
      }
    }
    if let Some((rec_pref, filt_pref)) = best
      && let Ok(rel) = logical_path.strip_prefix(rec_pref)
    {
      return if rel.as_os_str().is_empty() {
        filt_pref.clone()
      } else {
        InternedPath::from(filt_pref.join(rel))
      };
    }
    if self.follow_symlinks
      && let Some(bound) = links.get_bound(DepKind::Directory, root_ctx)
      && let Ok(rel) = logical_path.strip_prefix(root_ctx.as_path())
    {
      return if rel.as_os_str().is_empty() {
        bound.target.clone()
      } else {
        InternedPath::from(bound.target.join(rel))
      };
    }
    logical_path.clone()
  }

  async fn compute_bound_dep(
    &self,
    resolver: &mut LinkResolver,
    kind: DepKind,
    logical_path: &InternedPath,
  ) -> BoundDep {
    let resolved = match kind {
      DepKind::Directory => resolver.resolve_directory_root(logical_path),
      DepKind::File | DepKind::Missing => resolver.resolve(logical_path),
    };
    let mut active_links = Vec::new();
    for link in &resolved.links {
      if (matches!(kind, DepKind::File | DepKind::Missing) && link == logical_path)
        || !self.is_ignored_path(link.as_path()).await
      {
        active_links.push(link.clone());
      }
    }
    let target_active = if self.follow_symlinks {
      match kind {
        DepKind::File | DepKind::Missing => {
          resolved.target == *logical_path || !self.is_ignored_path(resolved.target.as_path()).await
        }
        DepKind::Directory => !self.is_ignored_path(resolved.target.as_path()).await,
      }
    } else {
      true
    };
    let first_missing = match resolved.first_missing {
      Some(head) if !self.is_ignored_path(head.as_path()).await => Some(head),
      _ => None,
    };
    BoundDep {
      target: resolved.target,
      target_active,
      links: active_links,
      first_missing,
    }
  }

  pub async fn rebind_single_dep(
    &self,
    kind: DepKind,
    logical_path: &InternedPath,
  ) -> Option<BoundDep> {
    if !self.is_registered(kind, logical_path) {
      return None;
    }
    let mut resolver = LinkResolver::new(self.follow_symlinks);
    let bound = self
      .compute_bound_dep(&mut resolver, kind, logical_path)
      .await;
    if !self.is_registered(kind, logical_path) {
      return None;
    }
    let mut evicted = Vec::new();
    let root_dir_to_seed = {
      let mut links = self.links.write().expect("links lock poisoned");
      links.bind_dep(
        kind,
        logical_path.clone(),
        bound.clone(),
        self.follow_symlinks,
        self.watched_trackers(),
        &mut evicted,
      );
      if bound.target_active {
        if kind == DepKind::Directory {
          Some(bound.target.clone())
        } else {
          bound.target.parent().map(InternedPath::from)
        }
      } else {
        None
      }
    };
    self.prune_evicted_physical_states(&evicted);
    if let Some(dir) = root_dir_to_seed
      && let Ok(meta) = dir.metadata()
      && meta.is_dir()
      && let Some(id) = directory_identity(dir.as_path(), &meta)
    {
      self
        .records
        .write()
        .expect("records lock poisoned")
        .directory_identities
        .entry(dir)
        .or_insert(id);
    }
    Some(bound)
  }

  pub fn mark_dep_dirty(&self, kind: DepKind, logical_path: &InternedPath) {
    self.links.write().expect("links lock poisoned").dirty_deps[kind as usize]
      .insert(logical_path.clone());
  }

  pub fn check_and_update_link_state(&self, link_path: &InternedPath, is_disk: bool) -> bool {
    let current = LinkState::read(link_path.as_path());
    let mut links = self.links.write().expect("links lock poisoned");
    if !links.link_ref_counts.contains_key(link_path) {
      return !is_disk;
    }
    let previous = links.link_states.insert(link_path.clone(), current.clone());
    if !is_disk {
      if let Some(mtime) = current.mtime {
        mtime_accuracy(mtime);
      }
      return true;
    }
    let unchanged = previous.as_ref() == Some(&current);
    match current.mtime {
      Some(mtime) => {
        let accuracy = mtime_accuracy(mtime);
        !(unchanged && mtime_to_millis(mtime).saturating_add(accuracy) < current_time())
      }
      None => !unchanged,
    }
  }

  pub fn check_context_link_changed(
    &self,
    owners: &[InternedPath],
    link_path: &InternedPath,
    is_disk: bool,
  ) -> bool {
    if owners.is_empty() {
      return false;
    }
    let current = LinkState::read(link_path.as_path());
    let mut any_active_owner = false;
    let mut all_unchanged = true;
    {
      let mut links = self.links.write().expect("links lock poisoned");
      for owner in owners {
        if !self.directories.all.contains(owner) {
          continue;
        }
        any_active_owner = true;
        let bindings = links
          .reverse_context_scans
          .entry(owner.clone())
          .or_default();
        if let Some(existing) = bindings.scanned_links.get_mut(link_path) {
          if *existing != current {
            all_unchanged = false;
            *existing = current.clone();
          }
        } else {
          all_unchanged = false;
          bindings
            .scanned_links
            .insert(link_path.clone(), current.clone());
        }
      }
    }
    if !any_active_owner {
      return false;
    }
    if !is_disk || !all_unchanged {
      if let Some(mtime) = current.mtime {
        mtime_accuracy(mtime);
      }
      return true;
    }
    match current.mtime {
      Some(mtime) => {
        let accuracy = mtime_accuracy(mtime);
        mtime_to_millis(mtime).saturating_add(accuracy) >= current_time()
      }
      None => false,
    }
  }

  fn collect_dirty_context_links(&self) -> (Vec<InternedPath>, InternedPathSet) {
    let links = self.links.read().expect("links lock poisoned");
    let mut dir_candidates = Vec::new();
    let mut dirty_roots = InternedPathSet::default();
    for dir in &links.dirty_deps[DepKind::Directory as usize] {
      push_unique(&mut dir_candidates, dir.clone());
    }
    for (logical_dir, bound) in &links.bound_deps[DepKind::Directory as usize] {
      if bound.links.iter().any(|l| {
        links
          .link_states
          .get(l)
          .is_some_and(|old| *old != LinkState::read(l.as_path()))
      }) {
        push_unique(&mut dir_candidates, logical_dir.clone());
      }
    }
    for (root_ctx, bindings) in &links.reverse_context_scans {
      if bindings
        .scanned_links
        .iter()
        .any(|(l, old)| *old != LinkState::read(l.as_path()))
      {
        dirty_roots.insert(root_ctx.clone());
      }
    }
    (dir_candidates, dirty_roots)
  }

  pub fn has_dirty_context_links(&self) -> bool {
    if !self
      .records
      .read()
      .expect("records lock poisoned")
      .pending_context_rescan
      .is_empty()
    {
      return true;
    }
    let (dirs, roots) = self.collect_dirty_context_links();
    !dirs.is_empty() || !roots.is_empty()
  }

  pub async fn refresh_dirty_contexts(&self) -> Vec<(InternedPath, bool)> {
    let (dir_candidates, mut dirty_roots) = self.collect_dirty_context_links();
    {
      let mut records = self.records.write().expect("records lock poisoned");
      dirty_roots.extend(std::mem::take(&mut records.pending_context_rescan));
    }

    let mut resolver = LinkResolver::new(self.follow_symlinks);
    let mut evicted = Vec::new();
    for dir in dir_candidates {
      if !self.directories.all.contains(&dir) {
        continue;
      }
      let bound = self
        .compute_bound_dep(&mut resolver, DepKind::Directory, &dir)
        .await;
      {
        let mut links = self.links.write().expect("links lock poisoned");
        links.dirty_deps[DepKind::Directory as usize].remove(&dir);
        links.clear_context_scan(&dir, self.watched_trackers(), &mut evicted);
        links.bind_dep(
          DepKind::Directory,
          dir.clone(),
          bound,
          self.follow_symlinks,
          self.watched_trackers(),
          &mut evicted,
        );
      }
      self.forget_context_entries_owned_by(&dir);
      dirty_roots.insert(dir);
    }
    self.prune_evicted_physical_states(&evicted);

    dirty_roots.into_iter().map(|d| (d, true)).collect()
  }

  pub fn resolve_entry_subscribers(&self, disk_path: &InternedPath) -> Vec<EntrySubscriber> {
    let mut out = Vec::new();
    let links = self.links.read().expect("links lock poisoned");
    if let Some(subs) = links.entry_aliases.get(disk_path) {
      for sub in subs {
        push_unique(&mut out, sub.clone());
      }
    }
    for (kind, tracker) in [
      (DepKind::File, &self.files),
      (DepKind::Missing, &self.missing),
      (DepKind::Directory, &self.directories),
    ] {
      if tracker.all.contains(disk_path) && !links.bound_deps[kind as usize].contains_key(disk_path)
      {
        push_unique(
          &mut out,
          EntrySubscriber::Registered {
            kind,
            logical_path: disk_path.clone(),
            is_link_hop: false,
          },
        );
      }
    }
    out
  }

  pub fn resolve_context_subscribers(
    &self,
    disk_path: &InternedPath,
    allow_logical_fallback: bool,
  ) -> Vec<(InternedPath, InternedPath, InternedPath)> {
    let mut out = Vec::new();
    let links = self.links.read().expect("links lock poisoned");
    for ancestor in disk_path.ancestors() {
      let ancestor_interned = InternedPath::from(ancestor);
      let rel = disk_path.strip_prefix(ancestor).expect("ancestor prefix");
      if let Some(mappings) = links.dir_aliases.get(&ancestor_interned) {
        for mapping in mappings {
          let logical_entry = if rel.as_os_str().is_empty() {
            mapping.record_prefix.clone()
          } else {
            InternedPath::from(mapping.record_prefix.join(rel))
          };
          let filter_entry = if rel.as_os_str().is_empty() {
            mapping.filter_prefix.clone()
          } else {
            InternedPath::from(mapping.filter_prefix.join(rel))
          };
          push_unique(
            &mut out,
            (mapping.root_context.clone(), logical_entry, filter_entry),
          );
        }
      }
      if !rel.as_os_str().is_empty()
        && self.directories.all.contains(&ancestor_interned)
        && (!links.bound_deps[DepKind::Directory as usize].contains_key(&ancestor_interned)
          || allow_logical_fallback)
      {
        let filter_entry = if let Some(bound) =
          links.get_bound(DepKind::Directory, &ancestor_interned)
          && self.follow_symlinks
        {
          InternedPath::from(bound.target.join(rel))
        } else {
          disk_path.clone()
        };
        push_unique(
          &mut out,
          (ancestor_interned, disk_path.clone(), filter_entry),
        );
      }
    }
    out
  }

  pub fn take_pending_missing_for_path(
    &self,
    disk_path: &InternedPath,
    include_descendants: bool,
  ) -> Vec<(DepKind, InternedPath)> {
    let links = self.links.read().expect("links lock poisoned");
    let mut out = Vec::new();
    if let Some(waiting) = links.pending_missing_heads.get(disk_path)
      && std::fs::symlink_metadata(disk_path).is_ok()
    {
      for item in waiting {
        push_unique(&mut out, item.clone());
      }
    }
    if include_descendants {
      for desc in links.collect_physical_descendants(disk_path) {
        if std::fs::symlink_metadata(&desc).is_ok()
          && let Some(waiting) = links.pending_missing_heads.get(&desc)
        {
          for item in waiting {
            push_unique(&mut out, item.clone());
          }
        }
      }
    }
    out
  }

  pub fn collect_physical_descendants(&self, dir: &InternedPath) -> Vec<InternedPath> {
    self
      .links
      .read()
      .expect("links lock poisoned")
      .collect_physical_descendants(dir)
  }

  pub fn check_physical_directory_replaced(&self, physical_dir: &InternedPath) -> bool {
    if !self
      .records
      .read()
      .expect("records lock poisoned")
      .directory_identities
      .contains_key(physical_dir)
    {
      return false;
    }
    let Ok(meta) = physical_dir.metadata() else {
      return false;
    };
    if !meta.is_dir() {
      return false;
    }
    let Some(new_id) = directory_identity(physical_dir.as_path(), &meta) else {
      return false;
    };
    match self
      .records
      .write()
      .expect("records lock poisoned")
      .directory_identities
      .insert(physical_dir.clone(), new_id)
    {
      Some(old_id) => old_id != new_id,
      None => false,
    }
  }

  pub async fn scan_contexts_with_plan(
    &self,
    start_time: SystemTime,
    contexts: &[(InternedPath, bool)],
    post_attach: bool,
  ) -> ContextScanOutput {
    let since_ms = system_time_to_millis(start_time);
    let mut output = ContextScanOutput::default();
    for (root_ctx, is_rescan) in contexts {
      if !self.directories.all.contains(root_ctx) {
        continue;
      }
      let sub_out = self
        .scan_root_context(root_ctx, *is_rescan, Some(since_ms), post_attach)
        .await;
      if sub_out.added_watched_paths {
        output.attached_contexts.push(root_ctx.clone());
      }
      output.changed_entries.extend(sub_out.changed_entries);
      output.added_watched_paths |= sub_out.added_watched_paths;
    }
    output
  }

  async fn scan_root_context(
    &self,
    root_ctx: &InternedPath,
    recovered_or_retargeted: bool,
    changed_since: Option<u64>,
    post_attach: bool,
  ) -> ContextScanOutput {
    let mut output = ContextScanOutput::default();
    let (physical_root, filter_root, active) = {
      let links = self.links.read().expect("links lock poisoned");
      if let Some(bound) = links.get_bound(DepKind::Directory, root_ctx) {
        let filter_prefix = if self.follow_symlinks {
          bound.target.clone()
        } else {
          root_ctx.clone()
        };
        (bound.target.clone(), filter_prefix, bound.target_active)
      } else {
        (root_ctx.clone(), root_ctx.clone(), true)
      }
    };
    if !active {
      return output;
    }

    if let Ok(meta) = physical_root.metadata()
      && meta.is_dir()
    {
      let id_opt = directory_identity(physical_root.as_path(), &meta);
      let mut records = self.records.write().expect("records lock poisoned");
      records.absent_directories.remove(root_ctx);
      if let Some(id) = id_opt {
        records
          .directory_identities
          .insert(physical_root.clone(), id);
        records.directory_identities.insert(root_ctx.clone(), id);
      }
    } else {
      self
        .records
        .write()
        .expect("records lock poisoned")
        .absent_directories
        .insert(root_ctx.clone());
      return output;
    }

    self
      .scan_subtree(
        root_ctx,
        physical_root,
        root_ctx.clone(),
        filter_root,
        recovered_or_retargeted,
        changed_since,
        post_attach,
        &mut output,
      )
      .await;
    output
  }

  #[inline(never)]
  #[allow(clippy::too_many_arguments)]
  fn scan_subtree<'a>(
    &'a self,
    root_ctx: &'a InternedPath,
    start_physical: InternedPath,
    start_record: InternedPath,
    start_filter: InternedPath,
    initial_observed: bool,
    changed_since: Option<u64>,
    post_attach: bool,
    output: &'a mut ContextScanOutput,
  ) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
    Box::pin(async move {
      let mut resolver = LinkResolver::new(true);
      let mut pending: Vec<(InternedPath, InternedPath, InternedPath, Vec<PathBuf>, bool)> =
        vec![(
          start_physical,
          start_record,
          start_filter,
          Vec::new(),
          initial_observed,
        )];

      while let Some((phys_dir, record_dir, filter_dir, mut ancestry, observed)) = pending.pop() {
        if !self.directories.all.contains(root_ctx) {
          break;
        }
        if self.follow_symlinks {
          let Ok(real) = dunce::canonicalize(&phys_dir) else {
            continue;
          };
          if ancestry.contains(&real) {
            continue;
          }
          ancestry.push(real);
        }

        if let Ok(dir_meta) = phys_dir.metadata()
          && let Some(id) = directory_identity(phys_dir.as_path(), &dir_meta)
        {
          let mut records = self.records.write().expect("records lock poisoned");
          records.directory_identities.insert(phys_dir.clone(), id);
          records.directory_identities.insert(record_dir.clone(), id);
        }

        let Ok(entries) = std::fs::read_dir(&phys_dir) else {
          continue;
        };

        let mut seen_record_entries = InternedPathSet::default();
        let mut dir_watches_batch: Vec<(InternedPath, Option<ContextMapping>)> = Vec::new();
        let mut scanned_links_batch: Vec<InternedPath> = Vec::new();

        for entry in entries.flatten() {
          let entry_name = entry.file_name();
          let phys_entry = InternedPath::from(phys_dir.join(&entry_name));
          let record_entry = InternedPath::from(record_dir.join(&entry_name));
          let filter_entry = InternedPath::from(filter_dir.join(&entry_name));
          seen_record_entries.insert(record_entry.clone());

          if self.is_ignored_path(filter_entry.as_path()).await {
            continue;
          }
          if !self.directories.all.contains(root_ctx) {
            break;
          }

          let Ok(own_meta) = std::fs::symlink_metadata(&phys_entry) else {
            continue;
          };

          let is_symlink = own_meta.file_type().is_symlink();
          if self.follow_symlinks && is_symlink {
            scanned_links_batch.push(phys_entry.clone());
          }

          let mut followed_external_dir: Option<(InternedPath, std::fs::Metadata)> = None;
          let mut external_file_target: Option<InternedPath> = None;

          if self.follow_symlinks && is_symlink {
            let resolved_target = resolver.resolve(&phys_entry).target;
            if let Ok(canonical_target) = dunce::canonicalize(&phys_entry) {
              if let Some((_, target_meta)) = Self::classify_symlink_target(
                ancestry.last().map(PathBuf::as_path),
                phys_entry.as_path(),
                record_entry.as_path(),
                canonical_target,
              ) {
                if target_meta.is_dir() {
                  followed_external_dir = Some((resolved_target, target_meta));
                } else if target_meta.is_file() {
                  external_file_target = Some(resolved_target);
                }
              }
            } else if !record_entry
              .as_path()
              .starts_with(resolved_target.as_path())
              && resolved_target != phys_entry
              && !self.is_ignored_path(resolved_target.as_path()).await
            {
              dir_watches_batch.push((
                resolved_target,
                Some(ContextMapping {
                  root_context: root_ctx.clone(),
                  record_prefix: record_entry.clone(),
                  filter_prefix: filter_entry.clone(),
                }),
              ));
            }
          }

          if let Some((effective_dir, effective_meta, is_external_link)) = followed_external_dir
            .map(|(d, m)| (d, m, true))
            .or_else(|| {
              own_meta
                .is_dir()
                .then(|| (phys_entry.clone(), own_meta.clone(), false))
            })
          {
            let was_known_dir =
              self.record_context_dir_owners(&record_entry, std::slice::from_ref(root_ctx));
            if self.forget_if_orphaned(&record_entry) {
              break;
            }
            if let Ok(mtime) = effective_meta.modified() {
              mtime_accuracy(mtime);
            }
            let alias_mapping =
              (is_external_link || effective_dir != record_entry || filter_entry != record_entry)
                .then(|| ContextMapping {
                  root_context: root_ctx.clone(),
                  record_prefix: record_entry.clone(),
                  filter_prefix: filter_entry.clone(),
                });
            if cfg!(any(
              feature = "testing",
              not(any(target_os = "macos", target_os = "windows"))
            )) || alias_mapping.is_some()
            {
              dir_watches_batch.push((effective_dir.clone(), alias_mapping));
            }
            if (!post_attach || !was_known_dir)
              && changed_since.is_some_and(|since| {
                effective_meta
                  .created()
                  .or_else(|_| effective_meta.modified())
                  .or_else(|_| own_meta.created())
                  .is_ok_and(|birthtime| mtime_safe_time(birthtime) >= since)
              })
            {
              output
                .changed_entries
                .push((filter_entry.clone(), root_ctx.clone()));
            }
            pending.push((
              effective_dir,
              record_entry,
              filter_entry,
              ancestry.clone(),
              observed,
            ));
          } else if let Ok(mtime) = own_meta.modified().or_else(|_| own_meta.created()) {
            let had_context_dir = self
              .records
              .read()
              .expect("records lock poisoned")
              .context_directories
              .contains_key(&record_entry);
            if had_context_dir {
              self.remove_file_times_under(&record_entry);
              let mut evicted = Vec::new();
              {
                let mut links = self.links.write().expect("links lock poisoned");
                links.clear_context_entries_under_prefix(
                  record_entry.as_path(),
                  self.watched_trackers(),
                  &mut evicted,
                );
              }
              self.prune_evicted_physical_states(&evicted);
            }
            let written = {
              let mut records = self.records.write().expect("records lock poisoned");
              records.record(&record_entry, |rec| {
                let owners = rec.context_files.entry(record_entry.clone()).or_default();
                push_unique(owners, root_ctx.clone());
                rec.write_file_time(&record_entry, mtime, !(observed || post_attach), true)
              })
            };
            let active_registered = (self.files.all.contains(&record_entry)
              && self
                .bound_dep(DepKind::File, &record_entry)
                .is_none_or(|b| b.target_active))
              || (self.missing.all.contains(&record_entry)
                && self
                  .bound_dep(DepKind::Missing, &record_entry)
                  .is_none_or(|b| b.target_active));
            if self.forget_if_orphaned(&record_entry) {
              break;
            }
            let mut ext_changed = false;
            if let Some(ext_target) = external_file_target
              && !self.is_ignored_path(ext_target.as_path()).await
            {
              if let Some(tm) = disk_mtime(&ext_target) {
                if observed || post_attach {
                  let prev_tm = self
                    .records
                    .read()
                    .expect("records lock poisoned")
                    .baselines
                    .get(&ext_target)
                    .and_then(|b| b.mtime);
                  let baseline_differs = if post_attach {
                    Some(tm) != prev_tm
                  } else {
                    prev_tm.is_some_and(|old| old != tm)
                  };
                  let newer_than_since =
                    changed_since.is_some_and(|since| mtime_safe_time(tm) >= since);
                  if baseline_differs || newer_than_since {
                    ext_changed = true;
                    self.set_file_time(&record_entry, tm, false, false);
                  }
                }
                self
                  .records
                  .write()
                  .expect("records lock poisoned")
                  .baselines
                  .insert(
                    ext_target.clone(),
                    PhysicalBaseline {
                      mtime: Some(tm),
                      link: None,
                    },
                  );
              } else {
                self.clear_physical_baseline(&ext_target);
              }
              let added = {
                let mut links = self.links.write().expect("links lock poisoned");
                links.add_context_file_watch(
                  root_ctx,
                  ext_target,
                  record_entry.clone(),
                  self.watched_trackers(),
                )
              };
              output.added_watched_paths |= added;
            }
            let own_changed = if post_attach {
              written
            } else {
              (observed && written)
                || changed_since.is_some_and(|since| mtime_safe_time(mtime) >= since)
            };
            if !active_registered && (own_changed || ext_changed) {
              output
                .changed_entries
                .push((filter_entry, root_ctx.clone()));
            }
          }
        }

        if (!scanned_links_batch.is_empty() || !dir_watches_batch.is_empty())
          && self.directories.all.contains(root_ctx)
        {
          let trackers = self.watched_trackers();
          let mut links = self.links.write().expect("links lock poisoned");
          for link in scanned_links_batch {
            links.record_context_scanned_link(root_ctx, link);
          }
          for (dir, mapping) in dir_watches_batch {
            output.added_watched_paths |=
              links.add_context_dir_watch(root_ctx, dir, mapping, trackers);
          }
        }

        let prev_children: Vec<InternedPath> = {
          let records = self.records.read().expect("records lock poisoned");
          records
            .recorded_children
            .get(&record_dir)
            .map_or_default(|set| set.iter().cloned().collect())
        };
        for prev_child in prev_children {
          if seen_record_entries.contains(&prev_child) {
            continue;
          }
          let owned_by_root = {
            let records = self.records.read().expect("records lock poisoned");
            records
              .context_files
              .get(&prev_child)
              .is_some_and(|o| o.contains(root_ctx))
              || records
                .context_directories
                .get(&prev_child)
                .is_some_and(|o| o.contains(root_ctx))
          };
          if !owned_by_root {
            continue;
          }
          self.remove_owner_from_subtree(&prev_child, root_ctx);
          if (observed || post_attach)
            && let Ok(rel) = prev_child.strip_prefix(&record_dir)
          {
            output
              .changed_entries
              .push((InternedPath::from(filter_dir.join(rel)), root_ctx.clone()));
          }
        }
      }
    })
  }

  fn has_any_active_context_owner_in(&self, records: &RecordState, path: &InternedPath) -> bool {
    if let Some(owners) = records.context_files.get(path)
      && owners.iter().any(|o| self.directories.all.contains(o))
    {
      return true;
    }
    if let Some(owners) = records.context_directories.get(path)
      && owners.iter().any(|o| self.directories.all.contains(o))
    {
      return true;
    }
    false
  }

  fn forget_if_orphaned(&self, path: &InternedPath) -> bool {
    let phys_watched = self
      .links
      .read()
      .expect("links lock poisoned")
      .is_physically_watched_dir(path);
    let mut records = self.records.write().expect("records lock poisoned");
    if self.has_any_active_context_owner_in(&records, path) {
      return false;
    }
    records.context_files.remove(path);
    records.context_directories.remove(path);
    if !self.files.all.contains(path) && !self.missing.all.contains(path) {
      records.file_times.remove(path);
    }
    if !self.directories.all.contains(path) {
      records.last_watch_events.remove(path);
      if !phys_watched {
        records.directory_identities.remove(path);
      }
    }
    records.unlink_unrecorded(path);
    true
  }

  fn remove_owner_from_subtree(&self, root_entry: &InternedPath, owner: &InternedPath) {
    let active_phys_dirs = self.active_physically_watched_dirs();
    let mut records = self.records.write().expect("records lock poisoned");
    let mut subtree = Vec::new();
    let mut pending = vec![root_entry.clone()];
    while let Some(path) = pending.pop() {
      if let Some(children) = records.recorded_children.get(&path) {
        pending.extend(children.iter().cloned());
      }
      subtree.push(path);
    }
    for path in &subtree {
      let file_emptied = if let Some(owners) = records.context_files.get_mut(path) {
        if let Some(pos) = owners.iter().position(|o| o == owner) {
          owners.remove(pos);
        }
        owners.is_empty()
      } else {
        false
      };
      if file_emptied {
        records.context_files.remove(path);
        if !self.files.all.contains(path) && !self.missing.all.contains(path) {
          records.file_times.remove(path);
          records.unlink_unrecorded(path);
        }
      }

      let dir_emptied = if let Some(owners) = records.context_directories.get_mut(path) {
        if let Some(pos) = owners.iter().position(|o| o == owner) {
          owners.remove(pos);
        }
        owners.is_empty()
      } else {
        false
      };
      if dir_emptied {
        records.context_directories.remove(path);
        if !self.directories.all.contains(path) {
          records.last_watch_events.remove(path);
          if !active_phys_dirs.contains(path) {
            records.directory_identities.remove(path);
          }
          records.unlink_unrecorded(path);
        }
      }
    }
  }

  pub fn mark_context_target_absent(&self, root_ctx: &InternedPath) {
    self.forget_context_entries_owned_by(root_ctx);
    self
      .records
      .write()
      .expect("records lock poisoned")
      .absent_directories
      .insert(root_ctx.clone());
  }

  fn forget_context_entries_owned_by(&self, dir: &InternedPath) {
    self.remove_owner_from_subtree(dir, dir);
  }

  pub fn clear_context_for_rebind(&self, root_ctx: &InternedPath) {
    let mut evicted = Vec::new();
    {
      let mut links = self.links.write().expect("links lock poisoned");
      links.clear_context_scan(root_ctx, self.watched_trackers(), &mut evicted);
    }
    self.prune_evicted_physical_states(&evicted);
    self.forget_context_entries_owned_by(root_ctx);
  }

  pub async fn recover_absent_contexts_under(&self, dir_path: &InternedPath) -> bool {
    let dir_std = dir_path.as_ref() as &Path;
    let absent_snapshot: Vec<InternedPath> = self
      .records
      .read()
      .expect("records lock poisoned")
      .absent_directories
      .iter()
      .cloned()
      .collect();
    if absent_snapshot.is_empty() {
      return false;
    }
    let recovered: Vec<InternedPath> = {
      let links = self.links.read().expect("links lock poisoned");
      absent_snapshot
        .into_iter()
        .filter(|context| {
          if !context.is_dir() {
            return false;
          }
          if context.starts_with(dir_std) {
            return true;
          }
          links
            .get_bound(DepKind::Directory, context)
            .is_some_and(|b| b.target_active && b.target.starts_with(dir_std) && b.target.is_dir())
        })
        .collect()
    };
    let mut added_watched = false;
    for context in &recovered {
      let out = self.scan_root_context(context, true, None, false).await;
      added_watched |= out.added_watched_paths;
    }
    added_watched
  }

  fn classify_symlink_target(
    real_parent: Option<&Path>,
    phys_entry: &Path,
    logical_entry: &Path,
    canonical_target: PathBuf,
  ) -> Option<(InternedPath, std::fs::Metadata)> {
    let into_own_dir = real_parent.is_some_and(|rp| canonical_target.parent() == Some(rp));
    let above_itself = real_parent.is_some_and(|rp| {
      phys_entry
        .file_name()
        .is_some_and(|name| rp.join(name).starts_with(&canonical_target))
    }) || logical_entry.starts_with(&canonical_target);
    if into_own_dir || above_itself {
      return None;
    }
    let meta = std::fs::metadata(&canonical_target).ok()?;
    Some((InternedPath::from(canonical_target), meta))
  }

  fn record_context_dir_owners(&self, path: &InternedPath, owners: &[InternedPath]) -> bool {
    let mut records = self.records.write().expect("records lock poisoned");
    records.record(path, |rec| {
      rec.context_files.remove(path);
      if !self.files.all.contains(path) && !self.missing.all.contains(path) {
        rec.file_times.remove(path);
      }
      let existed = rec.context_directories.contains_key(path);
      let entry = rec.context_directories.entry(path.clone()).or_default();
      for owner in owners {
        push_unique(entry, owner.clone());
      }
      existed
    })
  }

  pub async fn observe_live_context_entry(
    &self,
    physical_path: &InternedPath,
    own_meta_in: Option<&std::fs::Metadata>,
    logical_path: &InternedPath,
    owners: &[InternedPath],
    flags: LiveEntryFlags,
  ) -> (ContextEntryOutcome, bool) {
    let LiveEntryFlags {
      judged,
      is_disk,
      renamed,
    } = flags;
    let mut added_watched = self.recover_absent_contexts_under(logical_path).await;

    let registered_directory = self.directories.all.contains(logical_path);
    if !registered_directory && owners.is_empty() {
      return (ContextEntryOutcome::Changed, added_watched);
    }

    let meta_storage;
    let own_meta = match own_meta_in {
      Some(meta) => meta,
      None => {
        let Ok(meta) = std::fs::symlink_metadata(physical_path) else {
          return (ContextEntryOutcome::Changed, added_watched);
        };
        meta_storage = meta;
        &meta_storage
      }
    };

    let is_symlink = own_meta.file_type().is_symlink();
    let mut external_dir_target: Option<(InternedPath, std::fs::Metadata)> = None;
    let mut is_external_file_link = false;

    if self.follow_symlinks
      && is_symlink
      && let Some(real_parent) = physical_path
        .parent()
        .and_then(|p| dunce::canonicalize(p).ok())
      && let Ok(canonical_target) = dunce::canonicalize(physical_path)
      && let Some((_, target_meta)) = Self::classify_symlink_target(
        Some(real_parent.as_path()),
        physical_path.as_path(),
        logical_path.as_path(),
        canonical_target,
      )
    {
      if target_meta.is_dir() {
        let resolved_target = LinkResolver::new(true).resolve(physical_path).target;
        external_dir_target = Some((resolved_target, target_meta));
      } else if target_meta.is_file() {
        is_external_file_link = true;
      }
    }

    if let Some((effective_dir, effective_meta, is_external_link)) =
      external_dir_target.map(|(d, m)| (d, m, true)).or_else(|| {
        own_meta
          .is_dir()
          .then(|| (physical_path.clone(), own_meta.clone(), false))
      })
    {
      if let Ok(mtime) = effective_meta.modified() {
        mtime_accuracy(mtime);
      }
      if registered_directory && (!is_external_link || owners.is_empty()) {
        if !owners.is_empty() {
          let _ = self.record_context_dir_owners(logical_path, owners);
        }
        let outcome = if owners.is_empty() || (is_disk && !renamed) {
          ContextEntryOutcome::ExistingDirectory
        } else {
          ContextEntryOutcome::Changed
        };
        return (outcome, added_watched);
      }

      let link_changed =
        is_external_link && self.check_context_link_changed(owners, physical_path, is_disk);
      let new_id = directory_identity(effective_dir.as_path(), &effective_meta);
      let prev_id = new_id.and_then(|id| {
        self
          .records
          .write()
          .expect("records lock poisoned")
          .directory_identities
          .insert(logical_path.clone(), id)
      });
      let replaced = prev_id.zip(new_id).is_some_and(|(old, cur)| old != cur);
      if replaced {
        self.remove_file_times_under(logical_path);
        if is_external_link {
          let mut evicted = Vec::new();
          {
            let mut links = self.links.write().expect("links lock poisoned");
            links.clear_context_entries_under_prefix(
              logical_path.as_path(),
              self.watched_trackers(),
              &mut evicted,
            );
          }
          self.prune_evicted_physical_states(&evicted);
        }
      }
      let was_known = self.record_context_dir_owners(logical_path, owners);
      if !was_known || replaced || link_changed {
        for owner in owners {
          if is_external_link {
            self
              .records
              .write()
              .expect("records lock poisoned")
              .pending_context_rescan
              .insert(owner.clone());
          }
          let start_filter = self.filter_path_for_context(owner, logical_path);
          let mut out = ContextScanOutput::default();
          self
            .scan_subtree(
              owner,
              effective_dir.clone(),
              logical_path.clone(),
              start_filter,
              replaced,
              None,
              false,
              &mut out,
            )
            .await;
          added_watched |= out.added_watched_paths;
        }
      } else if is_external_link {
        let mut records = self.records.write().expect("records lock poisoned");
        for owner in owners {
          records.pending_context_rescan.insert(owner.clone());
        }
      }

      let outcome = if !link_changed && was_known && !replaced && is_disk && !renamed {
        ContextEntryOutcome::ExistingDirectory
      } else {
        ContextEntryOutcome::Changed
      };
      return (outcome, added_watched);
    }

    if registered_directory && owners.is_empty() {
      return (ContextEntryOutcome::Changed, added_watched);
    }

    let had_context_dir = self
      .records
      .read()
      .expect("records lock poisoned")
      .context_directories
      .contains_key(logical_path);
    if had_context_dir {
      self.remove_file_times_under(logical_path);
      let mut evicted = Vec::new();
      {
        let mut links = self.links.write().expect("links lock poisoned");
        links.clear_context_entries_under_prefix(
          logical_path.as_path(),
          self.watched_trackers(),
          &mut evicted,
        );
      }
      self.prune_evicted_physical_states(&evicted);
    }
    if is_external_file_link {
      let mut records = self.records.write().expect("records lock poisoned");
      for owner in owners {
        records.pending_context_rescan.insert(owner.clone());
      }
    }

    {
      let mut records = self.records.write().expect("records lock poisoned");
      records.record(logical_path, |rec| {
        let entry = rec.context_files.entry(logical_path.clone()).or_default();
        for owner in owners {
          push_unique(entry, owner.clone());
        }
      });
    }
    let Ok(mtime) = own_meta.modified().or_else(|_| own_meta.created()) else {
      return (ContextEntryOutcome::Changed, added_watched);
    };
    if judged {
      return (ContextEntryOutcome::Changed, added_watched);
    };
    let mtime_ms = mtime_to_millis(mtime);
    let accuracy = mtime_accuracy(mtime);
    let untouched = self
      .records
      .read()
      .expect("records lock poisoned")
      .file_times
      .get(logical_path)
      .is_some_and(|old| old.mtime == mtime && mtime_ms + accuracy < current_time());
    if untouched {
      return (ContextEntryOutcome::Unchanged, added_watched);
    }
    self.set_file_time(logical_path, mtime, false, false);
    (ContextEntryOutcome::Changed, added_watched)
  }

  pub async fn set_context_entry(&self, path: &InternedPath, judged: bool) -> bool {
    let owners: Vec<InternedPath> = path
      .ancestors()
      .skip(1)
      .map(InternedPath::from)
      .filter(|a| self.directories.all.contains(a))
      .collect();
    !matches!(
      self
        .observe_live_context_entry(
          path,
          None,
          path,
          &owners,
          LiveEntryFlags {
            judged,
            is_disk: false,
            renamed: false,
          },
        )
        .await
        .0,
      ContextEntryOutcome::Unchanged
    )
  }

  pub fn may_contain_records(&self, path: &InternedPath) -> bool {
    if self.directories.all.contains(path) {
      return true;
    }
    let records = self.records.read().expect("records lock poisoned");
    records.context_directories.contains_key(path)
      || records.registered_ancestors.contains_key(path)
      || records.recorded_children.contains_key(path)
  }

  fn is_below_context(&self, path: &InternedPath) -> bool {
    {
      let records = self.records.read().expect("records lock poisoned");
      if self.has_any_active_context_owner_in(&records, path) {
        return true;
      }
    }
    path
      .ancestors()
      .skip(1)
      .any(|ancestor| self.directories.all.contains(&InternedPath::from(ancestor)))
  }

  pub fn set_file_time(
    &self,
    path: &InternedPath,
    mtime: SystemTime,
    initial: bool,
    ignore_when_equal: bool,
  ) -> bool {
    let mut records = self.records.write().expect("records lock poisoned");
    records.record(path, |rec| {
      rec.write_file_time(path, mtime, initial, ignore_when_equal)
    })
  }

  pub fn set_registered_file_time(
    &self,
    path: &InternedPath,
    mtime: SystemTime,
    initial: bool,
    ignore_when_equal: bool,
  ) -> bool {
    let mut records = self.records.write().expect("records lock poisoned");
    if !self.files.all.contains(path) && !self.missing.all.contains(path) {
      return false;
    }
    records.record(path, |rec| {
      rec.write_file_time(path, mtime, initial, ignore_when_equal)
    })
  }

  pub fn set_last_watch_events(&self, path: &InternedPath) {
    let now = current_time();
    let mut records = self.records.write().expect("records lock poisoned");
    for ancestor in path.ancestors() {
      let ancestor = InternedPath::from(ancestor);
      if self.directories.all.contains(&ancestor)
        || records.context_directories.contains_key(&ancestor)
      {
        records.record(&ancestor, |rec| {
          rec.last_watch_events.insert(ancestor.clone(), now);
        });
      }
    }
  }

  pub fn set_last_watch_events_for_context(
    &self,
    logical_entry: &InternedPath,
    root_context: &InternedPath,
  ) {
    let now = current_time();
    let mut records = self.records.write().expect("records lock poisoned");
    if !self.directories.all.contains(root_context) {
      return;
    }
    records.record(root_context, |rec| {
      rec.last_watch_events.insert(root_context.clone(), now);
    });
    for ancestor in logical_entry.ancestors() {
      if !ancestor.starts_with(root_context.as_path()) {
        break;
      }
      let ancestor = InternedPath::from(ancestor);
      if ancestor == *root_context || records.context_directories.contains_key(&ancestor) {
        records.record(&ancestor, |rec| {
          rec.last_watch_events.insert(ancestor.clone(), now);
        });
      }
    }
  }

  #[cfg(test)]
  pub fn has_mtime_changed(&self, path: &InternedPath) -> bool {
    if !self.files.all.contains(path) && !self.missing.all.contains(path) {
      return true;
    }
    self.has_physical_mtime_changed(path, path)
  }

  pub fn advance_physical_baseline(&self, physical: &InternedPath) -> Option<SystemTime> {
    let link = physical
      .is_symlink()
      .then(|| LinkState::read(physical.as_path()));
    let current_mtime = disk_mtime(physical);
    let Some(mtime) = current_mtime else {
      if let Some(ref cur_link) = link {
        let prev = self
          .records
          .write()
          .expect("records lock poisoned")
          .baselines
          .insert(
            physical.clone(),
            PhysicalBaseline {
              mtime: None,
              link: link.clone(),
            },
          );
        if prev.as_ref().and_then(|p| p.link.as_ref()) != Some(cur_link) {
          return std::fs::symlink_metadata(physical)
            .and_then(|m| m.modified().or_else(|_| m.created()))
            .ok();
        }
        return None;
      }
      return None;
    };

    let mtime_ms = mtime_to_millis(mtime);
    let next_baseline = PhysicalBaseline {
      mtime: Some(mtime),
      link,
    };
    let prev = self
      .records
      .write()
      .expect("records lock poisoned")
      .baselines
      .insert(physical.clone(), next_baseline.clone());
    if prev.as_ref() == Some(&next_baseline)
      && mtime_ms.saturating_add(mtime_accuracy(mtime)) < current_time()
    {
      return None;
    }
    Some(mtime)
  }

  pub fn has_physical_mtime_changed(
    &self,
    physical: &InternedPath,
    logical: &InternedPath,
  ) -> bool {
    let Some(mtime) = self.advance_physical_baseline(physical) else {
      return disk_mtime(physical).is_none() && !physical.is_symlink();
    };
    let logical_mtime = disk_mtime(logical).unwrap_or(mtime);
    self.set_file_time(logical, logical_mtime, false, false);
    true
  }

  pub async fn is_ignored_registered_subscriber(
    &self,
    phys: &InternedPath,
    logical_path: &InternedPath,
    is_link_hop: bool,
  ) -> bool {
    if !self.follow_symlinks {
      return self.is_ignored_path(logical_path.as_path()).await;
    }
    self.is_ignored_path(phys.as_path()).await
      || (phys != logical_path
        && !is_link_hop
        && self.is_ignored_path(logical_path.as_path()).await)
  }

  pub async fn is_ignored_context_chain(&self, filter_root: &Path, filter_entry: &Path) -> bool {
    if self.is_ignored_path(filter_entry).await {
      return true;
    }
    let Ok(rel) = filter_entry.strip_prefix(filter_root) else {
      return false;
    };
    let mut curr = filter_root.to_path_buf();
    let mut comps = rel.components().peekable();
    while let Some(comp) = comps.next() {
      if comps.peek().is_none() {
        break;
      }
      curr.push(comp.as_os_str());
      if self.is_ignored_path(&curr).await {
        return true;
      }
    }
    false
  }

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
  ) -> Result<ScanPlan> {
    self
      .update_vecs(
        (files.0.collect(), files.1.collect()),
        (directories.0.collect(), directories.1.collect()),
        (missing.0.collect(), missing.1.collect()),
      )
      .await
  }

  #[inline(never)]
  async fn update_vecs(
    &self,
    files: (Vec<InternedPath>, Vec<InternedPath>),
    directories: (Vec<InternedPath>, Vec<InternedPath>),
    missing: (Vec<InternedPath>, Vec<InternedPath>),
  ) -> Result<ScanPlan> {
    let (added_files, removed_files_reg) = PathUpdater::from_vecs(files)
      .update(&self.files, &self.ignored)
      .await?;
    let (added_dirs, removed_dirs_reg) = PathUpdater::from_vecs(directories)
      .update(&self.directories, &self.ignored)
      .await?;
    if !added_dirs.is_empty() {
      let mut records = self.records.write().expect("records lock poisoned");
      for dir in &added_dirs {
        if !dir.exists() {
          records.absent_directories.insert(dir.clone());
        } else {
          records.absent_directories.remove(dir);
        }
      }
    }
    let (added_missing, removed_missing_reg) = PathUpdater::from_vecs(missing)
      .update(&self.missing, &self.ignored)
      .await?;

    {
      let mut records = self.records.write().expect("records lock poisoned");
      for path in added_files.iter().chain(&added_dirs).chain(&added_missing) {
        records.retain_registered_ancestors(path);
      }
      for path in removed_files_reg
        .iter()
        .chain(&removed_dirs_reg)
        .chain(&removed_missing_reg)
      {
        records.release_registered_ancestors(path);
      }
    }

    let removed_files: Vec<InternedPath> = self.files.removed.iter().map(|p| p.clone()).collect();
    let removed_missing: Vec<InternedPath> =
      self.missing.removed.iter().map(|p| p.clone()).collect();
    let removed_dirs: Vec<InternedPath> =
      self.directories.removed.iter().map(|p| p.clone()).collect();

    let mut evicted = Vec::new();
    {
      let mut links = self.links.write().expect("links lock poisoned");
      let trackers = self.watched_trackers();
      for (kind, active_set, removed_list) in [
        (DepKind::File, &self.files.all, &removed_files),
        (DepKind::Missing, &self.missing.all, &removed_missing),
      ] {
        for path in removed_list {
          if !active_set.contains(path) {
            links.unbind_dep(kind, path, trackers, &mut evicted);
          }
        }
      }
      for dir in &removed_dirs {
        if !self.directories.all.contains(dir) {
          links.clear_context_scan(dir, trackers, &mut evicted);
          links.unbind_dep(DepKind::Directory, dir, trackers, &mut evicted);
        }
      }
    }

    for path in removed_files.iter().chain(&removed_missing) {
      let still_file = self.files.all.contains(path);
      let still_missing = self.missing.all.contains(path);
      if still_file && still_missing {
        continue;
      }
      let below_ctx = self.is_below_context(path);
      let mut records = self.records.write().expect("records lock poisoned");
      if !still_file && !still_missing {
        records.baselines.remove(path);
        if !below_ctx {
          records.file_times.remove(path);
          records.unlink_unrecorded(path);
        }
      } else if !still_file && !below_ctx {
        records.file_times.remove(path);
      }
    }
    for dir in &removed_dirs {
      if self.directories.all.contains(dir) {
        continue;
      }
      self.forget_context_entries_owned_by(dir);
      let phys_watched = self
        .links
        .read()
        .expect("links lock poisoned")
        .is_physically_watched_dir(dir);
      let mut records = self.records.write().expect("records lock poisoned");
      let absent = records.absent_directories.remove(dir);
      records.pending_context_rescan.remove(dir);
      if absent || !self.has_any_active_context_owner_in(&records, dir) {
        records.last_watch_events.remove(dir);
        if !phys_watched {
          records.directory_identities.remove(dir);
        }
        records.unlink_unrecorded(dir);
      }
    }
    self.prune_evicted_physical_states(&evicted);
    evicted.clear();

    let mut files_set: InternedPathSet = self.files.added.iter().map(|p| p.clone()).collect();
    let mut newly_added_set: InternedPathSet = files_set.clone();
    let mut missing_set: InternedPathSet = self.missing.added.iter().map(|p| p.clone()).collect();
    let directories_added: Vec<InternedPath> =
      self.directories.added.iter().map(|p| p.clone()).collect();
    let mut replaced_physical_dirs = Vec::new();

    let mut rebind_candidates: [InternedPathSet; 3] = Default::default();
    let mut context_rescans: InternedPathSet = {
      let mut records = self.records.write().expect("records lock poisoned");
      std::mem::take(&mut records.pending_context_rescan)
        .into_iter()
        .filter(|ctx| self.directories.all.contains(ctx))
        .collect()
    };

    let root_dirs_snapshot: Vec<InternedPath> = self
      .links
      .read()
      .expect("links lock poisoned")
      .root_dir_refs
      .keys()
      .cloned()
      .collect();
    let mut replaced_roots = InternedPathSet::default();
    for root_dir in root_dirs_snapshot {
      if let Ok(meta) = root_dir.metadata()
        && meta.is_dir()
        && let Some(new_id) = directory_identity(root_dir.as_path(), &meta)
      {
        let prev = self
          .records
          .write()
          .expect("records lock poisoned")
          .directory_identities
          .insert(root_dir.clone(), new_id);
        if prev.is_some_and(|old_id| old_id != new_id) {
          replaced_roots.insert(root_dir.clone());
          replaced_physical_dirs.push(root_dir);
        }
      }
    }

    {
      let mut links = self.links.write().expect("links lock poisoned");
      for kind in DepKind::ALL {
        let idx = kind as usize;
        rebind_candidates[idx].extend(std::mem::take(&mut links.dirty_deps[idx]));
      }

      let changed_links: InternedPathSet = links
        .link_states
        .iter()
        .filter(|(link, old)| **old != LinkState::read(link.as_path()))
        .map(|(link, _)| link.clone())
        .collect();
      let appeared_heads: InternedPathSet = links
        .pending_missing_heads
        .keys()
        .filter(|head| std::fs::symlink_metadata(head).is_ok())
        .cloned()
        .collect();

      if !changed_links.is_empty() || !appeared_heads.is_empty() || !replaced_roots.is_empty() {
        for kind in DepKind::ALL {
          for (logical, bound) in &links.bound_deps[kind as usize] {
            let hit = bound.links.iter().any(|l| changed_links.contains(l))
              || bound
                .first_missing
                .as_ref()
                .is_some_and(|h| appeared_heads.contains(h))
              || replaced_roots
                .iter()
                .any(|r| bound.target.starts_with(r.as_path()));
            if hit {
              rebind_candidates[kind as usize].insert(logical.clone());
            }
          }
        }
      }

      if !replaced_roots.is_empty() {
        for f in self.files.all.iter() {
          if replaced_roots.iter().any(|r| f.starts_with(r.as_path())) {
            files_set.insert(f.clone());
          }
        }
        for m in self.missing.all.iter() {
          if replaced_roots.iter().any(|r| m.starts_with(r.as_path())) {
            missing_set.insert(m.clone());
          }
        }
        for d in self.directories.all.iter() {
          if replaced_roots.iter().any(|r| d.starts_with(r.as_path())) {
            context_rescans.insert(d.clone());
          }
        }
        for (phys_dir, mappings) in &links.dir_aliases {
          if replaced_roots
            .iter()
            .any(|r| phys_dir.starts_with(r.as_path()))
          {
            for m in mappings {
              context_rescans.insert(m.root_context.clone());
            }
          }
        }
      }

      if self.follow_symlinks {
        for kind in [DepKind::Directory, DepKind::Missing] {
          for (logical, bound) in &links.bound_deps[kind as usize] {
            let dangling = match kind {
              DepKind::Directory => !bound.target.is_dir(),
              DepKind::Missing => bound.first_missing.is_some() || !bound.target.exists(),
              DepKind::File => false,
            };
            if dangling && (!bound.links.is_empty() || bound.first_missing.is_some()) {
              rebind_candidates[kind as usize].insert(logical.clone());
            }
          }
        }
        for (root_ctx, bindings) in &links.reverse_context_scans {
          if bindings
            .scanned_links
            .iter()
            .any(|(l, old)| *old != LinkState::read(l.as_path()))
          {
            context_rescans.insert(root_ctx.clone());
          }
        }
      }
    }

    let mut resolver = LinkResolver::new(self.follow_symlinks);
    let mut seen_bind: [InternedPathSet; 3] = Default::default();
    let mut to_bind: Vec<(DepKind, InternedPath, bool)> = Vec::new();
    for f in &newly_added_set {
      if self.files.all.contains(f) && seen_bind[DepKind::File as usize].insert(f.clone()) {
        to_bind.push((DepKind::File, f.clone(), false));
      }
    }
    for d in &directories_added {
      if self.directories.all.contains(d)
        && seen_bind[DepKind::Directory as usize].insert(d.clone())
      {
        to_bind.push((DepKind::Directory, d.clone(), false));
      }
    }
    for m in &missing_set {
      if self.missing.all.contains(m) && seen_bind[DepKind::Missing as usize].insert(m.clone()) {
        to_bind.push((DepKind::Missing, m.clone(), false));
      }
    }
    for kind in DepKind::ALL {
      for logical in std::mem::take(&mut rebind_candidates[kind as usize]) {
        if self.is_registered(kind, &logical) && seen_bind[kind as usize].insert(logical.clone()) {
          to_bind.push((kind, logical, true));
        }
      }
    }

    for (kind, logical, is_rebind) in to_bind {
      let prev_bound = self.bound_dep(kind, &logical);
      let bound = self.compute_bound_dep(&mut resolver, kind, &logical).await;
      let target_changed = prev_bound
        .as_ref()
        .is_some_and(|old| old.target != bound.target || old.target_active != bound.target_active);
      {
        let mut links = self.links.write().expect("links lock poisoned");
        if is_rebind && kind == DepKind::Directory && target_changed {
          links.clear_context_scan(&logical, self.watched_trackers(), &mut evicted);
        }
        links.bind_dep(
          kind,
          logical.clone(),
          bound,
          self.follow_symlinks,
          self.watched_trackers(),
          &mut evicted,
        );
      }
      if is_rebind {
        match kind {
          DepKind::File => {
            files_set.insert(logical.clone());
            if target_changed {
              newly_added_set.insert(logical);
            }
          }
          DepKind::Missing => {
            missing_set.insert(logical);
          }
          DepKind::Directory => {
            if target_changed {
              self.forget_context_entries_owned_by(&logical);
              context_rescans.insert(logical);
            } else if self
              .records
              .read()
              .expect("records lock poisoned")
              .absent_directories
              .contains(&logical)
              && logical.is_dir()
            {
              context_rescans.insert(logical);
            }
          }
        }
      }
    }

    let mut contexts_to_scan: Vec<(InternedPath, bool)> = directories_added
      .iter()
      .map(|p| (p.clone(), false))
      .collect();
    let added_dir_set: InternedPathSet = directories_added.iter().cloned().collect();
    for ctx in context_rescans {
      if self.directories.all.contains(&ctx) && !added_dir_set.contains(&ctx) {
        {
          let mut links = self.links.write().expect("links lock poisoned");
          links.clear_context_scan(&ctx, self.watched_trackers(), &mut evicted);
        }
        contexts_to_scan.push((ctx, true));
      }
    }
    self.prune_evicted_physical_states(&evicted);

    // Seed directory identities for all active physical root directories (including explicit `files` / `missing` parents).
    let root_dirs: Vec<InternedPath> = self
      .links
      .read()
      .expect("links lock poisoned")
      .root_dir_refs
      .keys()
      .cloned()
      .collect();
    let unseeded_roots: Vec<InternedPath> = {
      let records = self.records.read().expect("records lock poisoned");
      root_dirs
        .into_iter()
        .filter(|d| !records.directory_identities.contains_key(d))
        .collect()
    };
    if !unseeded_roots.is_empty() {
      let mut new_identities = Vec::new();
      for dir in unseeded_roots {
        if let Ok(meta) = dir.metadata()
          && meta.is_dir()
          && let Some(id) = directory_identity(dir.as_path(), &meta)
        {
          new_identities.push((dir, id));
        }
      }
      if !new_identities.is_empty() {
        let mut records = self.records.write().expect("records lock poisoned");
        for (dir, id) in new_identities {
          records.directory_identities.entry(dir).or_insert(id);
        }
      }
    }

    for dir in &directories_added {
      let bound = self.bound_dep(DepKind::Directory, dir);
      let exists = match &bound {
        Some(b) if !b.target_active => dir.symlink_metadata().is_ok(),
        Some(b) => b.target.is_dir(),
        None => dir.exists(),
      };
      let mut records = self.records.write().expect("records lock poisoned");
      if exists {
        records.absent_directories.remove(dir);
      } else {
        records.absent_directories.insert(dir.clone());
      }
    }

    let plan = ScanPlan {
      files: files_set.into_iter().collect(),
      newly_added_files: newly_added_set.into_iter().collect(),
      directories: directories_added,
      contexts_to_scan,
      missing: missing_set.into_iter().collect(),
      replaced_physical_dirs,
    };
    #[cfg(any(test, feature = "testing"))]
    {
      *self.pending_scan.lock().expect("scan plan lock poisoned") = Some(plan.clone());
    }
    Ok(plan)
  }

  pub async fn is_ignored_path(&self, path: &Path) -> bool {
    match path.to_str() {
      Some(s) => self.ignored.is_ignored(s).await,
      None => false,
    }
  }
}

impl PathManager {
  fn file_time_from_state(&self, records: &RecordState, path: &InternedPath) -> Option<FileTime> {
    if let Some(time) = records.file_times.get(path) {
      return Some(*time);
    }
    disk_mtime(path)
      .or_else(|| {
        (!self.follow_symlinks)
          .then(|| {
            std::fs::symlink_metadata(path)
              .and_then(|m| m.modified().or_else(|_| m.created()))
              .ok()
          })
          .flatten()
      })
      .map(FileTime::initial)
  }

  fn entry(time: FileTime) -> TimeInfoEntry {
    let time = time.narrowed();
    TimeInfoEntry::Entry {
      safe_time: time.safe_time,
      timestamp: mtime_to_millis(time.mtime),
      accuracy: time.accuracy,
    }
  }

  pub fn collect_time_info_entries(&self) -> (TimeInfoEntries, TimeInfoEntries) {
    let accessor = self.access();
    let path_to_string = |path: &Path| {
      path
        .to_str()
        .map_or_else(|| path.to_string_lossy().into_owned(), str::to_owned)
    };

    let file_paths: Vec<InternedPath> = accessor.files().0.iter().map(|p| p.clone()).collect();
    let ignored_files: FxHashSet<InternedPath> = {
      let links = self.links.read().expect("links lock poisoned");
      file_paths
        .iter()
        .filter(|p| {
          links
            .get_bound(DepKind::File, p)
            .is_some_and(|b| !b.target_active)
        })
        .cloned()
        .collect()
    };

    let records = self.records.read().expect("records lock poisoned");
    let mut file_timestamps = TimeInfoEntries::with_capacity(file_paths.len());
    let mut owned_safe_times: Vec<(InternedPath, Vec<InternedPath>, u64)> = Vec::new();

    for path in &file_paths {
      let entry = if ignored_files.contains(path) {
        records
          .file_times
          .get(path)
          .map_or(TimeInfoEntry::Null, |t| Self::entry(*t))
      } else {
        self
          .file_time_from_state(&records, path)
          .map_or(TimeInfoEntry::Null, Self::entry)
      };
      if let TimeInfoEntry::Entry { safe_time, .. } = entry {
        if let Some(owners) = records.context_files.get(path) {
          owned_safe_times.push((path.clone(), owners.clone(), safe_time));
        } else if !self.follow_symlinks {
          let owners: Vec<InternedPath> = path
            .ancestors()
            .skip(1)
            .map(InternedPath::from)
            .filter(|a| accessor.directories().0.contains(a))
            .collect();
          if !owners.is_empty() {
            owned_safe_times.push((path.clone(), owners, safe_time));
          }
        }
      }
      file_timestamps.push((path_to_string(path.as_path()), entry));
    }

    for path in accessor.missing().0.iter() {
      let entry = records
        .file_times
        .get(&*path)
        .map_or(TimeInfoEntry::Null, |time| Self::entry(*time));
      if let TimeInfoEntry::Entry { safe_time, .. } = entry
        && let Some(owners) = records.context_files.get(&*path)
      {
        owned_safe_times.push((path.clone(), owners.clone(), safe_time));
      }
      file_timestamps.push((path_to_string(path.as_path()), entry));
    }

    for (path, owners) in &records.context_files {
      if accessor.files().0.contains(path) {
        continue;
      }
      let Some(time) = records.file_times.get(path).copied() else {
        continue;
      };
      owned_safe_times.push((path.clone(), owners.clone(), time.narrowed().safe_time));
      file_timestamps.push((path_to_string(path.as_path()), Self::entry(time)));
    }

    let mut dir_paths: Vec<InternedPath> =
      accessor.directories().0.iter().map(|p| p.clone()).collect();
    dir_paths.extend(
      records
        .context_directories
        .keys()
        .filter(|key| !accessor.directories().0.contains(*key))
        .cloned(),
    );
    let mut dir_safe_times: FxHashMap<InternedPath, Option<u64>> =
      FxHashMap::with_capacity_and_hasher(dir_paths.len(), Default::default());
    for dir in &dir_paths {
      let last_watch_event = records.last_watch_events.get(dir).copied();
      let safe_time = if records.absent_directories.contains(dir) {
        dir
          .is_dir()
          .then(|| disk_mtime(dir))
          .flatten()
          .map(mtime_safe_time)
          .map(|own| last_watch_event.map_or(own, |event| own.max(event)))
      } else {
        Some(last_watch_event.unwrap_or_default())
      };
      dir_safe_times.insert(dir.clone(), safe_time);
    }

    if !dir_safe_times.is_empty() {
      let mut parent_indices: FxHashMap<(&OsStr, &Path), usize> = FxHashMap::default();
      let mut grouped: Vec<(&Path, &Path, u64)> = Vec::new();
      for (file, owners, safe_time) in &owned_safe_times {
        if let Some(parent) = file.parent() {
          for owner in owners {
            let key = (parent.as_os_str(), owner.as_path());
            let idx = *parent_indices.entry(key).or_insert_with(|| {
              grouped.push((parent, owner.as_path(), 0));
              grouped.len() - 1
            });
            grouped[idx].2 = grouped[idx].2.max(*safe_time);
          }
        }
      }
      for (parent, owner, safe_time) in grouped {
        for dir in parent.ancestors() {
          if !dir.starts_with(owner) {
            break;
          }
          if let Some(Some(current)) = dir_safe_times.get_mut(&InternedPath::from(dir)) {
            *current = (*current).max(safe_time);
          }
        }
      }
    }

    let mut directory_timestamps = TimeInfoEntries::with_capacity(dir_paths.len());
    for dir in &dir_paths {
      let path = path_to_string(dir.as_path());
      let has_file_record = accessor.files().0.contains(dir)
        || (records.context_files.contains_key(dir) && records.file_times.contains_key(dir));
      match dir_safe_times.get(dir).copied().flatten() {
        Some(safe_time) => {
          if !has_file_record {
            file_timestamps.push((path.clone(), TimeInfoEntry::ExistenceOnlyTimeEntry));
          }
          directory_timestamps.push((path, TimeInfoEntry::OnlySafeTimeEntry { safe_time }));
        }
        None => {
          if !has_file_record {
            file_timestamps.push((path.clone(), TimeInfoEntry::Null));
          }
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
