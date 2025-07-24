use std::{borrow::Cow, fmt::Debug};

use async_trait::async_trait;
use cow_utils::CowUtils;
use fast_glob::glob_match;
use rspack_error::Result;
use rspack_regex::RspackRegex;

#[async_trait]
pub trait Ignored: Send + Sync {
  async fn ignore(&self, path: &str) -> Result<bool>;
}

#[derive(Default)]
pub enum FsWatcherIgnored {
  #[default]
  None,
  Path(String),
  Paths(Vec<String>),
  Regex(RspackRegex),
  Fn(Box<dyn Ignored>),
}

impl Debug for FsWatcherIgnored {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      FsWatcherIgnored::None => write!(f, "FsWatcherIgnored::None"),
      FsWatcherIgnored::Path(s) => write!(f, "FsWatcherIgnored::Pattern({s})"),
      FsWatcherIgnored::Paths(s) => write!(f, "FsWatcherIgnored::Patterns({s:?})"),
      FsWatcherIgnored::Regex(reg) => write!(f, "FsWatcherIgnored::Reg({reg:?})"),
      FsWatcherIgnored::Fn(_) => write!(f, "FsWatcherIgnored::Fn(...)"),
    }
  }
}

/// Normalize the path by replacing backslashes with forward slashes.
/// Smooth out the differences in the system, specifically for Windows
fn normalize_path<'a>(path: &'a str) -> Cow<'a, str> {
  path.cow_replace("\\", "/")
}

impl FsWatcherIgnored {
  pub async fn should_be_ignored(&self, p: &str) -> Result<bool> {
    match self {
      FsWatcherIgnored::None => Ok(false),
      FsWatcherIgnored::Path(path) => Ok(glob_match(path, normalize_path(p).as_bytes())),
      FsWatcherIgnored::Paths(paths) => Ok(
        paths
          .iter()
          .any(|path| glob_match(path, normalize_path(p).as_bytes())),
      ),
      FsWatcherIgnored::Regex(reg) => Ok(reg.test(&normalize_path(p))),
      FsWatcherIgnored::Fn(ignored) => ignored.ignore(p).await, // Function-based ignored cannot be empty
    }
  }
}
