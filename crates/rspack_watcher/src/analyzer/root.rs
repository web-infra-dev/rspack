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

    // files / directories / missing share one `path_tree`. On recovery a path
    // can migrate between sets within a single cycle (e.g. the unresolved
    // dependency `node_modules/some-module` becomes a real directory): it then
    // shows up in one set's `added` and another set's `removed`. Applying each
    // set's delta independently lets the stray `removed` delete a node the other
    // set still references — orphaning its children and disconnecting the tree.
    // Merge the three sets first, then cancel migrating paths out of both nets.
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
    // Detach from the PARENT's child set. The previous code removed `path` from
    // its own set (a no-op), leaving a stale child reference on the parent that
    // could later surface as a tree node that no longer exists.
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

  #[test]
  fn test_remove_path_detaches_from_parent() {
    let base = std::env::current_dir().expect("Failed to get current directory");
    let dir = ArcPath::from(base.join("a"));
    let leaf = ArcPath::from(base.join("a").join("b.js"));

    let tree = PathTree::default();
    tree.add_path(&leaf); // builds `a` as an ancestor whose child is `b.js`
    assert!(
      tree
        .inner
        .get(&dir)
        .expect("a present")
        .children
        .contains(&leaf)
    );

    // Removing the leaf must detach it from its PARENT's child set; the old code
    // removed it from its own set (a no-op), leaving a stale child reference.
    tree.remove_path(&leaf);
    assert!(!tree.inner.contains_key(&leaf));
    assert!(
      !tree
        .inner
        .get(&dir)
        .expect("a present")
        .children
        .contains(&leaf),
      "leaf must be detached from its parent's children"
    );
  }

  #[test]
  fn test_analyze_cancels_cross_set_migration() {
    // `some-module` migrates missing -> directory on recovery: in the same cycle
    // it appears in both `missing.removed` and `directories.added`. The
    // union-difference must cancel it, so the shared tree neither re-adds nor
    // deletes it — keeping the tree connected so `find_common_root` cannot land
    // on a non-existent orphan root and panic.
    let base = std::env::current_dir().expect("cwd");
    let sm = ArcPath::from(base.join("__mig__").join("some-module"));
    let sub = |s: &str| ArcPath::from(base.join("__mig__").join("some-module").join(s));

    let pm = PathManager::default();
    let analyzer = WatcherRootAnalyzer::default();

    // STEP0: some-module is an unresolved missing dependency.
    pm.update(
      (std::iter::empty(), std::iter::empty()),
      (std::iter::empty(), std::iter::empty()),
      (std::iter::once(sm.clone()), std::iter::empty()),
    )
    .unwrap();
    analyzer.analyze(pm.access());
    pm.reset();

    // STEP1: recover. some-module migrates into `directories`; directory
    // resolution adds some-module/{index.js (file), package.json, index
    // (missing)}; some-module leaves `missing`.
    pm.update(
      (std::iter::once(sub("index.js")), std::iter::empty()),
      (std::iter::once(sm.clone()), std::iter::empty()),
      (
        vec![sub("package.json"), sub("index")].into_iter(),
        std::iter::once(sm.clone()),
      ),
    )
    .unwrap();

    // Must not panic, and must keep the migrated `some-module` node.
    let patterns = analyzer.analyze(pm.access());
    assert_eq!(patterns.len(), 1);
    assert!(
      analyzer.path_tree.inner.contains_key(&sm),
      "migrated some-module must be preserved, not deleted"
    );
  }
}
