use rspack_cacheable::{cacheable, utils::PortablePath, with::As};
use rspack_hash::RspackHashDigest;
use rspack_paths::{InternedPathMap, InternedPathSet};
use rspack_regex::RspackRegex;

/// Timestamp information captured for a file.
///
/// `safe_time` mirrors webpack's filesystem-accuracy guard. A timestamp newer
/// than a snapshot's start time cannot prove that the file stayed unchanged
/// while the snapshot was being created.
#[cacheable]
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct FileSystemInfoEntry {
  pub(crate) safe_time: u64,
  pub(crate) timestamp: Option<u64>,
}

#[cacheable]
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum FileHash {
  Digest(RspackHashDigest),
  Directory,
}

#[cacheable]
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TimestampAndHash {
  pub(crate) safe_time: u64,
  pub(crate) timestamp: Option<u64>,
  pub(crate) hash: FileHash,
}

#[cacheable]
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ContextFileSystemInfoEntry {
  pub(crate) safe_time: u64,
  pub(crate) timestamp_hash: RspackHashDigest,
}

#[cacheable]
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ContextTimestampAndHash {
  pub(crate) safe_time: u64,
  pub(crate) timestamp_hash: RspackHashDigest,
  pub(crate) hash: RspackHashDigest,
}

/// Serializable filesystem state captured for cache validity checks.
///
/// The optional maps follow webpack's `Snapshot` layout: a snapshot allocates
/// only the collections required by its strategy. Children are reserved for
/// shared snapshots; ordinary build-dependency merges combine their maps
/// directly.
#[cacheable]
#[derive(Debug, Default, Clone)]
pub struct Snapshot {
  pub(crate) start_time: Option<u64>,
  pub(crate) file_timestamps: Option<InternedPathMap<Option<FileSystemInfoEntry>>>,
  pub(crate) file_hashes: Option<InternedPathMap<Option<FileHash>>>,
  pub(crate) file_timestamp_hashes: Option<InternedPathMap<Option<TimestampAndHash>>>,
  pub(crate) context_timestamps: Option<InternedPathMap<Option<ContextFileSystemInfoEntry>>>,
  pub(crate) context_hashes: Option<InternedPathMap<Option<RspackHashDigest>>>,
  pub(crate) context_timestamp_hashes: Option<InternedPathMap<Option<ContextTimestampAndHash>>>,
  pub(crate) missing_existence: Option<InternedPathMap<bool>>,
  pub(crate) managed_item_info: Option<InternedPathMap<String>>,
  pub(crate) managed_files: Option<InternedPathSet>,
  pub(crate) managed_contexts: Option<InternedPathSet>,
  pub(crate) managed_missing: Option<InternedPathSet>,
  #[cacheable(omit_bounds)]
  pub(crate) children: Option<Vec<Snapshot>>,
}

impl Snapshot {
  pub(crate) fn merge(&mut self, other: Self) {
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

/// Use string or regex to match path
#[cacheable]
#[derive(Debug, Clone, Hash)]
pub enum PathMatcher {
  String(#[cacheable(with=As<PortablePath>)] String),
  Regexp(RspackRegex),
}

impl PathMatcher {
  fn try_match(&self, path: &str) -> bool {
    match self {
      Self::String(string) => path.contains(string),
      Self::Regexp(regex) => regex.test(path),
    }
  }
}

/// Snapshot options
#[cacheable]
#[derive(Debug, Clone, Hash)]
pub struct SnapshotOptions {
  /// immutable paths, snapshot will ignore them
  immutable_paths: Vec<PathMatcher>,
  /// unmanaged paths, snapshot will use compile time strategy even if
  /// them are in managed_paths
  unmanaged_paths: Vec<PathMatcher>,
  /// managed_paths, snapshot will use lib version strategy
  managed_paths: Vec<PathMatcher>,
  dependencies: SnapshotStrategyOptions,
  context_dependencies: SnapshotStrategyOptions,
}

impl Default for SnapshotOptions {
  fn default() -> Self {
    Self {
      immutable_paths: Default::default(),
      unmanaged_paths: Default::default(),
      managed_paths: Default::default(),
      dependencies: SnapshotStrategyOptions::hash_and_timestamp(),
      context_dependencies: SnapshotStrategyOptions::timestamp(),
    }
  }
}

#[cacheable]
#[derive(Debug, Clone, Copy, Hash)]
pub struct SnapshotStrategyOptions {
  pub hash: bool,
  pub timestamp: bool,
}

impl SnapshotStrategyOptions {
  pub const fn new(hash: bool, timestamp: bool) -> Self {
    Self { hash, timestamp }
  }

  pub const fn hash() -> Self {
    Self::new(true, false)
  }

  pub const fn timestamp() -> Self {
    Self::new(false, true)
  }

  pub const fn hash_and_timestamp() -> Self {
    Self::new(true, true)
  }
}

impl Default for SnapshotStrategyOptions {
  fn default() -> Self {
    Self::timestamp()
  }
}

impl SnapshotOptions {
  pub fn new(
    immutable_paths: Vec<PathMatcher>,
    unmanaged_paths: Vec<PathMatcher>,
    managed_paths: Vec<PathMatcher>,
  ) -> Self {
    Self {
      immutable_paths,
      unmanaged_paths,
      managed_paths,
      ..Default::default()
    }
  }

  pub fn dependencies_strategy(&self) -> SnapshotStrategyOptions {
    self.dependencies
  }

  pub fn context_dependencies_strategy(&self) -> SnapshotStrategyOptions {
    self.context_dependencies
  }

  pub fn is_immutable_path(&self, path_str: &str) -> bool {
    for item in &self.immutable_paths {
      if item.try_match(path_str) {
        return true;
      }
    }
    false
  }

  pub fn is_managed_path(&self, path_str: &str) -> bool {
    for item in &self.unmanaged_paths {
      if item.try_match(path_str) {
        return false;
      }
    }

    for item in &self.managed_paths {
      if item.try_match(path_str) {
        return true;
      }
    }
    false
  }
}

#[cfg(test)]
mod tests {
  use rspack_regex::RspackRegex;

  use super::{PathMatcher, SnapshotOptions};

  #[test]
  fn should_path_matcher_works() {
    let matcher = PathMatcher::String("abc".into());
    assert!(matcher.try_match("aabcc"));
    assert!(matcher.try_match("abccd"));
    assert!(matcher.try_match("xxabc"));
    assert!(!matcher.try_match("aadcc"));

    let matcher = PathMatcher::Regexp(RspackRegex::new("[0-9]").unwrap());
    assert!(matcher.try_match("aa0cc"));
    assert!(matcher.try_match("3cc"));
    assert!(!matcher.try_match("abc"));
  }

  #[test]
  fn should_snapshot_options_works() {
    let options = SnapshotOptions::new(
      vec![
        PathMatcher::String("constant".into()),
        PathMatcher::Regexp(RspackRegex::new("global/[A-Z]+").unwrap()),
      ],
      vec![
        PathMatcher::String("node_modules/test1".into()),
        PathMatcher::Regexp(RspackRegex::new("test_modules/test.+").unwrap()),
      ],
      vec![
        PathMatcher::String("node_modules".into()),
        PathMatcher::Regexp(RspackRegex::new("test_modules/.+").unwrap()),
      ],
    );

    assert!(options.is_immutable_path("/root/project/constant/var.js"));
    assert!(options.is_immutable_path("/root/project/constant1/var.js"));
    assert!(options.is_immutable_path("/root/project/1constant/var.js"));

    assert!(options.is_immutable_path("/root/project/global/NAME.js"));
    assert!(options.is_immutable_path("/root/project/global/Name.js"));
    assert!(!options.is_immutable_path("/root/project/global/var.js"));

    assert!(options.is_managed_path("/root/project/node_modules/var.js"));
    assert!(!options.is_managed_path("/root/project/node_modules/test1/var.js"));

    assert!(options.is_managed_path("/root/project/test_modules/var.js"));
    assert!(!options.is_managed_path("/root/project/test_modules/test1/var.js"));
  }
}
