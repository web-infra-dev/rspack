mod build_deps;
mod file_system_info;

use rspack_cacheable::cacheable;
use rspack_hash::RspackHashDigest;
use rspack_paths::{InternedPathMap, InternedPathSet};

pub use self::{
  build_deps::{BuildDeps, BuildDepsValidationResult},
  file_system_info::{FileSystemInfo, SnapshotValidationResult},
};

/// Timestamp information captured for a file.
///
/// `safe_time` mirrors webpack's filesystem-accuracy guard. A timestamp newer
/// than a snapshot's start time cannot prove that the file stayed unchanged
/// while the snapshot was being created.
#[cacheable]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileSystemInfoEntry {
  safe_time: u64,
  timestamp: Option<u64>,
}

#[cacheable]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FileHash {
  Digest(RspackHashDigest),
  Directory,
}

#[cacheable]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TimestampAndHash {
  safe_time: u64,
  timestamp: Option<u64>,
  hash: FileHash,
}

#[cacheable]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextFileSystemInfoEntry {
  safe_time: u64,
  timestamp_hash: RspackHashDigest,
}

#[cacheable]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextTimestampAndHash {
  safe_time: u64,
  timestamp_hash: RspackHashDigest,
  hash: RspackHashDigest,
}

/// Serializable filesystem state captured by [`FileSystemInfo`].
///
/// The optional maps follow webpack's `Snapshot` layout: a snapshot allocates
/// only the collections required by its strategy. Children are reserved for
/// shared snapshots; ordinary build-dependency merges combine their maps
/// directly.
#[cacheable]
#[derive(Debug, Default)]
pub struct Snapshot {
  pub(super) start_time: Option<u64>,
  pub(super) file_timestamps: Option<InternedPathMap<Option<FileSystemInfoEntry>>>,
  pub(super) file_hashes: Option<InternedPathMap<Option<FileHash>>>,
  pub(super) file_timestamp_hashes: Option<InternedPathMap<Option<TimestampAndHash>>>,
  pub(super) context_timestamps: Option<InternedPathMap<Option<ContextFileSystemInfoEntry>>>,
  pub(super) context_hashes: Option<InternedPathMap<Option<RspackHashDigest>>>,
  pub(super) context_timestamp_hashes: Option<InternedPathMap<Option<ContextTimestampAndHash>>>,
  pub(super) missing_existence: Option<InternedPathMap<bool>>,
  pub(super) managed_item_info: Option<InternedPathMap<String>>,
  pub(super) managed_files: Option<InternedPathSet>,
  pub(super) managed_contexts: Option<InternedPathSet>,
  pub(super) managed_missing: Option<InternedPathSet>,
  #[cacheable(omit_bounds)]
  pub(super) children: Option<Vec<Box<Snapshot>>>,
}

impl Snapshot {
  pub(super) fn merge(&mut self, other: Self) {
    self.start_time = match (self.start_time, other.start_time) {
      (Some(first), Some(second)) => Some(first.min(second)),
      (first, second) => first.or(second),
    };
    merge_maps(&mut self.file_timestamps, other.file_timestamps);
    merge_maps(&mut self.file_hashes, other.file_hashes);
    merge_maps(&mut self.file_timestamp_hashes, other.file_timestamp_hashes);
    merge_maps(&mut self.context_timestamps, other.context_timestamps);
    merge_maps(&mut self.context_hashes, other.context_hashes);
    merge_maps(
      &mut self.context_timestamp_hashes,
      other.context_timestamp_hashes,
    );
    merge_maps(&mut self.missing_existence, other.missing_existence);
    merge_maps(&mut self.managed_item_info, other.managed_item_info);
    merge_sets(&mut self.managed_files, other.managed_files);
    merge_sets(&mut self.managed_contexts, other.managed_contexts);
    merge_sets(&mut self.managed_missing, other.managed_missing);

    if let Some(children) = other.children {
      self.children.get_or_insert_default().extend(children);
    }
  }
}

fn merge_maps<T>(target: &mut Option<InternedPathMap<T>>, source: Option<InternedPathMap<T>>) {
  let Some(source) = source else {
    return;
  };
  target.get_or_insert_default().extend(source);
}

fn merge_sets(target: &mut Option<InternedPathSet>, source: Option<InternedPathSet>) {
  let Some(source) = source else {
    return;
  };
  target.get_or_insert_default().extend(source);
}
