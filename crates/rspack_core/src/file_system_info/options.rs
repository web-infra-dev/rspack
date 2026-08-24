use rspack_cacheable::{cacheable, utils::PortablePath, with::As};
use rspack_regex::RspackRegex;

/// Use string or regex to match path.
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

/// Filesystem snapshot options shared by cache implementations.
#[cacheable]
#[derive(Debug, Clone, Hash)]
pub struct SnapshotOptions {
  immutable_paths: Vec<PathMatcher>,
  unmanaged_paths: Vec<PathMatcher>,
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

/// Controls which filesystem information is captured in a snapshot.
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

  pub fn is_immutable_path(&self, path: &str) -> bool {
    self.immutable_paths.iter().any(|item| item.try_match(path))
  }

  pub fn is_managed_path(&self, path: &str) -> bool {
    !self.unmanaged_paths.iter().any(|item| item.try_match(path))
      && self.managed_paths.iter().any(|item| item.try_match(path))
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
