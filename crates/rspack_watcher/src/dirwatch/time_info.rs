use std::path::Path;

use rspack_paths::InternedPath;
use rspack_util::{fx_hash::FxHashMap, time::system_time_to_millis};

use super::{FileTime, WatcherManager, disk_mtime};
use crate::{TimeInfoEntries, TimeInfoEntry};

impl WatcherManager {
  /// watchpack's `collectTimeInfoEntries(fileTimestamps, directoryTimestamps)`,
  /// returned as `(fileTimestamps, directoryTimestamps)`.
  ///
  /// - A registered file reads its record, or a fresh stat when it has none.
  /// - A registered missing path reads its record, or `null`: never a stat,
  ///   the missing set is every resolver miss.
  /// - A file found below a context: every `discovered` entry with a record
  ///   that is not a registered file (that one is reported above). A
  ///   registered missing path among them is reported a second time, with the
  ///   same entry, as legacy does.
  /// - A directory — a registered context, or a subdirectory linked into a
  ///   watcher's `directories` — reads `{}` and `{ safeTime }`: the max of its
  ///   `lastWatchEvent` and the safe times of every file reported above that
  ///   lies below it. Only an absent context is stat'ed; absent, it reads
  ///   `null` in both, and no record below can raise it.
  pub fn collect_time_info_entries(&self) -> (TimeInfoEntries, TimeInfoEntries) {
    let registered_files: Vec<InternedPath> =
      self.registry.files.all.iter().map(|p| p.clone()).collect();
    let registered_missing: Vec<InternedPath> = self
      .registry
      .missing
      .all
      .iter()
      .map(|p| p.clone())
      .collect();
    let mut file_timestamps = TimeInfoEntries::with_capacity(registered_files.len());
    let mut file_safe_times: Vec<(InternedPath, u64)> = Vec::new();
    let mut report_file = |path: &InternedPath, time: Option<FileTime>| {
      let entry = match time.map(FileTime::narrowed) {
        Some(time) => {
          file_safe_times.push((path.clone(), time.safe_time()));
          TimeInfoEntry::Entry {
            safe_time: time.safe_time(),
            timestamp: system_time_to_millis(time.mtime()),
            accuracy: time.accuracy(),
          }
        }
        None => TimeInfoEntry::Null,
      };
      file_timestamps.push((path.to_string_lossy().to_string(), entry));
    };

    for path in &registered_files {
      let time = self
        .recorded_time(path)
        .or_else(|| disk_mtime(path).map(FileTime::initial));
      report_file(path, time);
    }
    for path in &registered_missing {
      report_file(path, self.recorded_time(path));
    }

    let mut found_files: Vec<(InternedPath, FileTime)> = Vec::new();
    let mut found_directories: Vec<InternedPath> = Vec::new();
    for watcher in self.watchers.iter() {
      found_files.extend(
        watcher
          .files
          .iter()
          .filter(|(path, entry)| entry.discovered && !self.registry.files.all.contains(*path))
          .filter_map(|(path, entry)| Some((path.clone(), entry.time?))),
      );
      found_directories.extend(
        watcher
          .directories
          .iter()
          .filter(|dir| !self.registry.directories.all.contains(*dir))
          .cloned(),
      );
    }
    for (path, time) in &found_files {
      report_file(path, Some(*time));
    }

    let mut dir_paths: Vec<InternedPath> = self
      .registry
      .directories
      .all
      .iter()
      .map(|p| p.clone())
      .collect();
    dir_paths.extend(found_directories);
    let mut dir_safe_times: FxHashMap<InternedPath, Option<u64>> =
      FxHashMap::with_capacity_and_hasher(dir_paths.len(), Default::default());
    for dir in &dir_paths {
      let (last_watch_event, absent) = self.watchers.get(dir).map_or((0, false), |watcher| {
        (watcher.last_watch_event, watcher.absent)
      });
      let safe_time = if absent {
        dir
          .is_dir()
          .then(|| disk_mtime(dir))
          .flatten()
          .map(|mtime| crate::state::mtime_safe_time(system_time_to_millis(mtime)))
          .map(|own| own.max(last_watch_event))
      } else {
        Some(last_watch_event)
      };
      dir_safe_times.insert(dir.clone(), safe_time);
    }
    // Files are folded into their parent first, so each ancestor chain is
    // walked once.
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

  fn recorded_time(&self, path: &InternedPath) -> Option<FileTime> {
    let watcher = self.watchers.get(&InternedPath::from(path.parent()?))?;
    watcher.files.get(path)?.time
  }
}
