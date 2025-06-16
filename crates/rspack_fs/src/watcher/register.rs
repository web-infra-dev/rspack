use std::path::PathBuf;

use async_trait::async_trait;
use dashmap::{setref::multiple::RefMulti, DashSet as HashSet};

use super::IncrementalPaths;

#[async_trait]
pub trait Ignored: Send + Sync {
  async fn ignore(&self, path: &str) -> bool;
}

/// An iterator that chains together references to all files, directories, and missing paths
/// stored in the PathRegister. This allows iteration over all registered paths as a single sequence.
pub struct All<'a> {
  inner: Box<dyn Iterator<Item = RefMulti<'a, PathBuf>> + 'a>,
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
    files: &'a HashSet<PathBuf>,
    directories: &'a HashSet<PathBuf>,
    missing: &'a HashSet<PathBuf>,
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
  type Item = RefMulti<'a, PathBuf>;

  fn next(&mut self) -> Option<Self::Item> {
    self.inner.next()
  }
}

#[derive(Default)]
pub struct PathRegister {
  pub files: HashSet<PathBuf>,
  pub directories: HashSet<PathBuf>,
  pub missing: HashSet<PathBuf>,
  pub ignored: Option<Box<dyn Ignored>>,
}

impl PathRegister {
  pub fn new(ignored: Option<Box<dyn Ignored>>) -> Self {
    Self {
      files: HashSet::new(),
      directories: HashSet::new(),
      missing: HashSet::new(),
      ignored,
    }
  }

  pub async fn save(
    &self,
    files: IncrementalPaths,
    directories: IncrementalPaths,
    missing: IncrementalPaths,
  ) {
    self._save(&self.files, files).await;
    self._save(&self.directories, directories).await;
    self._save(&self.missing, missing).await;
  }

  pub fn files(&self) -> &HashSet<PathBuf> {
    &self.files
  }

  pub fn directories(&self) -> &HashSet<PathBuf> {
    &self.directories
  }

  pub fn missing(&self) -> &HashSet<PathBuf> {
    &self.missing
  }

  pub fn all(&self) -> All<'_> {
    All::new(&self.files, &self.directories, &self.missing)
  }

  async fn _save(&self, register: &HashSet<PathBuf>, incremental: IncrementalPaths) {
    let added_paths = incremental.0;
    let removed_paths = incremental.1;

    for dep in added_paths {
      if let Some(ignored) = &self.ignored {
        if ignored.ignore(&dep).await {
          continue;
        }
      }

      register.insert(PathBuf::from(dep));
    }

    for dep in removed_paths {
      register.remove(&PathBuf::from(dep));
    }
  }
}

// mod tests {
//   use super::*;
//   use crate::watcher::IncrementalPaths;

//   #[test]
//   async fn test_path_register() {
//     let register = PathRegister::new(None);
//     let files = IncrementalPaths(vec!["file1.txt".to_string()], vec![]);
//     let directories = IncrementalPaths(vec!["dir1".to_string()], vec![]);
//     let missing = IncrementalPaths(vec![], vec!["missing.txt".to_string()]);

//     tokio::runtime::Runtime::new().unwrap().block_on(async {
//       register.save(files, directories, missing).await;
//     });

//     assert!(register.files.contains(&PathBuf::from("file1.txt")));
//     assert!(register.directories.contains(&PathBuf::from("dir1")));
//     assert!(register.missing.contains(&PathBuf::from("missing.txt")));
//   }
// }
