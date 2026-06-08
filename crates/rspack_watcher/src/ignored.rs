use std::{borrow::Cow, fmt::Debug, path::Path};

use cow_utils::CowUtils;
use fast_glob::glob_match;
use rspack_regex::RspackRegex;

#[derive(Default)]
pub enum FsWatcherIgnored {
  #[default]
  None,
  Path(String),
  Paths(Vec<String>),
  Regex(RspackRegex),
}

impl Debug for FsWatcherIgnored {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      FsWatcherIgnored::None => write!(f, "FsWatcherIgnored::None"),
      FsWatcherIgnored::Path(s) => write!(f, "FsWatcherIgnored::Path({s})"),
      FsWatcherIgnored::Paths(s) => write!(f, "FsWatcherIgnored::Paths({s:?})"),
      FsWatcherIgnored::Regex(reg) => write!(f, "FsWatcherIgnored::Regex({reg:?})"),
    }
  }
}

/// Normalize the path by replacing backslashes with forward slashes.
/// Smooth out the differences in the system, specifically for Windows
fn normalize_path<'a>(path: &'a str) -> Cow<'a, str> {
  path.cow_replace("\\", "/")
}

impl FsWatcherIgnored {
  pub fn should_be_ignored(&self, p: &str) -> bool {
    match self {
      FsWatcherIgnored::None => false,
      FsWatcherIgnored::Path(path) => glob_match(path, normalize_path(p).as_bytes()),
      FsWatcherIgnored::Paths(paths) => paths
        .iter()
        .any(|path| glob_match(path, normalize_path(p).as_bytes())),

      FsWatcherIgnored::Regex(reg) => reg.test(&normalize_path(p)),
    }
  }

  /// Returns `true` if `path` itself, or any of its ancestors, matches an
  /// ignored pattern.
  ///
  /// A glob like `**/dist/.rstest-temp` matches the directory entry itself but
  /// not the files inside it, so checking only the event path lets writes such
  /// as `dist/.rstest-temp/foo.mjs` slip through. Walking the ancestors makes
  /// "lives inside an ignored directory" sufficient to be ignored, regardless
  /// of whether the pattern explicitly trails with `/**`.
  pub fn matches_with_ancestors(&self, path: &Path) -> bool {
    if matches!(self, FsWatcherIgnored::None) {
      return false;
    }
    std::iter::successors(Some(path), |p| p.parent())
      .filter_map(Path::to_str)
      .any(|s| self.should_be_ignored(s))
  }
}

#[cfg(test)]
mod tests {
  use std::path::Path;

  use super::*;

  #[test]
  fn ancestor_walk_catches_files_inside_a_matched_directory() {
    // The pattern matches the directory entry, not the files within it.
    let ignored = FsWatcherIgnored::Paths(vec!["**/dist/.rstest-temp".to_owned()]);

    // The directory itself is matched by the bare `should_be_ignored`.
    assert!(ignored.should_be_ignored("/proj/dist/.rstest-temp"));
    // A file inside is NOT — this is the gap the ancestor walk closes.
    assert!(!ignored.should_be_ignored("/proj/dist/.rstest-temp/foo.mjs"));

    assert!(ignored.matches_with_ancestors(Path::new("/proj/dist/.rstest-temp")));
    assert!(ignored.matches_with_ancestors(Path::new("/proj/dist/.rstest-temp/foo.mjs")));
    assert!(ignored.matches_with_ancestors(Path::new("/proj/dist/.rstest-temp/nested/bar.mjs")));
  }

  #[test]
  fn ancestor_walk_keeps_unrelated_paths() {
    let ignored = FsWatcherIgnored::Paths(vec!["**/dist/.rstest-temp".to_owned()]);
    assert!(!ignored.matches_with_ancestors(Path::new("/proj/src/index.js")));
    assert!(!ignored.matches_with_ancestors(Path::new("/proj/dist/main.js")));
  }

  #[test]
  fn none_short_circuits() {
    assert!(!FsWatcherIgnored::None.matches_with_ancestors(Path::new("/anything/at/all")));
  }
}
