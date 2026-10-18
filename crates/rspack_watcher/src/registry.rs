use std::path::{Path, PathBuf};

use rspack_error::Result;
use rspack_paths::{InternedPath, InternedPathDashSet};

use super::{FsWatcherIgnored, ignored::IgnoredMatcher, state::PathDiff};

/// `PathAccessor` provides access to the sets of files, directories, and missing paths.
pub(crate) struct PathAccessor<'a> {
  files: &'a PathTracker,
  directories: &'a PathTracker,
  missing: &'a PathTracker,
}

impl<'a> PathAccessor<'a> {
  /// Creates a new `PathAccessor` with references to the sets of files, directories, and missing paths.
  fn new(registry: &'a Registry) -> Self {
    Self {
      files: &registry.files,
      directories: &registry.directories,
      missing: &registry.missing,
    }
  }

  /// Returns references to the sets of files, including added and removed files.
  pub fn files(
    &self,
  ) -> (
    &'a InternedPathDashSet,
    &'a InternedPathDashSet,
    &'a InternedPathDashSet,
  ) {
    (&self.files.all, &self.files.added, &self.files.removed)
  }

  /// Returns references to the set of directories, including added and removed directories.
  pub fn directories(
    &self,
  ) -> (
    &'a InternedPathDashSet,
    &'a InternedPathDashSet,
    &'a InternedPathDashSet,
  ) {
    (
      &self.directories.all,
      &self.directories.added,
      &self.directories.removed,
    )
  }

  /// Returns references to the set of missing paths, including added and removed missing paths.
  pub fn missing(
    &self,
  ) -> (
    &'a InternedPathDashSet,
    &'a InternedPathDashSet,
    &'a InternedPathDashSet,
  ) {
    (
      &self.missing.all,
      &self.missing.added,
      &self.missing.removed,
    )
  }
}

/// `PathUpdater` is used to update collections of registered paths (files, directories, and missing paths)
/// by specifying which paths have been added and which have been removed. It holds vectors of paths to be
/// added and removed, and provides functionality to apply these changes to a path tracker. This struct
/// facilitates batch updates to the path sets, ensuring that additions and removals are processed efficiently.
#[derive(Debug)]
struct PathUpdater {
  pub added: Vec<InternedPath>,
  pub removed: Vec<InternedPath>,
  base_dir: PathBuf,
}

impl<Added, Removed> From<(Added, Removed)> for PathUpdater
where
  Added: Iterator<Item = InternedPath>,
  Removed: Iterator<Item = InternedPath>,
{
  fn from((added, removed): (Added, Removed)) -> Self {
    let added: Vec<_> = added.collect();
    let removed: Vec<_> = removed.collect();
    let base_dir = if added.iter().chain(&removed).any(|path| !path.is_absolute()) {
      std::env::current_dir().unwrap_or_default()
    } else {
      Default::default()
    };
    Self {
      added,
      removed,
      base_dir,
    }
  }
}

impl PathUpdater {
  /// Update the paths in the given set. Returns the paths it newly holds and
  /// those it no longer does.
  async fn update(
    self,
    watch_tracker: &PathTracker,
    ignored: &IgnoredMatcher,
  ) -> Result<(Vec<InternedPath>, Vec<InternedPath>)> {
    let added_paths = self.added;
    let removed_paths = self.removed;
    let mut registered = Vec::new();
    let mut unregistered = Vec::new();

    for added in added_paths {
      // Absolutize before the ignored check so registration filters on the
      // same absolute path the watcher reports for events.
      let added = if added.is_absolute() {
        added
      } else {
        InternedPath::from(self.base_dir.join(added.as_ref()))
      };

      // Skip ignored paths AND anything inside an ignored directory.
      if ignored
        .is_ignored(added.to_str().expect("Path should be valid UTF-8"))
        .await
      {
        continue;
      }

      if watch_tracker.add(added.clone()) {
        registered.push(added);
      }
    }

    for removed in removed_paths {
      let removed = if removed.is_absolute() {
        removed
      } else {
        InternedPath::from(self.base_dir.join(removed.as_ref()))
      };
      if watch_tracker.remove(removed.clone()) {
        unregistered.push(removed);
      }
    }
    Ok((registered, unregistered))
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
pub(crate) struct PathTracker {
  pub(crate) added: InternedPathDashSet,
  pub(crate) removed: InternedPathDashSet,
  pub(crate) all: InternedPathDashSet,
}

impl PathTracker {
  fn reset(&self) {
    self.added.clear();
    self.removed.clear();
  }

  /// Adds a path to the tracker; whether it was not tracked before.
  fn add(&self, path: InternedPath) -> bool {
    self.added.insert(path.clone());
    self.all.insert(path)
  }

  /// Removes a path from the tracker; whether it was tracked.
  fn remove(&self, path: InternedPath) -> bool {
    let tracked = self.all.remove(&path).is_some();
    self.removed.insert(path);
    tracked
  }
}

/// What one [`Registry::update`] changed in each set.
pub(crate) struct RegistryChanges {
  pub files: PathDiff,
  pub directories: PathDiff,
  pub missing: PathDiff,
}

/// The registered files, directories and missing paths, with this cycle's
/// diff, and the `ignored` filter applied when they are registered.
#[derive(Default)]
pub(crate) struct Registry {
  pub(crate) files: PathTracker,
  pub(crate) directories: PathTracker,
  pub(crate) missing: PathTracker,
  pub(crate) ignored: IgnoredMatcher,
}

impl Registry {
  pub fn new(ignored: FsWatcherIgnored) -> Self {
    Self {
      files: PathTracker::default(),
      directories: PathTracker::default(),
      missing: PathTracker::default(),
      ignored: IgnoredMatcher::new(ignored),
    }
  }

  /// Clear the per-`watch()`-call diff (added / removed sets).
  pub fn reset(&self) {
    self.files.reset();
    self.directories.reset();
    self.missing.reset();
  }

  /// Apply this cycle's diff and return actual additions and removals for each set.
  pub async fn update(
    &self,
    files: (
      impl Iterator<Item = InternedPath>,
      impl Iterator<Item = InternedPath>,
    ),
    directories: (
      impl Iterator<Item = InternedPath>,
      impl Iterator<Item = InternedPath>,
    ),
    missing: (
      impl Iterator<Item = InternedPath>,
      impl Iterator<Item = InternedPath>,
    ),
  ) -> Result<RegistryChanges> {
    Ok(RegistryChanges {
      files: PathUpdater::from(files)
        .update(&self.files, &self.ignored)
        .await?,
      directories: PathUpdater::from(directories)
        .update(&self.directories, &self.ignored)
        .await?,
      missing: PathUpdater::from(missing)
        .update(&self.missing, &self.ignored)
        .await?,
    })
  }

  /// Create a new `PathAccessor` to access the current state of paths, directories, and missing paths.
  pub fn access(&self) -> PathAccessor<'_> {
    PathAccessor::new(self)
  }

  /// Whether `path` is excluded from watching by the configured ignored
  /// patterns — directly, or by living inside an ignored directory.
  pub async fn is_ignored_path(&self, path: &Path) -> bool {
    match path.to_str() {
      Some(s) => self.ignored.is_ignored(s).await,
      None => false,
    }
  }
}

#[cfg(test)]
mod tests {
  use rspack_paths::Utf8Path;

  use super::*;

  #[tokio::test]
  async fn test_updater() {
    let updater = PathUpdater::from((
      vec![
        InternedPath::from(Utf8Path::new("src/index.js")),
        InternedPath::from(Utf8Path::new("node_modules/.pnpm/axios/lib/index.js")),
        InternedPath::from(Utf8Path::new(".git/abc/")),
      ]
      .into_iter(),
      vec![].into_iter(),
    ));
    let ignored = FsWatcherIgnored::Paths(vec![
      "**/.git/**".to_owned(),
      "**/node_modules/**".to_owned(),
    ]);

    let path_tracker = PathTracker::default();

    updater
      .update(&path_tracker, &IgnoredMatcher::new(ignored))
      .await
      .unwrap();

    let all = path_tracker.all;

    assert_eq!(all.len(), 1);
    assert!(
      all
        .iter()
        .any(|p| p.to_string_lossy().contains("src/index.js"))
    )
  }

  #[tokio::test]
  async fn test_accessor() {
    let registry = Registry::default();

    let files = (
      vec![InternedPath::from(Utf8Path::new("src/index.js"))].into_iter(),
      vec![].into_iter(),
    );
    let dirs = (
      vec![InternedPath::from(Utf8Path::new("src"))].into_iter(),
      vec![].into_iter(),
    );
    let missing = (
      vec![InternedPath::from(Utf8Path::new("src/page/index.ts"))].into_iter(),
      vec![].into_iter(),
    );

    registry.update(files, dirs, missing).await.unwrap();

    let accessor = PathAccessor::new(&registry);
    let mut all_paths = vec![];

    for set in [
      accessor.files().0,
      accessor.directories().0,
      accessor.missing().0,
    ] {
      for path in set.iter() {
        all_paths.push(path.to_string_lossy().to_string());
      }
    }

    all_paths.sort();

    assert_eq!(all_paths.len(), 3);

    let should_exist_paths = vec!["src", "src/index.js", "src/page/index.ts"];

    for path in should_exist_paths {
      assert!(all_paths.iter().any(|p| p.ends_with(path)));
    }
  }
}
