use std::{sync::Arc, time::SystemTime};

use rspack_paths::{InternedPath, InternedPathMap, InternedPathSet};
use rspack_util::time::system_time_to_millis;

use super::scan::ScanQueue;
use crate::state::{fs_accuracy, mtime_accuracy};

/// One value of watchpack's `files` map, as written by `setFileTime`: the mtime
/// a path was last seen with, plus the safe time and accuracy fixed at that
/// observation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct FileTime {
  mtime: SystemTime,
  safe_time: u64,
  accuracy: u64,
}

impl FileTime {
  /// watchpack `setFileTime(initial = true)`: the path was found on disk by a
  /// scan, so only its mtime says when it changed — the safe time is that
  /// mtime padded by the filesystem's timestamp accuracy.
  pub fn initial(mtime: SystemTime) -> Self {
    let mtime_ms = system_time_to_millis(mtime);
    Self {
      mtime,
      safe_time: crate::state::mtime_safe_time(mtime_ms),
      accuracy: mtime_accuracy(mtime_ms),
    }
  }

  /// watchpack `setFileTime(initial = false)`: a live event just reported the
  /// path. The change happened no later than now, whatever mtime the new
  /// content carries, so the safe time is now, exactly.
  pub fn observed(mtime: SystemTime) -> Self {
    Self {
      mtime,
      safe_time: crate::state::now(),
      accuracy: 0,
    }
  }

  pub fn mtime(&self) -> SystemTime {
    self.mtime
  }

  /// watchpack's `fixupEntryAccuracy`: a record taken while the filesystem
  /// accuracy was coarser is read with the accuracy learned since.
  pub fn narrowed(self) -> Self {
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

  pub fn safe_time(&self) -> u64 {
    self.safe_time
  }

  pub fn accuracy(&self) -> u64 {
    self.accuracy
  }
}

/// What is known of one file in a [`DirectoryWatcher`].
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct FileEntry {
  /// The time info record. `None` while only a baseline is kept: a registered
  /// file that this cycle re-registered as missing reads `null` until it is
  /// seen again, yet keeps deduplicating events against its baseline.
  pub time: Option<FileTime>,
  /// The mtime the build already knows, for registered files and missing
  /// paths only: only this deduplicates a later event carrying the same mtime.
  pub baseline: Option<SystemTime>,
  /// Found below a registered context by a scan or named there by an event
  /// (legacy `context_files`): time info reports it as a file of that
  /// context. A record written only to seed or backfill a registered path is
  /// not.
  pub discovered: bool,
  /// Only a regular file's lstat timestamp also describes a file subscription.
  pub regular_file: bool,
}

impl FileEntry {
  pub fn is_empty(&self) -> bool {
    self.time.is_none() && self.baseline.is_none() && !self.discovered
  }
}

/// watchpack's `DirectoryWatcher`: the records of one directory, and what is
/// registered directly in it. The registrations themselves live in the
/// `Registry`; the counts here only mirror them, to know when to destroy it.
#[derive(Debug)]
pub(crate) struct DirectoryWatcher {
  pub path: InternedPath,
  /// Scans may await ignored predicates, so serialize them independently of
  /// the record lock. The Arc also identifies this watcher's lifetime.
  pub scan_queue: Arc<ScanQueue>,
  pub initial_scan_finished: Option<SystemTime>,
  pub reuse_scan_on_attach: bool,
  pub scan_invalidated: bool,
  /// It is a registered context, or below one.
  pub nested_watching: bool,
  /// Keyed by full path.
  pub files: InternedPathMap<FileEntry>,
  /// The subdirectories found while `nested_watching`.
  pub directories: InternedPathSet,
  /// When an event last reached it or something below it, in epoch millis;
  /// 0 until the first one.
  pub last_watch_event: u64,
  /// It is a registered context not on disk.
  pub absent: bool,
  pub registered_files: u32,
  pub registered_missing: u32,
  pub is_context: bool,
  /// The context whose nested subtree it belongs to.
  pub owner_context: Option<InternedPath>,
}

impl DirectoryWatcher {
  pub fn new(path: InternedPath) -> Self {
    Self {
      path,
      scan_queue: Arc::default(),
      initial_scan_finished: None,
      reuse_scan_on_attach: false,
      scan_invalidated: false,
      nested_watching: false,
      files: InternedPathMap::default(),
      directories: InternedPathSet::default(),
      last_watch_event: 0,
      absent: false,
      registered_files: 0,
      registered_missing: 0,
      is_context: false,
      owner_context: None,
    }
  }

  /// Nothing is registered in it and no context's subtree holds it.
  pub fn is_unreferenced(&self) -> bool {
    self.registered_files == 0
      && self.registered_missing == 0
      && !self.is_context
      && self.owner_context.is_none()
  }
}
