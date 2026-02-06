use std::{fmt::Debug, ops::Deref, path::PathBuf};

use dashmap::setref::multiple::RefMulti;
use rspack_error::Result;
use rspack_paths::{ArcPath, ArcPathDashSet};

use super::FsWatcherIgnored;

/// An iterator that chains together references to all files, directories, and missing paths
/// stored in the [`PathTracker`]. This allows iteration over all registered paths as a single sequence.
pub(crate) struct All<'a> {
  inner: Box<dyn Iterator<Item = RefMulti<'a, ArcPath>> + 'a>,
}

impl<'a> All<'a> {
  /// Creates a new `All` iterator from the given sets of files, directories, and missing paths.
  fn new(
    files: &'a ArcPathDashSet,
    directories: &'a ArcPathDashSet,
    missing: &'a ArcPathDashSet,
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

/// `PathAccessor` provides access to the sets of files, directories, and missing paths.
pub(crate) struct PathAccessor<'a> {
  files: &'a PathTracker,
  directories: &'a PathTracker,
  missing: &'a PathTracker,
}

impl<'a> PathAccessor<'a> {
  /// Creates a new `PathAccessor` with references to the sets of files, directories, and missing paths.
  fn new(path_manager: &'a PathManager) -> Self {
    Self {
      files: &path_manager.files,
      directories: &path_manager.directories,
      missing: &path_manager.missing,
    }
  }

  /// Returns references to the sets of files, including added and removed files.
  pub(crate) fn files(&self) -> (&'a ArcPathDashSet, &'a ArcPathDashSet, &'a ArcPathDashSet) {
    (&self.files.all, &self.files.added, &self.files.removed)
  }

  /// Returns references to the set of directories, including added and removed directories.
  pub(crate) fn directories(&self) -> (&'a ArcPathDashSet, &'a ArcPathDashSet, &'a ArcPathDashSet) {
    (
      &self.directories.all,
      &self.directories.added,
      &self.directories.removed,
    )
  }

  /// Returns references to the set of missing paths, including added and removed missing paths.
  pub(crate) fn missing(&self) -> (&'a ArcPathDashSet, &'a ArcPathDashSet, &'a ArcPathDashSet) {
    (
      &self.missing.all,
      &self.missing.added,
      &self.missing.removed,
    )
  }

  /// Returns an iterator that combines all files, directories, and missing paths into a single sequence.
  pub(crate) fn all(&self) -> impl Iterator<Item = ArcPath> + '_ {
    All::new(&self.files.all, &self.directories.all, &self.missing.all)
  }
}

/// `PathUpdater` is used to update collections of registered paths (files, directories, and missing paths)
/// by specifying which paths have been added and which have been removed. It holds vectors of paths to be
/// added and removed, and provides functionality to apply these changes to a path tracker. This struct
/// facilitates batch updates to the path sets, ensuring that additions and removals are processed efficiently.
#[derive(Debug)]
struct PathUpdater {
  pub added: Vec<ArcPath>,
  pub removed: Vec<ArcPath>,
  base_dir: PathBuf,
}

impl<Added, Removed> From<(Added, Removed)> for PathUpdater
where
  Added: Iterator<Item = ArcPath>,
  Removed: Iterator<Item = ArcPath>,
{
  fn from((added, removed): (Added, Removed)) -> Self {
    Self {
      added: added.collect(),
      removed: removed.collect(),
      base_dir: std::env::current_dir().unwrap_or_default(),
    }
  }
}

impl PathUpdater {
  /// Update the paths in the given set.
  fn update(self, watch_tracker: &PathTracker, ignored: &FsWatcherIgnored) -> Result<()> {
    let added_paths = self.added;
    let removed_paths = self.removed;

    for added in added_paths {
      if ignored.should_be_ignored(added.to_str().expect("Path should be valid UTF-8")) {
        continue; // Skip ignored paths
      }

      if added.is_absolute() {
        watch_tracker.add(added);
        continue;
      }

      let added_absolute_path = self.base_dir.join(added.as_ref());

      watch_tracker.add(ArcPath::from(added_absolute_path));
    }

    for removed in removed_paths {
      if removed.is_absolute() {
        watch_tracker.remove(removed);
        continue;
      }

      let removed_absolute_path = self.base_dir.join(removed.as_ref());

      watch_tracker.remove(ArcPath::from(removed_absolute_path));
    }
    Ok(())
  }
}

#[derive(Debug, Default)]
/// `PathTracker` is responsible for tracking the state of file system paths for the watcher.
///
/// It maintains three sets:
/// - `added`: Paths that have been recently added and are being watched.
/// - `removed`: Paths that have been removed from watching.
/// - `all`: All currently watched paths.
///
/// This struct enables efficient updates and queries for the file system watcher,
/// ensuring that changes to the set of watched paths are tracked and managed correctly.
struct PathTracker {
  added: ArcPathDashSet,
  removed: ArcPathDashSet,
  all: ArcPathDashSet,
}

impl PathTracker {
  fn reset(&self) {
    self.added.clear();
    self.removed.clear();
  }

  /// Adds a path to the tracker.
  fn add(&self, path: ArcPath) {
    self.added.insert(path.clone());
    self.all.insert(path);
  }

  /// Removes a path from the tracker.
  fn remove(&self, path: ArcPath) {
    self.all.remove(&path);
    self.removed.insert(path);
  }
}

/// `PathManager` is responsible for managing the set of files, directories, and missing paths.
#[derive(Default)]
pub(crate) struct PathManager {
  files: PathTracker,
  directories: PathTracker,
  missing: PathTracker,
  pub ignored: FsWatcherIgnored,
}

impl PathManager {
  /// Create a new `PathManager` with an optional ignored paths filter.
  pub(crate) fn new(ignored: FsWatcherIgnored) -> Self {
    Self {
      files: PathTracker::default(),
      directories: PathTracker::default(),
      missing: PathTracker::default(),
      ignored,
    }
  }

  /// Reset the state of the `PathManager`, clearing all tracked paths.
  pub(crate) fn reset(&self) {
    self.files.reset();
    self.directories.reset();
    self.missing.reset();
  }

  /// Update the paths, directories, and missing paths in the `PathManager`.
  pub(crate) fn update(
    &self,
    files: (impl Iterator<Item = ArcPath>, impl Iterator<Item = ArcPath>),
    directories: (impl Iterator<Item = ArcPath>, impl Iterator<Item = ArcPath>),
    missing: (impl Iterator<Item = ArcPath>, impl Iterator<Item = ArcPath>),
  ) -> Result<()> {
    PathUpdater::from(files).update(&self.files, &self.ignored)?;
    PathUpdater::from(directories).update(&self.directories, &self.ignored)?;
    PathUpdater::from(missing).update(&self.missing, &self.ignored)?;

    Ok(())
  }

  /// Create a new `PathAccessor` to access the current state of paths, directories, and missing paths.
  pub(crate) fn access(&self) -> PathAccessor<'_> {
    PathAccessor::new(self)
  }
}

#[cfg(test)]
mod tests {
  use rspack_paths::Utf8Path;

  use super::*;

  #[test]
  fn test_updater() {
    let updater = PathUpdater::from((
      vec![
        ArcPath::from(Utf8Path::new("src/index.js")),
        ArcPath::from(Utf8Path::new("node_modules/.pnpm/axios/lib/index.js")),
        ArcPath::from(Utf8Path::new(".git/abc/")),
      ]
      .into_iter(),
      vec![].into_iter(),
    ));
    let ignored = FsWatcherIgnored::Paths(vec![
      "**/.git/**".to_owned(),
      "**/node_modules/**".to_owned(),
    ]);

    let path_tracker = PathTracker::default();

    updater.update(&path_tracker, &ignored).unwrap();

    let all = path_tracker.all;

    assert_eq!(all.len(), 1);
    assert!(
      all
        .iter()
        .any(|p| p.to_string_lossy().contains("src/index.js"))
    )
  }

  #[test]
  fn test_accessor() {
    let path_manager = PathManager::default();

    let files = (
      vec![ArcPath::from(Utf8Path::new("src/index.js"))].into_iter(),
      vec![].into_iter(),
    );
    let dirs = (
      vec![ArcPath::from(Utf8Path::new("src"))].into_iter(),
      vec![].into_iter(),
    );
    let missing = (
      vec![ArcPath::from(Utf8Path::new("src/page/index.ts"))].into_iter(),
      vec![].into_iter(),
    );

    path_manager.update(files, dirs, missing).unwrap();

    let accessor = PathAccessor::new(&path_manager);
    let mut all_paths = vec![];

    for path in accessor.all() {
      all_paths.push(path.to_string_lossy().to_string());
    }

    all_paths.sort();

    assert_eq!(all_paths.len(), 3);

    let should_exist_paths = vec!["src", "src/index.js", "src/page/index.ts"];

    for path in should_exist_paths {
      assert!(all_paths.iter().any(|p| p.ends_with(path)));
    }
  }

  #[test]
  fn test_manager() {
    let ignored = FsWatcherIgnored::Paths(vec![
      "**/node_modules/**".to_string(),
      "**/.git/**".to_string(),
    ]);
    let path_manager = PathManager::new(ignored);
    let files = (
      vec![ArcPath::from(Utf8Path::new("src/index.js"))].into_iter(),
      vec![].into_iter(),
    );
    let directories = (
      vec![
        ArcPath::from(Utf8Path::new("src/")),
        ArcPath::from(Utf8Path::new("node_modules/")),
      ]
      .into_iter(),
      vec![].into_iter(),
    );
    let missing = (
      vec![ArcPath::from(Utf8Path::new("src/page/index.ts"))].into_iter(),
      vec![].into_iter(),
    );

    path_manager.update(files, directories, missing).unwrap();

    let accessor = path_manager.access();
    let mut all_paths = accessor
      .all()
      .map(|p| p.to_string_lossy().to_string())
      .collect::<Vec<_>>();

    all_paths.sort();

    assert_eq!(all_paths.len(), 3);

    let should_exist_paths = vec!["src/", "src/index.js", "src/page/index.ts"];

    for path in should_exist_paths {
      assert!(all_paths.iter().any(|p| p.ends_with(path)));
    }
  }
}
