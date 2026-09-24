use rspack_cacheable::{cacheable, utils::PortablePath, with::As};
use rspack_regex::RspackRegex;
#[cfg(allocative)]
use rspack_util::allocative;

/// Use string or regex to match path
#[cacheable]
#[derive(Debug, Clone, Hash)]
#[cfg_attr(allocative, derive(allocative::Allocative))]
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
#[cfg_attr(allocative, derive(allocative::Allocative))]
pub struct SnapshotOptions {
  /// immutable paths, snapshot will ignore them
  pub immutable_paths: Vec<PathMatcher>,
  /// unmanaged paths, snapshot will use compile time strategy even if
  /// them are in managed_paths
  pub unmanaged_paths: Vec<PathMatcher>,
  /// managed_paths, snapshot will use lib version strategy
  pub managed_paths: Vec<PathMatcher>,
  pub resolve_build_dependencies: SnapshotStrategyOptions,
  pub build_dependencies: SnapshotStrategyOptions,
  pub resolve: SnapshotStrategyOptions,
  pub module: SnapshotStrategyOptions,
  pub context_module: SnapshotStrategyOptions,
}

#[cacheable]
#[derive(Debug, Clone, Copy, Hash)]
#[cfg_attr(allocative, derive(allocative::Allocative))]
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

impl SnapshotOptions {
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
  use crate::SnapshotStrategyOptions;

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
    let options = SnapshotOptions {
      immutable_paths: vec![
        PathMatcher::String("constant".into()),
        PathMatcher::Regexp(RspackRegex::new("global/[A-Z]+").unwrap()),
      ],
      unmanaged_paths: vec![
        PathMatcher::String("node_modules/test1".into()),
        PathMatcher::Regexp(RspackRegex::new("test_modules/test.+").unwrap()),
      ],
      managed_paths: vec![
        PathMatcher::String("node_modules".into()),
        PathMatcher::Regexp(RspackRegex::new("test_modules/.+").unwrap()),
      ],
      resolve_build_dependencies: SnapshotStrategyOptions::hash_and_timestamp(),
      build_dependencies: SnapshotStrategyOptions::hash_and_timestamp(),
      resolve: SnapshotStrategyOptions::hash_and_timestamp(),
      module: SnapshotStrategyOptions::hash_and_timestamp(),
      context_module: SnapshotStrategyOptions::timestamp(),
    };

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
