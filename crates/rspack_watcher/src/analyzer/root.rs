#![allow(unused)]
use rspack_paths::{InternedPath, InternedPathDashMap, InternedPathDashSet, InternedPathSet};

use super::{Analyzer, WatchPattern};
use crate::paths::PathAccessor;

/// Recursive backends can share a registered directory with its descendants.
#[derive(Default)]
pub struct WatcherRootAnalyzer {
  path_tree: PathTree,
}

impl Analyzer for WatcherRootAnalyzer {
  fn analyze(&self, paths: PathAccessor<'_>) -> Vec<WatchPattern> {
    let subscriptions = [paths.files(), paths.directories(), paths.missing()];
    for (_, added, removed) in subscriptions {
      for path in added.iter() {
        self.path_tree.add_path(&path);
      }
      for path in removed.iter() {
        if !subscriptions.iter().any(|(all, _, _)| all.contains(&*path)) {
          self.path_tree.unregister(&path);
        }
      }
    }
    self.path_tree.watch_roots()
  }
}

#[derive(Debug, Default)]
struct PathTree {
  inner: InternedPathDashMap<TreeNode>,
  roots: InternedPathDashSet,
}

impl PathTree {
  fn watch_roots(&self) -> Vec<WatchPattern> {
    let mut watches = Vec::new();
    let mut pending: Vec<_> = self.roots.iter().map(|root| root.clone()).collect();
    while let Some(path) = pending.pop() {
      let node = self.inner.get(&path).expect("registered path tree node");
      // A file or a missing child needs its nearest existing directory.
      // Otherwise separate directory branches need no watch on their common ancestor.
      if node.registered || node.children.iter().any(|child| !child.is_dir()) {
        watches.push(WatchPattern {
          path,
          mode: notify::RecursiveMode::Recursive,
        });
      } else {
        pending.extend(node.children.iter().cloned());
      }
    }
    watches
  }

  fn add_path(&self, path: &InternedPath) {
    self.inner.entry(path.clone()).or_default().registered = true;
    let mut child = path.clone();
    while let Some(parent) = child.parent().map(InternedPath::from) {
      if !self
        .inner
        .entry(parent.clone())
        .or_default()
        .children
        .insert(child)
      {
        return;
      }
      child = parent;
    }
    self.roots.insert(child);
  }

  fn unregister(&self, path: &InternedPath) {
    if let Some(mut node) = self.inner.get_mut(path) {
      node.registered = false;
    }
    for ancestor in path.ancestors().map(InternedPath::from) {
      if self
        .inner
        .get(&ancestor)
        .is_none_or(|node| node.registered || !node.children.is_empty())
      {
        break;
      }
      self.remove_path(&ancestor);
    }
  }

  fn remove_path(&self, path: &InternedPath) {
    self.inner.remove(path);
    if let Some(parent) = path.parent().map(InternedPath::from) {
      if let Some(mut node) = self.inner.get_mut(&parent) {
        node.children.remove(path);
      }
    } else {
      self.roots.remove(path);
    }
  }
}

#[derive(Debug, Default)]
struct TreeNode {
  registered: bool,
  children: InternedPathSet,
}

#[cfg(test)]
mod tests {
  use rspack_paths::InternedPath;

  use super::*;
  use crate::paths::PathManager;

  #[tokio::test]
  async fn test_find_watch_root() {
    let current_dir = std::env::current_dir().expect("Failed to get current directory");
    let file_0 = InternedPath::from(current_dir.join("Cargo.toml"));
    let file_1 = InternedPath::from(current_dir.join("src/lib.rs"));
    let dir_0 = InternedPath::from(current_dir.clone());
    let dir_1 = InternedPath::from(current_dir.join("src"));
    let path_manager = PathManager::default();
    let files = (vec![file_0, file_1].into_iter(), vec![].into_iter());
    let dirs = (vec![dir_0, dir_1].into_iter(), vec![].into_iter());
    let missing = (vec![].into_iter(), vec![].into_iter());
    path_manager.update(files, dirs, missing).await.unwrap();

    let analyzer = WatcherRootAnalyzer::default();
    let watch_patterns = analyzer.analyze(path_manager.access());

    assert_eq!(watch_patterns.len(), 1);
    assert_eq!(watch_patterns[0].path, InternedPath::from(current_dir));
    assert_eq!(watch_patterns[0].mode, notify::RecursiveMode::Recursive);
  }

  #[tokio::test]
  async fn test_find_with_missing() {
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

    path_manager.update(files, dirs, missing).await.unwrap();

    let analyzer = WatcherRootAnalyzer::default();
    let watch_patterns = analyzer.analyze(path_manager.access());

    assert_eq!(watch_patterns.len(), 1);
    assert_eq!(watch_patterns[0].path, InternedPath::from(current_dir));
  }

  #[test]
  fn test_remove_path_detaches_from_parent() {
    let base = std::env::current_dir().expect("Failed to get current directory");
    let dir = InternedPath::from(base.join("a"));
    let leaf = InternedPath::from(base.join("a").join("b.js"));

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

  #[tokio::test]
  async fn test_analyze_cancels_cross_set_migration() {
    // `some-module` migrates missing -> directory on recovery: in the same cycle
    // it appears in both `missing.removed` and `directories.added`. The
    // union-difference must cancel it, so the shared tree neither re-adds nor
    // deletes it — keeping the tree connected so `find_common_root` cannot land
    // on a non-existent orphan root and panic.
    let base = std::env::current_dir().expect("cwd");
    let sm = InternedPath::from(base.join("__mig__").join("some-module"));
    let sub = |s: &str| InternedPath::from(base.join("__mig__").join("some-module").join(s));

    let pm = PathManager::default();
    let analyzer = WatcherRootAnalyzer::default();

    // STEP0: some-module is an unresolved missing dependency.
    pm.update(
      (std::iter::empty(), std::iter::empty()),
      (std::iter::empty(), std::iter::empty()),
      (std::iter::once(sm.clone()), std::iter::empty()),
    )
    .await
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
    .await
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
