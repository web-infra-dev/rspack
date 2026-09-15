use rspack_cacheable::{cacheable, utils::PortablePath, with::As};
use rspack_regex::RspackRegex;

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

  fn matches_directory(&self, path: &str) -> bool {
    match self {
      Self::String(directory) => directory_prefix_length(directory, path).is_some(),
      Self::Regexp(regex) => regex.test(path),
    }
  }

  fn managed_root_length(&self, path: &str) -> Option<usize> {
    match self {
      Self::String(directory) => directory_prefix_length(directory, path),
      Self::Regexp(regex) => regex.first_capture_or_match(path).map(str::len),
    }
  }
}

fn directory_prefix_length(directory: &str, path: &str) -> Option<usize> {
  let directory = directory.trim_end_matches(['/', '\\']);
  let rest = path.strip_prefix(directory)?;
  rest.starts_with(['/', '\\']).then_some(directory.len() + 1)
}

/// Classification used by the new cache. Legacy snapshots retain their
/// historical substring matching through `is_immutable_path`/`is_managed_path`.
pub(crate) enum SnapshotPath<'a> {
  Unmanaged,
  Immutable,
  Managed(&'a str),
}

// Follow webpack's getManagedItem: a managed root contains packages, including
// scoped packages and packages with their own nested node_modules directory.
fn managed_item(path: &str, root_length: usize) -> Option<&str> {
  let bytes = path.as_bytes();
  let mut index = root_length;
  let mut segments = 1;
  let mut starting = true;
  while index < bytes.len() {
    match bytes[index] {
      b'/' | b'\\' => {
        segments -= 1;
        if segments == 0 {
          break;
        }
        starting = true;
      }
      b'.' if starting => return None,
      b'@' => {
        if !starting {
          return None;
        }
        segments += 1;
      }
      _ => starting = false,
    }
    index += 1;
  }
  if index == bytes.len() {
    segments -= 1;
  }
  if segments != 0 {
    return None;
  }
  if let Some(rest) = path.get(index + 1..)
    && let Some(rest) = rest.strip_prefix("node_modules")
  {
    if rest.is_empty() {
      return Some(path);
    }
    if rest.starts_with(['/', '\\']) {
      return managed_item(path, index + 14);
    }
  }
  path.get(..index)
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
  pub(crate) fn classify_path<'a>(&self, path: &'a str) -> SnapshotPath<'a> {
    if self
      .unmanaged_paths
      .iter()
      .any(|item| item.matches_directory(path))
    {
      return SnapshotPath::Unmanaged;
    }
    if self
      .immutable_paths
      .iter()
      .any(|item| item.matches_directory(path))
    {
      return SnapshotPath::Immutable;
    }
    for matcher in &self.managed_paths {
      if let Some(root_length) = matcher.managed_root_length(path)
        && let Some(item) = managed_item(path, root_length)
      {
        return SnapshotPath::Managed(item);
      }
    }
    SnapshotPath::Unmanaged
  }

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
