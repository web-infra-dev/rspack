use std::{ffi::OsStr, path::Path, sync::Arc, time::SystemTime};

use dashmap::mapref::one::MappedRefMut;
use rspack_paths::{InternedPath, InternedPathDashMap, InternedPathIndexSet};
use rspack_util::{fx_hash::FxHashMap, time::current_time};

use super::PathManager;
use crate::{
  TimeInfoEntries, TimeInfoEntry,
  time::{fs_accuracy, mtime_accuracy, mtime_safe_time, mtime_to_millis},
  watcher_manager::DirectoryWatcher,
};

/// One value of watchpack's `files` map, as written by `setFileTime`: the mtime
/// a path was last seen with, plus the safe time and accuracy fixed at that
/// observation.
#[derive(Debug, Clone, Copy)]
pub(crate) struct FileTime {
  pub(super) mtime: SystemTime,
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
pub(super) enum ReportedBaseline {
  #[default]
  Unknown,
  Present(SystemTime),
  Absent,
}

#[derive(Default)]
pub(crate) struct FileRecord {
  /// Last observation exported by `collect_time_info_entries`.
  pub(super) time: Option<FileTime>,
  /// State the build already knows from registration or a reported event.
  pub(super) reported: ReportedBaseline,
}

impl FileRecord {
  pub(super) fn set_time(
    &mut self,
    mtime: SystemTime,
    initial: bool,
    ignore_when_equal: bool,
  ) -> bool {
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

  pub(crate) fn is_empty(&self) -> bool {
    self.time.is_none() && self.reported == ReportedBaseline::Unknown
  }

  pub(super) fn clear_present(&mut self) {
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

/// File records live with their parent watcher, including explicitly watched files outside a context.
pub(super) struct FileTimes(pub(super) Arc<InternedPathDashMap<DirectoryWatcher>>);

impl FileTimes {
  pub(super) fn get(&self, path: &InternedPath) -> Option<FileTime> {
    self
      .0
      .get(&DirectoryWatcher::owner(path))?
      .files
      .get(path)?
      .time
  }

  fn reported(&self, path: &InternedPath) -> ReportedBaseline {
    self
      .0
      .get(&DirectoryWatcher::owner(path))
      .and_then(|watcher| watcher.files.get(path).map(|r| r.reported))
      .unwrap_or_default()
  }

  pub(super) fn record(
    &self,
    path: &InternedPath,
  ) -> MappedRefMut<'_, InternedPath, DirectoryWatcher, FileRecord> {
    self
      .0
      .entry(DirectoryWatcher::owner(path))
      .or_default()
      .map(|watcher| {
        watcher.note_member(path);
        watcher.files.entry(path.clone()).or_default()
      })
  }

  pub(super) fn clear(&self, path: &InternedPath, clear: impl FnOnce(&mut FileRecord)) {
    self
      .0
      .remove_if_mut(&DirectoryWatcher::owner(path), |_, watcher| {
        watcher.note_member(path);
        watcher.clear_file(path, clear);
        watcher.is_unused()
      });
  }
}

impl PathManager {
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
  pub fn set_reported_file_removal(&self, path: &InternedPath) {
    self.file_times.record(path).reported = ReportedBaseline::Absent;
    if (!self.files.all.contains(path) && !self.missing.all.contains(path))
      || std::fs::symlink_metadata(path).is_ok()
    {
      self.file_times.clear(path, FileRecord::clear_absent);
    }
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
      if let Some(mut watcher) = self.watchers.directories.get_mut(&ancestor)
        && watcher.subscriptions.is_active()
      {
        watcher.last_watch_event = Some(now);
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
    if !self.files.all.contains(path) && !self.missing.all.contains(path) {
      return true;
    }

    let Some(current_mtime) = disk_mtime(path) else {
      return true;
    };
    let mtime_ms = mtime_to_millis(current_mtime);
    let present = ReportedBaseline::Present(current_mtime);
    let aged = mtime_ms.saturating_add(mtime_accuracy(current_mtime)) < current_time();
    // A stale event needs no index lock. Recheck under the write lock in
    // case the scanner reported this mtime in the meantime.
    if aged && self.file_times.reported(path) == present {
      return false;
    }
    self.record(path, || {
      let mut record = self.file_times.record(path);
      if aged && record.reported == present {
        return false;
      }
      record.reported = present;
      record.time = Some(FileTime::observed(current_mtime));
      true
    })
  }
}

pub(crate) struct RegisteredTimeInfo {
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
    // The registered-file fallback can stat the disk. Keep it outside the commit lock.
    loop {
      let revision = *self.lock_index();
      let snapshot = self.collect_context_time_info(self.collect_registered_time_info());
      if *self.lock_index() == revision {
        return snapshot;
      }
    }
  }

  pub(crate) fn collect_registered_time_info(&self) -> RegisteredTimeInfo {
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

  pub(crate) fn collect_context_time_info(
    &self,
    registered: RegisteredTimeInfo,
  ) -> (TimeInfoEntries, TimeInfoEntries) {
    let RegisteredTimeInfo {
      file_paths,
      mut dir_paths,
      mut file_timestamps,
      mut file_safe_times,
    } = registered;
    for watcher in self.watchers.directories.iter() {
      for (path, member) in &watcher.members {
        if member.directory {
          dir_paths.insert(path.clone());
        }
        if member.file
          && !file_paths.contains(path)
          && let Some(time) = watcher.files.get(path).and_then(|record| record.time)
        {
          file_safe_times.push((path.clone(), time.narrowed().safe_time));
          file_timestamps.push((path_to_string(path.as_path()), Self::entry(time)));
        }
      }
    }
    let mut dir_safe_times: FxHashMap<InternedPath, Option<u64>> =
      FxHashMap::with_capacity_and_hasher(dir_paths.len(), Default::default());
    for dir in &dir_paths {
      let last_watch_event = self.watchers.event_time(dir);
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
