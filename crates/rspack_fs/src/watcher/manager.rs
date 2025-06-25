use std::{fmt::Debug, ops::Deref, path::PathBuf};

use async_trait::async_trait;
use dashmap::{setref::multiple::RefMulti, DashSet as HashSet};
use rspack_paths::ArcPath;

#[async_trait]
pub trait Ignored: Send + Sync {
  async fn ignore(&self, path: &str) -> bool;
}

/// An iterator that chains together references to all files, directories, and missing paths
/// stored in the PathRegister. This allows iteration over all registered paths as a single sequence.
pub struct All<'a> {
  inner: Box<dyn Iterator<Item = RefMulti<'a, ArcPath>> + 'a>,
}

impl<'a> All<'a> {
  /// Creates a new `All` iterator from the given sets of files, directories, and missing paths.
  ///
  /// # Arguments
  ///
  /// * `files` - A reference to a set of files.
  /// * `directories` - A reference to a set of directories.
  /// * `missing` - A reference to a set of missing paths.
  pub fn new(
    files: &'a HashSet<ArcPath>,
    directories: &'a HashSet<ArcPath>,
    missing: &'a HashSet<ArcPath>,
  ) -> Self {
    let files_iter = files.iter();
    let directories_iter = directories.iter();
    let missing_iter = missing.iter();
    let chain = files_iter.chain(directories_iter).chain(missing_iter);

    Self {
      inner: Box::new(chain),
    }
  }
}

impl<'a> Iterator for All<'a> {
  type Item = ArcPath;

  fn next(&mut self) -> Option<Self::Item> {
    self.inner.next().map(|v| v.deref().clone())
  }
}

pub struct PathAccessor<'a> {
  files: &'a HashSet<ArcPath>,
  directories: &'a HashSet<ArcPath>,
  missing: &'a HashSet<ArcPath>,
}

impl<'a> PathAccessor<'a> {
  pub fn new(
    files: &'a HashSet<ArcPath>,
    directories: &'a HashSet<ArcPath>,
    missing: &'a HashSet<ArcPath>,
  ) -> Self {
    Self {
      files,
      directories,
      missing,
    }
  }

  pub fn files(&self) -> &'a HashSet<ArcPath> {
    self.files
  }

  pub fn directories(&self) -> &'a HashSet<ArcPath> {
    self.directories
  }

  pub fn missing(&self) -> &'a HashSet<ArcPath> {
    self.missing
  }

  pub fn all(&self) -> All<'a> {
    All::new(self.files, self.directories, self.missing)
  }
}

/// Updating the set of registered paths, directories, and missing paths.
#[derive(Debug)]
pub struct PathUpdater {
  pub added: Vec<String>,
  pub removed: Vec<String>,
}

impl PathUpdater {
  /// Update the paths in the given set.
  ///
  /// # Arguments
  ///
  /// * `paths` - A reference to the set of paths to update.
  /// * `ignored` - An optional reference to an ignored paths filter.
  pub async fn update(self, paths: &HashSet<ArcPath>, ignored: &Option<Box<dyn Ignored>>) {
    let added_paths = self.added;
    let removed_paths = self.removed;

    for added in added_paths {
      if let Some(ignored) = ignored {
        if ignored.ignore(&added).await {
          continue;
        }
      }

      paths.insert(ArcPath::from(PathBuf::from(added)));
    }

    for removed in removed_paths {
      paths.remove(&ArcPath::from(PathBuf::from(removed)));
    }
  }
}

/// `PathManager` is responsible for managing the set of registered paths, directories, and missing paths.
#[derive(Default)]
pub struct PathManager {
  pub files: HashSet<ArcPath>,
  pub directories: HashSet<ArcPath>,
  pub missing: HashSet<ArcPath>,
  pub ignored: Option<Box<dyn Ignored>>,
}

impl PathManager {
  pub fn new(ignored: Option<Box<dyn Ignored>>) -> Self {
    Self {
      files: HashSet::new(),
      directories: HashSet::new(),
      missing: HashSet::new(),
      ignored,
    }
  }

  pub async fn update_paths(
    &self,
    files: PathUpdater,
    directories: PathUpdater,
    missing: PathUpdater,
  ) {
    files.update(&self.files, &self.ignored).await;
    directories.update(&self.directories, &self.ignored).await;
    missing.update(&self.missing, &self.ignored).await;
  }

  pub fn access(&self) -> PathAccessor<'_> {
    PathAccessor::new(&self.files, &self.directories, &self.missing)
  }
}

#[cfg(test)]
mod tests {
  use std::path::PathBuf;

  use async_trait::async_trait;
  use dashmap::DashSet as HashSet;

  use super::*;

  struct TestIgnored {
    pub ignored: Vec<String>,
  }

  #[async_trait]
  impl Ignored for TestIgnored {
    async fn ignore(&self, path: &str) -> bool {
      self.ignored.iter().any(|ignore| path.contains(ignore))
    }
  }

  #[tokio::test]
  async fn test_updater() {
    let updater = PathUpdater {
      added: vec![
        "src/index.js".to_string(),
        "node_modules/.pnpm/axios/lib/index.js".to_string(),
        ".git/abc/".to_string(),
      ],
      removed: vec![],
    };
    let paths: HashSet<ArcPath> = HashSet::new();
    let ignored = Box::new(TestIgnored {
      ignored: vec!["node_modules".to_string(), ".git".to_string()],
    });

    updater.update(&paths, &Some(ignored)).await;

    let mut path_iter = paths.into_iter().map(|p| p);
    assert_eq!(
      path_iter.next(),
      Some(ArcPath::from(PathBuf::from("src/index.js")))
    );
    assert_eq!(path_iter.next(), None);
  }

  #[test]
  fn test_accessor() {
    let files = HashSet::new();
    let file_0 = ArcPath::from(PathBuf::from("src/index.js"));
    files.insert(file_0);

    let directories = HashSet::new();
    let dir_0 = ArcPath::from(PathBuf::from("src"));
    directories.insert(dir_0);

    let missing = HashSet::new();
    let miss_0 = ArcPath::from(PathBuf::from("src/page/index.ts"));
    missing.insert(miss_0);

    let accessor = PathAccessor::new(&files, &directories, &missing);
    let mut all_paths = vec![];

    for path in accessor.all() {
      all_paths.push(path.to_string_lossy().to_string());
    }

    all_paths.sort();

    assert_eq!(all_paths, vec!["src", "src/index.js", "src/page/index.ts"]);
  }

  #[tokio::test]
  async fn test_manager() {
    let ignored = Box::new(TestIgnored {
      ignored: vec!["node_modules".to_string(), ".git".to_string()],
    });
    let path_manager = PathManager::new(Some(ignored));
    let files = PathUpdater {
      added: vec!["src/index.js".to_string()],
      removed: vec![],
    };
    let directories = PathUpdater {
      added: vec!["src".to_string(), "node_modules".to_string()],
      removed: vec![],
    };
    let missing = PathUpdater {
      added: vec!["src/page/index.ts".to_string()],
      removed: vec![],
    };
    path_manager.update_paths(files, directories, missing).await;

    let accessor = path_manager.access();
    let mut all_paths = accessor
      .all()
      .map(|p| p.to_string_lossy().to_string())
      .collect::<Vec<_>>();

    all_paths.sort();

    assert_eq!(all_paths, vec!["src", "src/index.js", "src/page/index.ts"]);
  }
}
