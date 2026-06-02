use std::fmt::Debug;

use fast_glob::glob_match;
use rspack_paths::RspackPath;
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

impl FsWatcherIgnored {
  pub fn should_be_ignored(&self, p: &str) -> bool {
    let path = RspackPath::from_path_str(p)
      .expect("watch path should be representable as RspackPath")
      .to_request_path_string();
    match self {
      FsWatcherIgnored::None => false,
      FsWatcherIgnored::Path(pattern) => glob_match(pattern, path.as_bytes()),
      FsWatcherIgnored::Paths(paths) => paths
        .iter()
        .any(|pattern| glob_match(pattern, path.as_bytes())),

      FsWatcherIgnored::Regex(reg) => reg.test(&path),
    }
  }
}
