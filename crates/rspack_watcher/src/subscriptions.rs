use std::path::Path;

use rspack_paths::InternedPath;
use smallvec::SmallVec;

pub(crate) type RequestedPaths = SmallVec<[InternedPath; 1]>;

#[derive(Debug)]
pub(crate) struct PathSubscription {
  pub watched_path: InternedPath,
  pub requested_path: InternedPath,
}

impl From<InternedPath> for PathSubscription {
  fn from(path: InternedPath) -> Self {
    Self {
      watched_path: path.clone(),
      requested_path: path,
    }
  }
}

impl PathSubscription {
  pub fn absolute(self, base: &Path) -> Self {
    let absolute = |path: InternedPath| {
      if path.is_absolute() {
        path
      } else {
        base.join(path.as_path()).into()
      }
    };
    if self.watched_path == self.requested_path {
      return absolute(self.watched_path).into();
    }
    Self {
      watched_path: absolute(self.watched_path),
      requested_path: absolute(self.requested_path),
    }
  }
}

pub(crate) struct Subscriber {
  pub path: InternedPath,
  pub file_or_missing: bool,
}

pub(crate) type Subscribers = SmallVec<[Subscriber; 1]>;
