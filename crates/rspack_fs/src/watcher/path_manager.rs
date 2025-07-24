use std::{fmt::Debug, ops::Deref, path::PathBuf};

use dashmap::{setref::multiple::RefMulti, DashSet as HashSet};
use rayon::prelude::*;
use rspack_error::Result;
use rspack_paths::ArcPath;

use super::FsWatcherIgnored;

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

/// `PathAccessor` provides access to the sets of files, directories, and missing paths.
pub struct PathAccessor<'a> {
  files: &'a HashSet<ArcPath>,
  directories: &'a HashSet<ArcPath>,
  missing: &'a HashSet<ArcPath>,
  incremental_manager: &'a IncrementalManager,
  incremental_directories: &'a IncrementalManager,
  incremental_missing: &'a IncrementalManager,
}

impl<'a> PathAccessor<'a> {
  /// Creates a new `PathAccessor` with references to the sets of files, directories, and missing paths.
  pub fn new(path_manager: &'a PathManager) -> Self {
    Self {
      files: &path_manager.files,
      directories: &path_manager.directories,
      missing: &path_manager.missing,
      incremental_manager: &path_manager.incremental_files,
      incremental_directories: &path_manager.incremental_directories,
      incremental_missing: &path_manager.incremental_missing,
    }
  }

  /// Returns references to the sets of files, directories, and missing paths.
  pub fn files(&self) -> &'a HashSet<ArcPath> {
    self.files
  }

  /// Returns references to the set of directories.
  pub fn directories(&self) -> &'a HashSet<ArcPath> {
    self.directories
  }

  /// Returns references to the set of missing paths.
  pub fn missing(&self) -> &'a HashSet<ArcPath> {
    self.missing
  }

  /// Returns references to the incremental managers for files, directories, and missing paths.
  pub fn incremental_files(&self) -> &'a IncrementalManager {
    self.incremental_manager
  }

  /// Returns references to the incremental manager for directories.
  pub fn incremental_directories(&self) -> &'a IncrementalManager {
    self.incremental_directories
  }

  /// Returns references to the incremental manager for missing paths.
  pub fn incremental_missing(&self) -> &'a IncrementalManager {
    self.incremental_missing
  }

  /// Returns an iterator that combines all files, directories, and missing paths into a single sequence.
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
  pub fn update(
    self,
    paths: &HashSet<ArcPath>,
    incremental_manager: &IncrementalManager,
    ignored: &FsWatcherIgnored,
  ) -> Result<()> {
    let added_paths = self.added;
    let removed_paths = self.removed;

    for should_be_added in added_paths
      .par_iter()
      .map(|added| {
        if ignored.should_be_ignored(added) {
          return None; // Skip ignored paths
        }

        let path = ArcPath::from(PathBuf::from(added));
        Some(path)
      })
      .collect::<Vec<_>>()
      .into_iter()
      .flatten()
    {
      // Insert the path into the set
      paths.insert(should_be_added.clone());
      incremental_manager.insert_added(should_be_added);
    }

    for removed in removed_paths {
      paths.remove(&ArcPath::from(PathBuf::from(&removed)));
      incremental_manager.insert_removed(ArcPath::from(PathBuf::from(&removed)));
    }
    Ok(())
  }
}

#[derive(Default)]
/// `IncrementalManager` is responsible for managing the incremental changes to the sets of added and removed paths.
pub struct IncrementalManager {
  added: HashSet<ArcPath>,
  removed: HashSet<ArcPath>,
}

impl IncrementalManager {
  /// clear the incremental changes.
  pub fn clear(&self) {
    self.added.clear();
    self.removed.clear();
  }

  /// Inserts a path that has been added.
  pub fn insert_added(&self, path: ArcPath) {
    self.added.insert(path);
  }

  /// Inserts a path that has been removed.
  pub fn insert_removed(&self, path: ArcPath) {
    self.removed.insert(path);
  }

  /// Returns a reference to the set of added paths.
  pub fn added(&self) -> &HashSet<ArcPath> {
    &self.added
  }

  /// Returns a reference to the set of removed paths.
  pub fn removed(&self) -> &HashSet<ArcPath> {
    &self.removed
  }
}

/// `PathManager` is responsible for managing the set of files, directories, and missing paths.
#[derive(Default)]
pub struct PathManager {
  pub files: HashSet<ArcPath>,
  pub directories: HashSet<ArcPath>,
  pub missing: HashSet<ArcPath>,
  incremental_files: IncrementalManager,
  incremental_directories: IncrementalManager,
  incremental_missing: IncrementalManager,
  pub ignored: FsWatcherIgnored,
}

impl PathManager {
  /// Create a new `PathManager` with an optional ignored paths filter.
  pub fn new(ignored: FsWatcherIgnored) -> Self {
    Self {
      files: HashSet::new(),
      directories: HashSet::new(),
      missing: HashSet::new(),
      incremental_files: IncrementalManager::default(),
      incremental_directories: IncrementalManager::default(),
      incremental_missing: IncrementalManager::default(),
      ignored,
    }
  }

  pub fn reset(&self) {
    self.incremental_files.clear();
    self.incremental_directories.clear();
    self.incremental_missing.clear();
  }

  /// Update the paths, directories, and missing paths in the `PathManager`.
  pub fn update_paths(
    &self,
    files: PathUpdater,
    directories: PathUpdater,
    missing: PathUpdater,
  ) -> Result<()> {
    files.update(&self.files, &self.incremental_files, &self.ignored)?;
    directories.update(
      &self.directories,
      &self.incremental_directories,
      &self.ignored,
    )?;
    missing.update(&self.missing, &self.incremental_missing, &self.ignored)?;

    Ok(())
  }

  /// Create a new `PathAccessor` to access the current state of paths, directories, and missing paths.
  pub fn access(&self) -> PathAccessor<'_> {
    PathAccessor::new(self)
  }
}

#[cfg(test)]
mod tests {
  use std::path::PathBuf;

  use dashmap::DashSet as HashSet;

  use super::*;

  #[test]
  fn test_updater() {
    let updater = PathUpdater {
      added: vec![
        "src/index.js".to_string(),
        "node_modules/.pnpm/axios/lib/index.js".to_string(),
        ".git/abc/".to_string(),
      ],
      removed: vec![],
    };
    let paths: HashSet<ArcPath> = HashSet::new();
    let ignored = FsWatcherIgnored::Paths(vec![
      "**/.git/**".to_owned(),
      "**/node_modules/**".to_owned(),
    ]);
    let incremental_manager = IncrementalManager::default();

    updater
      .update(&paths, &incremental_manager, &ignored)
      .unwrap();

    let mut path_iter = paths.into_iter();
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

    let mut path_manager = PathManager::default();
    path_manager.files.extend(files);
    path_manager.directories.extend(directories);
    path_manager.missing.extend(missing);

    let accessor = PathAccessor::new(&path_manager);
    let mut all_paths = vec![];

    for path in accessor.all() {
      all_paths.push(path.to_string_lossy().to_string());
    }

    all_paths.sort();

    assert_eq!(all_paths, vec!["src", "src/index.js", "src/page/index.ts"]);
  }

  #[test]
  fn test_manager() {
    let ignored = FsWatcherIgnored::Paths(vec![
      "**/node_modules/**".to_string(),
      "**/.git/**".to_string(),
    ]);
    let path_manager = PathManager::new(ignored);
    let files = PathUpdater {
      added: vec!["src/index.js".to_string()],
      removed: vec![],
    };
    let directories = PathUpdater {
      added: vec!["src/".to_string(), "node_modules/".to_string()],
      removed: vec![],
    };
    let missing = PathUpdater {
      added: vec!["src/page/index.ts".to_string()],
      removed: vec![],
    };
    path_manager
      .update_paths(files, directories, missing)
      .unwrap();

    let accessor = path_manager.access();
    let mut all_paths = accessor
      .all()
      .map(|p| p.to_string_lossy().to_string())
      .collect::<Vec<_>>();

    all_paths.sort();

    assert_eq!(all_paths, vec!["src/", "src/index.js", "src/page/index.ts"]);
  }
}
