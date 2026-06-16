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
pub struct WatcherRootAnalyzer {
  path_tree: PathTree,
}

impl Analyzer for WatcherRootAnalyzer {
  fn analyze<'a>(&self, path_accessor: PathAccessor<'a>) -> Vec<WatchPattern> {
    let (_, added_files, removed_files) = path_accessor.files();
    let (_, added_directories, removed_directories) = path_accessor.directories();
    let (_, added_missing, removed_missing) = path_accessor.missing();

    // [FIX ①] Cancel cross-set migration: union the three sets' added/removed,
    // then update with adds-removes / removes-adds, so a path migrating
    // missing->directory in one cycle is not deleted by the stray removed.
    let adds = union3(added_files, added_directories, added_missing);
    let removes = union3(removed_files, removed_directories, removed_missing);
    let added = difference(&adds, &removes);
    let removed = difference(&removes, &adds);
    self.path_tree.update_paths(&added, &removed);

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

/// Union of three path sets into a fresh set.
fn union3(a: &ArcPathDashSet, b: &ArcPathDashSet, c: &ArcPathDashSet) -> ArcPathDashSet {
  let out = ArcPathDashSet::default();
  for set in [a, b, c] {
    for path in set.iter() {
      out.insert(path.deref().clone());
    }
  }
  out
}

/// Set difference `a - b` into a fresh set.
fn difference(a: &ArcPathDashSet, b: &ArcPathDashSet) -> ArcPathDashSet {
  let out = ArcPathDashSet::default();
  for path in a.iter() {
    if !b.contains(path.deref()) {
      out.insert(path.deref().clone());
    }
  }
  out
}

#[derive(Debug, Default)]
struct PathTree {
  inner: ArcPathDashMap<TreeNode>,
}

impl PathTree {
  pub fn find_common_root(&self) -> Option<ArcPath> {
    let root = self.find_root()?;
    // [WATCHER_ROOT_DEBUG] Temporary instrumentation to capture the real tree
    // state behind the Windows-only `assert!(path.exists())` panic. Emits a
    // one-line summary every cycle whenever the tree holds non-existent or
    // orphaned nodes, so we still see the forest/missing state even when the
    // unordered `find_root` pick does not happen to hit the panic.
    self.debug_summary(&root);
    Some(self.find_common_root_recursive(root, 0))
  }

  fn find_common_root_recursive(&self, path: ArcPath, depth: usize) -> ArcPath {
    let node = self
      .inner
      .get(&path)
      .expect("Path should exist in the tree");
    // [DEBUG] With fix ① the tree stays connected and this never fires; if it
    // does, dump the tree right before the original assert so CI shows why.
    if !path.exists() {
      self.debug_dump(&path, depth);
    }
    assert!(path.exists(), "Path should exist");

    if let Some(child) = node
      .only_child()
      // Check if the child exists in the tree
      .and_then(|child| if child.is_dir() { Some(child) } else { None })
    {
      self.find_common_root_recursive(child, depth + 1)
    } else {
      path // Return the current path if it has no single child
    }
  }

  fn parent_in_tree(&self, path: &ArcPath) -> Option<bool> {
    path
      .parent()
      .map(|parent| self.inner.contains_key(&ArcPath::from(parent)))
  }

  fn debug_summary(&self, root: &ArcPath) {
    let total = self.inner.len();
    let mut missing = 0usize;
    let mut orphans = 0usize;
    for entry in self.inner.iter() {
      let path = entry.key();
      if !path.exists() {
        missing += 1;
      }
      // An orphan = a node that has a parent which is NOT in the tree (i.e. the
      // tree got disconnected into a forest). FS roots have no parent and don't
      // count.
      if matches!(self.parent_in_tree(path), Some(false)) {
        orphans += 1;
      }
    }
    if missing > 0 || orphans > 0 {
      eprintln!(
        "[WATCHER_ROOT_DEBUG] cycle: total={total} missing_on_disk={missing} orphan_subtree_roots={orphans} find_root={:?} find_root_exists={}",
        root,
        root.exists(),
      );
    }
  }

  fn debug_dump(&self, trigger: &ArcPath, depth: usize) {
    let entry = if depth == 0 {
      "find_root result (forest / missing root)"
    } else {
      "descend step (possible TOCTOU delete)"
    };
    eprintln!("[WATCHER_ROOT_DEBUG] ===== panic-about-to-fire dump begin =====");
    eprintln!(
      "[WATCHER_ROOT_DEBUG] trigger={:?} exists={} is_dir={} depth={depth} entry={entry} parent_in_tree={:?}",
      trigger,
      trigger.exists(),
      trigger.is_dir(),
      self.parent_in_tree(trigger),
    );
    eprintln!(
      "[WATCHER_ROOT_DEBUG] find_root={:?} node_count={}",
      self.find_root(),
      self.inner.len(),
    );
    for item in self.inner.iter() {
      let path = item.key();
      eprintln!(
        "[WATCHER_ROOT_DEBUG]   node={:?} exists={} is_dir={} children={} parent_in_tree={:?}",
        path,
        path.exists(),
        path.is_dir(),
        item.value().children.len(),
        self.parent_in_tree(path),
      );
    }
    eprintln!("[WATCHER_ROOT_DEBUG] ===== panic-about-to-fire dump end =====");
  }

  pub fn update_paths(&self, added_paths: &ArcPathDashSet, removed_paths: &ArcPathDashSet) {
    for added in added_paths.iter() {
      self.add_path(added.deref());
    }
    for removed in removed_paths.iter() {
      self.remove_path(removed.deref());
    }
  }

  pub fn add_path(&self, path: &ArcPath) {
    self.inner.entry(path.clone()).or_default();
    self.add_path_recursive(path);
  }

  pub fn remove_path(&self, path: &ArcPath) {
    self.inner.remove(path);
    // [FIX ③] Detach from the PARENT's child set (the old code removed `path`
    // from its own set, a no-op, leaving a stale child reference on the parent).
    if let Some(parent) = path.parent().map(ArcPath::from)
      && let Some(parent_node) = self.inner.get(&parent)
    {
      parent_node.children.remove(path);
    }
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
