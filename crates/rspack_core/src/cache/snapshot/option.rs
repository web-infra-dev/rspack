use rspack_regex::RspackRegex;

#[derive(Debug)]
pub enum PathMatcher {
  String(String),
  Regexp(RspackRegex),
}

impl PathMatcher {
  fn try_match(&self, path: &str) -> bool {
    match self {
      Self::String(string) => path.starts_with(string),
      Self::Regexp(regex) => regex.test(path),
    }
  }
}

#[derive(Debug, Default)]
pub struct SnapshotOption {
  immutable_paths: Vec<PathMatcher>,
  unmanaged_paths: Vec<PathMatcher>,
  managed_paths: Vec<PathMatcher>,
}

impl SnapshotOption {
  pub fn new(
    immutable_paths: Vec<PathMatcher>,
    unmanaged_paths: Vec<PathMatcher>,
    managed_paths: Vec<PathMatcher>,
  ) -> Self {
    Self {
      immutable_paths,
      unmanaged_paths,
      managed_paths,
    }
  }

  pub fn is_immutable_path(&self, path: &str) -> bool {
    for item in &self.immutable_paths {
      if item.try_match(path) {
        return true;
      }
    }
    false
  }

  pub fn is_managed_path(&self, path: &str) -> bool {
    for item in &self.unmanaged_paths {
      if item.try_match(path) {
        return false;
      }
    }

    for item in &self.managed_paths {
      if item.try_match(path) {
        return true;
      }
    }
    false
  }
}
