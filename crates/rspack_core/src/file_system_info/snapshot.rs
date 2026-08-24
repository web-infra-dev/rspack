use rspack_cacheable::cacheable;
use rspack_paths::{InternedPath, InternedPathSet};

#[cacheable]
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TimestampAndHash {
  pub timestamp: u64,
  pub hash: u64,
}

#[cacheable]
#[derive(Debug, Clone)]
pub(crate) struct SnapshotEntry<T> {
  pub path: InternedPath,
  pub value: T,
}

#[cacheable]
#[derive(Debug, Clone)]
pub(crate) struct ManagedItemInfo {
  pub path: InternedPath,
  pub version: String,
}

/// Filesystem state captured by [`super::FileSystemInfo`].
///
/// Fields mirror webpack's `Snapshot`: values are grouped by dependency kind
/// and validation mode instead of storing one polymorphic strategy per path.
#[cacheable]
#[derive(Debug, Default, Clone)]
pub struct Snapshot {
  pub(crate) start_time: Option<u64>,
  pub(crate) file_timestamps: Vec<SnapshotEntry<Option<u64>>>,
  pub(crate) file_hashes: Vec<SnapshotEntry<Option<u64>>>,
  pub(crate) file_tshs: Vec<SnapshotEntry<Option<TimestampAndHash>>>,
  pub(crate) context_timestamps: Vec<SnapshotEntry<Option<u64>>>,
  pub(crate) context_hashes: Vec<SnapshotEntry<Option<u64>>>,
  pub(crate) context_tshs: Vec<SnapshotEntry<Option<TimestampAndHash>>>,
  pub(crate) missing_existence: Vec<SnapshotEntry<bool>>,
  pub(crate) managed_item_info: Vec<ManagedItemInfo>,
  pub(crate) managed_files: InternedPathSet,
  pub(crate) managed_contexts: InternedPathSet,
  pub(crate) managed_missing: InternedPathSet,
}

impl Snapshot {
  pub(crate) fn insert_managed_item(&mut self, item: ManagedItemInfo) {
    if let Some(current) = self
      .managed_item_info
      .iter_mut()
      .find(|current| current.path == item.path)
    {
      *current = item;
    } else {
      self.managed_item_info.push(item);
    }
  }

  pub(crate) fn merge(&mut self, other: Self) {
    self.start_time = match (self.start_time, other.start_time) {
      (Some(a), Some(b)) => Some(a.min(b)),
      (a, b) => a.or(b),
    };
    merge_entries(&mut self.file_timestamps, other.file_timestamps);
    merge_entries(&mut self.file_hashes, other.file_hashes);
    merge_entries(&mut self.file_tshs, other.file_tshs);
    merge_entries(&mut self.context_timestamps, other.context_timestamps);
    merge_entries(&mut self.context_hashes, other.context_hashes);
    merge_entries(&mut self.context_tshs, other.context_tshs);
    merge_entries(&mut self.missing_existence, other.missing_existence);
    for item in other.managed_item_info {
      self.insert_managed_item(item);
    }
    self.managed_files.extend(other.managed_files);
    self.managed_contexts.extend(other.managed_contexts);
    self.managed_missing.extend(other.managed_missing);
  }
}

fn merge_entries<T>(entries: &mut Vec<SnapshotEntry<T>>, added: Vec<SnapshotEntry<T>>) {
  for entry in added {
    if let Some(current) = entries
      .iter_mut()
      .find(|current| current.path == entry.path)
    {
      *current = entry;
    } else {
      entries.push(entry);
    }
  }
}
