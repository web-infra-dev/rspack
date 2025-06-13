use std::{path::PathBuf, sync::Arc};

use dashmap::DashSet as HashSet;

use super::StdSender;
use crate::watcher::register::PathRegister;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FsEventKind {
  Change,
  Delete,
}

#[derive(Debug, Clone)]
pub struct FsEvent {
  pub path: PathBuf,
  pub kind: FsEventKind,
}

pub struct DependencyFinder<'a> {
  pub files: &'a HashSet<PathBuf>,
  pub directories: &'a HashSet<PathBuf>,
  pub missing: &'a HashSet<PathBuf>,
}

impl<'a> DependencyFinder<'a> {
  pub fn find_dependency(&self, path: &PathBuf) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    if path.is_dir() && self.contains_directory(path) {
      paths.push(path.clone());
    }
    if path.is_file() && self.contains_file(path) {
      paths.push(path.clone());
    }

    self.recursiron_directories(path, &mut paths);

    paths
  }

  fn contains_file(&self, path: &PathBuf) -> bool {
    self.files.contains(path) || self.missing.contains(path)
  }

  fn contains_directory(&self, path: &PathBuf) -> bool {
    self.directories.contains(path) || self.missing.contains(path)
  }

  fn recursiron_directories(&self, path: &PathBuf, paths: &mut Vec<PathBuf>) {
    match path.parent() {
      Some(parent) => {
        let parent = parent.to_path_buf();
        if self.contains_directory(&parent) {
          paths.push(parent.to_path_buf());
        }
        self.recursiron_directories(&parent, paths);
      }
      None => {
        // Reached the root directory, stop recursion
      }
    }
  }
}

pub struct Trigger {
  register: Arc<PathRegister>,
  tx: StdSender<FsEvent>,
}

impl Trigger {
  pub fn new(register: Arc<PathRegister>, tx: StdSender<FsEvent>) -> Self {
    Self { register, tx }
  }

  pub fn on_event(&self, path: &PathBuf, kind: FsEventKind) {
    let finder = self.finder();
    let dependencies = finder.find_dependency(path);
    for dep in dependencies {
      self.trigger_event(dep, kind);
    }
  }

  fn finder(&self) -> DependencyFinder<'_> {
    DependencyFinder {
      files: self.register.files(),
      directories: self.register.directories(),
      missing: self.register.missing(),
    }
  }

  fn trigger_event(&self, path: PathBuf, kind: FsEventKind) {
    let event = FsEvent { path: path, kind };
    _ = self.tx.send(event);
  }
}
