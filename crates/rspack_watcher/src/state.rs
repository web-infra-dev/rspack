use rspack_paths::InternedPath;
pub(crate) use rspack_util::time::{
  current_time as now, fs_accuracy, mtime_accuracy, mtime_safe_time,
};

/// One side of a `watch()` diff: (added, removed).
pub(crate) type PathDiff = (Vec<InternedPath>, Vec<InternedPath>);

/// Where an event comes from.
#[derive(Debug, Clone, Copy)]
pub(crate) enum Origin {
  /// The OS watcher; `appeared` when it created or renamed the path into place.
  Disk { appeared: bool },
  /// A scan, for what changed while the watch was not active yet: a file a
  /// context's initial scan found changed since the start time, or a
  /// registered path the scanner backfills. Already judged a change, it is
  /// never dropped as a duplicate or as attribute-only.
  Scan,
}
