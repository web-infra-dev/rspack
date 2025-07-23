use std::fmt::Debug;

use async_trait::async_trait;
use cow_utils::CowUtils;
use fast_glob::glob_match;
use rspack_error::Result;

#[async_trait]
pub trait Ignored: Send + Sync {
  async fn ignore(&self, path: &str) -> Result<bool>;
}

#[derive(Default)]
pub enum FsWatcherIgnored {
  #[default]
  None,
  Path(String),
  Fn(Box<dyn Ignored>),
}

impl Debug for FsWatcherIgnored {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      FsWatcherIgnored::None => write!(f, "FsWatcherIgnored::None"),
      FsWatcherIgnored::Path(s) => write!(f, "FsWatcherIgnored::Pattern({s})"),
      FsWatcherIgnored::Fn(_) => write!(f, "FsWatcherIgnored::Fn(...)"),
    }
  }
}

impl FsWatcherIgnored {
  pub async fn should_be_ignored(&self, path: &str) -> Result<bool> {
    match self {
      FsWatcherIgnored::None => Ok(false),
      FsWatcherIgnored::Path(s) => Ok(glob_match(s, path.cow_replace("\\", "/").as_bytes())), // Smooth out the differences in the system, specifically for Windows
      FsWatcherIgnored::Fn(ignored) => ignored.ignore(path).await, // Function-based ignored cannot be empty
    }
  }
}
