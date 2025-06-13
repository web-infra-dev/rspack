use std::{collections::HashSet, path::PathBuf};

use super::{Analyzer, WatchInfo};
use crate::watcher::register::PathRegister;

#[derive(Default)]
pub struct WatcherDirectoriesAnalyzer;

impl Analyzer for WatcherDirectoriesAnalyzer {
  fn analyze(&self, register: &PathRegister) -> Vec<WatchInfo> {
    self
      .find_watch_directories(register)
      .into_iter()
      .map(|path| WatchInfo {
        path,
        mode: notify::RecursiveMode::NonRecursive,
      })
      .collect()
  }
}

impl WatcherDirectoriesAnalyzer {
  fn find_watch_directories(&self, register: &PathRegister) -> HashSet<PathBuf> {
    let mut directories = HashSet::new();
    let all = register.all();
    for p in all {
      let path = p.key();
      if path.is_dir() {
        directories.insert(path.clone());
      } else {
        if let Some(parent) = path.parent() {
          directories.insert(parent.to_path_buf());
        }
      }
    }
    directories
  }
}
