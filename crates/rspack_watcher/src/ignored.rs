use std::{borrow::Cow, fmt::Debug};

use cow_utils::CowUtils;
use fast_glob::glob_match;
use rspack_regex::RspackRegex;

pub enum FsWatcherIgnoredItem {
  Path(String),
  Regex(RspackRegex),
}

impl Debug for FsWatcherIgnoredItem {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      FsWatcherIgnoredItem::Path(s) => write!(f, "FsWatcherIgnoredItem::Path({s})"),
      FsWatcherIgnoredItem::Regex(reg) => write!(f, "FsWatcherIgnoredItem::Regex({reg:?})"),
    }
  }
}

#[derive(Default)]
pub enum FsWatcherIgnored {
  #[default]
  None,
  Path(String),
  Paths(Vec<String>),
  Regex(RspackRegex),
  Mixed(Vec<FsWatcherIgnoredItem>),
}

impl Debug for FsWatcherIgnored {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      FsWatcherIgnored::None => write!(f, "FsWatcherIgnored::None"),
      FsWatcherIgnored::Path(s) => write!(f, "FsWatcherIgnored::Path({s})"),
      FsWatcherIgnored::Paths(s) => write!(f, "FsWatcherIgnored::Paths({s:?})"),
      FsWatcherIgnored::Regex(reg) => write!(f, "FsWatcherIgnored::Regex({reg:?})"),
      FsWatcherIgnored::Mixed(items) => write!(f, "FsWatcherIgnored::Mixed({items:?})"),
    }
  }
}

/// Normalize the path by replacing backslashes with forward slashes.
/// Smooth out the differences in the system, specifically for Windows
fn normalize_path<'a>(path: &'a str) -> Cow<'a, str> {
  path.cow_replace("\\", "/")
}

fn glob_match_path(pattern: &str, normalized_path: &str) -> bool {
  if glob_match(pattern, normalized_path.as_bytes()) {
    return true;
  }

  let mut current = normalized_path;
  while let Some(index) = current.rfind('/') {
    current = &current[..index];
    if current.is_empty() {
      break;
    }
    if glob_match(pattern, current.as_bytes()) {
      return true;
    }
  }

  false
}

impl FsWatcherIgnored {
  pub fn should_be_ignored(&self, p: &str) -> bool {
    match self {
      FsWatcherIgnored::None => false,
      FsWatcherIgnored::Path(path) => glob_match_path(path, &normalize_path(p)),
      FsWatcherIgnored::Paths(paths) => {
        let normalized_path = normalize_path(p);
        paths
          .iter()
          .any(|path| glob_match_path(path, &normalized_path))
      }
      FsWatcherIgnored::Regex(reg) => reg.test(&normalize_path(p)),
      FsWatcherIgnored::Mixed(items) => {
        let normalized_path = normalize_path(p);
        items.iter().any(|item| match item {
          FsWatcherIgnoredItem::Path(path) => glob_match_path(path, &normalized_path),
          FsWatcherIgnoredItem::Regex(reg) => reg.test(&normalized_path),
        })
      }
    }
  }
}

#[cfg(test)]
mod tests {
  use rspack_regex::RspackRegex;

  use super::{FsWatcherIgnored, FsWatcherIgnoredItem};

  #[test]
  fn should_match_mixed_glob_and_regex_patterns() {
    let ignored = FsWatcherIgnored::Mixed(vec![
      FsWatcherIgnoredItem::Path("**/dist".to_string()),
      FsWatcherIgnoredItem::Regex(RspackRegex::new(r"\.cache[\\/]").expect("valid regex")),
    ]);

    assert!(ignored.should_be_ignored("/project/dist/index.js"));
    assert!(ignored.should_be_ignored("/project/.cache/output.js"));
    assert!(!ignored.should_be_ignored("/project/src/index.js"));
  }
}
