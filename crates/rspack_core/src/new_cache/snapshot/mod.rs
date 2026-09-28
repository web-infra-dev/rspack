mod file_system_info;

use rspack_cacheable::cacheable;
use rspack_hash::RspackHashDigest;
#[cfg(any(allocative, feature = "allocative"))]
use rspack_paths::InternedPath;
use rspack_paths::{InternedPathMap, InternedPathSet};
#[cfg(any(allocative, feature = "allocative"))]
use rspack_util::allocative;

pub use self::file_system_info::{FileSystemInfo, SnapshotValidationResult};

/// Timestamp information captured for a file.
///
/// `safe_time` mirrors webpack's filesystem-accuracy guard. A timestamp newer
/// than a snapshot's start time cannot prove that the file stayed unchanged
/// while the snapshot was being created.
///
/// See webpack's filesystem entry data structures:
/// https://github.com/webpack/webpack/blob/ce97d583e1cd8f3e47b70737de72e91b567a8497/lib/FileSystemInfo.js#L75-L132
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
///
/// See webpack's `Snapshot` data structure:
/// https://github.com/webpack/webpack/blob/ce97d583e1cd8f3e47b70737de72e91b567a8497/lib/FileSystemInfo.js#L303-L665
#[cacheable]
#[derive(Debug, Default, Clone)]
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
  pub(super) children: Option<Vec<Snapshot>>,
}

#[cfg(any(allocative, feature = "allocative"))]
impl allocative::Allocative for Snapshot {
  fn visit<'a, 'b: 'a>(&self, visitor: &'a mut allocative::Visitor<'b>) {
    let mut visitor = visitor.enter_self(self);
    let map_storage_bytes = self.file_timestamps.as_ref().map_or(0, |map| {
      map.capacity() * std::mem::size_of::<(InternedPath, Option<FileSystemInfoEntry>)>()
    }) + self.file_hashes.as_ref().map_or(0, |map| {
      map.capacity() * std::mem::size_of::<(InternedPath, Option<FileHash>)>()
    }) + self.file_timestamp_hashes.as_ref().map_or(0, |map| {
      map.capacity() * std::mem::size_of::<(InternedPath, Option<TimestampAndHash>)>()
    }) + self.context_timestamps.as_ref().map_or(0, |map| {
      map.capacity() * std::mem::size_of::<(InternedPath, Option<ContextFileSystemInfoEntry>)>()
    }) + self.context_hashes.as_ref().map_or(0, |map| {
      map.capacity() * std::mem::size_of::<(InternedPath, Option<RspackHashDigest>)>()
    }) + self.context_timestamp_hashes.as_ref().map_or(0, |map| {
      map.capacity() * std::mem::size_of::<(InternedPath, Option<ContextTimestampAndHash>)>()
    }) + self.missing_existence.as_ref().map_or(0, |map| {
      map.capacity() * std::mem::size_of::<(InternedPath, bool)>()
    }) + self.managed_item_info.as_ref().map_or(0, |map| {
      map.capacity() * std::mem::size_of::<(InternedPath, String)>()
        + map.values().map(String::capacity).sum::<usize>()
    }) + self.managed_files.as_ref().map_or(0, |set| {
      set.capacity() * std::mem::size_of::<InternedPath>()
    }) + self.managed_contexts.as_ref().map_or(0, |set| {
      set.capacity() * std::mem::size_of::<InternedPath>()
    }) + self.managed_missing.as_ref().map_or(0, |set| {
      set.capacity() * std::mem::size_of::<InternedPath>()
    });
    visitor.visit_simple(
      allocative::Key::new("filesystem_snapshot_indexes"),
      map_storage_bytes,
    );
    if let Some(children) = &self.children {
      visitor.visit_field_with(
        allocative::Key::new("child_snapshots"),
        std::mem::size_of_val(children)
          + (children.capacity() - children.len()) * std::mem::size_of::<Snapshot>(),
        |visitor| {
          for child in children {
            allocative::Allocative::visit(child, visitor);
          }
        },
      );
    }
    visitor.exit();
  }
}

impl Snapshot {
  /// See webpack's snapshot merge implementation:
  /// https://github.com/webpack/webpack/blob/ce97d583e1cd8f3e47b70737de72e91b567a8497/lib/FileSystemInfo.js#L3081-L3166
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
