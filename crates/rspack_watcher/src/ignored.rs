use std::{borrow::Cow, fmt::Debug};

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
}
