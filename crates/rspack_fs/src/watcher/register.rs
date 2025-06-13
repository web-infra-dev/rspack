use std::{hash::RandomState, iter::Chain, path::PathBuf};

use async_trait::async_trait;
use dashmap::{iter::Iter, setref::multiple::RefMulti, DashSet as HashSet};

use super::IncrementalPaths;

#[async_trait]
pub trait Ignored: Send + Sync {
  async fn ignore(&self, path: &str) -> bool;
}

pub struct All<'a> {
  chain: Box<dyn Iterator<Item = RefMulti<'a, PathBuf>> + 'a>,
}

impl<'a> All<'a> {
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
      chain: Box::new(chain),
    }
  }
}

impl<'a> Iterator for All<'a> {
  type Item = RefMulti<'a, PathBuf>;

  fn next(&mut self) -> Option<Self::Item> {
    self.chain.next()
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
