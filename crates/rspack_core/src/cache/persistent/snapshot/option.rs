use rspack_regex::RspackRegex;

/// Use string or regex to match path
#[derive(Debug, Clone)]
pub enum PathMatcher {
  String(String),
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
#[derive(Debug, Default, Clone)]
pub struct SnapshotOptions {
  /// immutable paths, snapshot will ignore them
  immutable_paths: Vec<PathMatcher>,
  /// unmanaged paths, snapshot will use compile time strategy even if
  /// them are in managed_paths
  unmanaged_paths: Vec<PathMatcher>,
  /// managed_paths, snapshot will use package version strategy
  managed_paths: Vec<PathMatcher>,
  /// hash_paths, snapshot will use file hash strategy
  hash_paths: Vec<PathMatcher>,
}

impl SnapshotOptions {
  pub fn new(
    immutable_paths: Vec<PathMatcher>,
    unmanaged_paths: Vec<PathMatcher>,
    managed_paths: Vec<PathMatcher>,
    hash_paths: Vec<PathMatcher>,
  ) -> Self {
    Self {
      immutable_paths,
      unmanaged_paths,
      managed_paths,
      hash_paths,
    }
  }

  #[inline]
  fn match_path(paths: &Vec<PathMatcher>, path_str: &str) -> bool {
    for item in paths {
      if item.try_match(path_str) {
        return true;
      }
    }
    false
  }

  pub fn is_immutable_path(&self, path_str: &str) -> bool {
    Self::match_path(&self.immutable_paths, path_str)
  }

  pub fn is_managed_path(&self, path_str: &str) -> bool {
    if Self::match_path(&self.unmanaged_paths, path_str) {
      return false;
    }
    Self::match_path(&self.managed_paths, path_str)
  }

  pub fn is_hash_path(&self, path_str: &str) -> bool {
    Self::match_path(&self.hash_paths, path_str)
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
      vec![
        PathMatcher::String("node_modules/hash".into()),
        PathMatcher::Regexp(RspackRegex::new("test_modules/hash/.+").unwrap()),
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

    assert!(!options.is_hash_path("/root/project/node_modules/var.js"));
    assert!(options.is_hash_path("/root/project/node_modules/hash/var.js"));

    assert!(options.is_hash_path("/root/project/test_modules/hash/var.js"));
    assert!(!options.is_hash_path("/root/project/test_modules/var.js"));
  }
}
