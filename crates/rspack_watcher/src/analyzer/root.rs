#![allow(unused)]
use std::ops::Deref;

use dashmap::DashSet as HashSet;
use rspack_paths::{ArcPath, ArcPathDashMap, ArcPathDashSet};
use rspack_util::fx_hash::FxDashMap as HashMap;

use super::{Analyzer, WatchPattern};
use crate::paths::PathAccessor;

#[derive(Default)]
/// The `WatcherRootAnalyzer` is an implementation of the `Analyzer` trait that
/// analyzes the root directory of the file system and determines the common root
/// path to be watched.
pub(super) struct WatcherRootAnalyzer {
  path_tree: PathTree,
}

impl Analyzer for WatcherRootAnalyzer {
  fn analyze<'a>(&self, path_accessor: PathAccessor<'a>) -> Vec<WatchPattern> {
    let (_, added_files, removed_files) = path_accessor.files();
    let (_, added_directories, removed_directories) = path_accessor.directories();
    let (_, added_missing, removed_missing) = path_accessor.missing();

    self.path_tree.update_paths(added_files, removed_files);
    self
      .path_tree
      .update_paths(added_directories, removed_directories);
    self.path_tree.update_paths(added_missing, removed_missing);

    let common_root = self.path_tree.find_common_root();

    match common_root {
      Some(root) => vec![WatchPattern {
        path: root,
        mode: notify::RecursiveMode::Recursive,
      }],
      None => vec![],
    }
  }
}

#[derive(Debug, Default)]
struct PathTree {
  inner: ArcPathDashMap<TreeNode>,
}

impl PathTree {
  pub(crate) fn find_common_root(&self) -> Option<ArcPath> {
    let root = self.find_root()?;
    Some(self.find_common_root_recursive(root))
  }

  fn find_common_root_recursive(&self, path: ArcPath) -> ArcPath {
    let node = self
      .inner
      .get(&path)
      .expect("Path should exist in the tree");
    // We need make sure the path is exists
    assert!(path.exists(), "Path should exist");

    if let Some(child) = node
      .only_child()
      // Check if the child exists in the tree
      .and_then(|child| if child.is_dir() { Some(child) } else { None })
    {
      self.find_common_root_recursive(child)
    } else {
      path // Return the current path if it has no single child
    }
  }

  pub(crate) fn update_paths(&self, added_paths: &ArcPathDashSet, removed_paths: &ArcPathDashSet) {
    for added in added_paths.iter() {
      self.add_path(added.deref());
    }
    for removed in removed_paths.iter() {
      self.remove_path(removed.deref());
    }
  }

  pub(crate) fn add_path(&self, path: &ArcPath) {
    self.inner.entry(path.clone()).or_default();
    self.add_path_recursive(path);
  }

  pub(crate) fn remove_path(&self, path: &ArcPath) {
    if let Some(node) = self.inner.get(path) {
      node.children.remove(path);
    }
    self.inner.remove(path);
  }

  fn find_root(&self) -> Option<ArcPath> {
    // Start from the current path and find the root recursively
    let path = self.inner.iter().next()?.key().clone();
    Some(self.find_root_recursive(path))
  }

  fn find_root_recursive(&self, path: ArcPath) -> ArcPath {
    // If the path is already a root, return it

    match path.parent() {
      Some(parent) => {
        // If the parent exists in the tree, continue searching up
        if self.inner.get(&ArcPath::from(parent)).is_some() {
          self.find_root_recursive(ArcPath::from(parent))
        } else {
          path
        }
      }
      None => path,
    }
  }

  fn add_path_recursive(&self, path: &ArcPath) {
    let tree = &self.inner;
    if let Some(parent) = path.parent() {
      if let Some(node) = tree.get_mut(&ArcPath::from(parent)) {
        node.add_child(path.clone());
        return;
      }
      let parent_node = TreeNode::default();
      parent_node.add_child(path.clone());
      tree.insert(ArcPath::from(parent), parent_node);
      self.add_path_recursive(&ArcPath::from(parent))
    }
  }
}

#[derive(Debug, Default)]
struct TreeNode {
  children: ArcPathDashSet,
}

impl TreeNode {
  fn add_child(&self, child: ArcPath) {
    self.children.insert(child);
  }

  fn only_child(&self) -> Option<ArcPath> {
    if self.children.len() == 1 {
      self.children.iter().next().map(|c| c.key().clone())
    } else {
      None
    }
  }
}

#[cfg(test)]
mod tests {
  use rspack_paths::ArcPath;

  use super::*;
  use crate::paths::PathManager;

  #[test]
  fn test_find_watch_root() {
    let current_dir = std::env::current_dir().expect("Failed to get current directory");
    let file_0 = ArcPath::from(current_dir.join("Cargo.toml"));
    let file_1 = ArcPath::from(current_dir.join("src/lib.rs"));
    let dir_0 = ArcPath::from(current_dir.clone());
    let dir_1 = ArcPath::from(current_dir.join("src"));
    let path_manager = PathManager::default();
    let files = (vec![file_0, file_1].into_iter(), vec![].into_iter());
    let dirs = (vec![dir_0, dir_1].into_iter(), vec![].into_iter());
    let missing = (vec![].into_iter(), vec![].into_iter());
    path_manager.update(files, dirs, missing).unwrap();

    let analyzer = WatcherRootAnalyzer::default();
    let watch_patterns = analyzer.analyze(path_manager.access());

    assert_eq!(watch_patterns.len(), 1);
    assert_eq!(watch_patterns[0].path, ArcPath::from(current_dir));
    assert_eq!(watch_patterns[0].mode, notify::RecursiveMode::Recursive);
  }

  #[test]
  fn test_find_with_missing() {
    let current_dir = std::env::current_dir().expect("Failed to get current directory");

    let path_manager = PathManager::default();
    let files = (vec![].into_iter(), vec![].into_iter());
    let dirs = (vec![].into_iter(), vec![].into_iter());
    let missing = (
      vec![
        current_dir.join("_missing").join("a").into(),
        current_dir.join("_missing").join("b").into(),
        current_dir.join("_missing").join("c.js").into(),
      ]
      .into_iter(),
      vec![].into_iter(),
    );

    path_manager.update(files, dirs, missing).unwrap();

    let analyzer = WatcherRootAnalyzer::default();
    let watch_patterns = analyzer.analyze(path_manager.access());

    assert_eq!(watch_patterns.len(), 1);
    assert_eq!(watch_patterns[0].path, ArcPath::from(current_dir));
  }
}
