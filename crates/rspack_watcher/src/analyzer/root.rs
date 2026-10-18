#![allow(unused)]
use rspack_paths::{InternedPath, InternedPathDashMap, InternedPathSet};

use super::{Analyzer, WatchPattern};
use crate::registry::PathAccessor;

#[derive(Default)]
/// The `WatcherRootAnalyzer` is an implementation of the `Analyzer` trait that
/// analyzes the root directory of the file system and determines the common root
/// path to be watched.
pub struct WatcherRootAnalyzer {
  path_tree: PathTree,
}

impl Analyzer for WatcherRootAnalyzer {
  fn analyze<'a>(&self, path_accessor: PathAccessor<'a>) -> Vec<WatchPattern> {
    let sets = [
      path_accessor.files(),
      path_accessor.directories(),
      path_accessor.missing(),
    ];
    let mut changed = InternedPathSet::default();
    for (_, added, removed) in &sets {
      changed.extend(added.iter().chain(removed.iter()).map(|path| path.clone()));
    }
    for path in changed {
      if sets.iter().any(|(all, _, _)| all.contains(&path)) {
        self.path_tree.add_path(&path);
      } else {
        self.path_tree.remove_path(&path);
      }
    }

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
  inner: InternedPathDashMap<TreeNode>,
}

impl PathTree {
  pub fn find_common_root(&self) -> Option<InternedPath> {
    let root = self.find_root()?;
    Some(self.find_common_root_recursive(root))
  }

  fn find_common_root_recursive(&self, path: InternedPath) -> InternedPath {
    let node = self
      .inner
      .get(&path)
      .expect("Path should exist in the tree");
    // We need make sure the path is exists
    assert!(path.exists(), "Path should exist");

    if node.registered {
      return path;
    }
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

  pub fn add_path(&self, path: &InternedPath) {
    self.inner.entry(path.clone()).or_default().registered = true;
    self.add_path_recursive(path);
  }

  pub fn remove_path(&self, path: &InternedPath) {
    let Some(mut node) = self.inner.get_mut(path) else {
      return;
    };
    node.registered = false;
    drop(node);
    let mut path = path.clone();
    loop {
      let removable = self
        .inner
        .get(&path)
        .is_some_and(|node| !node.registered && node.children.is_empty());
      if !removable {
        break;
      }
      self.inner.remove(&path);
      let Some(parent) = path.parent().map(InternedPath::from) else {
        break;
      };
      if let Some(mut node) = self.inner.get_mut(&parent) {
        node.children.remove(&path);
      }
      path = parent;
    }
  }

  fn find_root(&self) -> Option<InternedPath> {
    // Start from the current path and find the root recursively
    let path = self.inner.iter().next()?.key().clone();
    Some(self.find_root_recursive(path))
  }

  fn find_root_recursive(&self, path: InternedPath) -> InternedPath {
    // If the path is already a root, return it

    match path.parent() {
      Some(parent) => {
        // If the parent exists in the tree, continue searching up
        if self.inner.get(&InternedPath::from(parent)).is_some() {
          self.find_root_recursive(InternedPath::from(parent))
        } else {
          path
        }
      }
      None => path,
    }
  }

  fn add_path_recursive(&self, path: &InternedPath) {
    let tree = &self.inner;
    if let Some(parent) = path.parent() {
      if let Some(mut node) = tree.get_mut(&InternedPath::from(parent)) {
        node.add_child(path.clone());
        return;
      }
      let mut parent_node = TreeNode::default();
      parent_node.add_child(path.clone());
      tree.insert(InternedPath::from(parent), parent_node);
      self.add_path_recursive(&InternedPath::from(parent))
    }
  }
}

#[derive(Debug, Default)]
struct TreeNode {
  registered: bool,
  children: InternedPathSet,
}

impl TreeNode {
  fn add_child(&mut self, child: InternedPath) {
    self.children.insert(child);
  }

  fn only_child(&self) -> Option<InternedPath> {
    if self.children.len() == 1 {
      self.children.iter().next().cloned()
    } else {
      None
    }
  }
}

#[cfg(test)]
mod tests {
  use rspack_paths::InternedPath;

  use super::*;
  use crate::registry::Registry;

  #[tokio::test]
  async fn test_find_watch_root() {
    let current_dir = std::env::current_dir().expect("Failed to get current directory");
    let file_0 = InternedPath::from(current_dir.join("Cargo.toml"));
    let file_1 = InternedPath::from(current_dir.join("src/lib.rs"));
    let dir_0 = InternedPath::from(current_dir.clone());
    let dir_1 = InternedPath::from(current_dir.join("src"));
    let registry = Registry::default();
    let files = (vec![file_0, file_1].into_iter(), vec![].into_iter());
    let dirs = (vec![dir_0, dir_1].into_iter(), vec![].into_iter());
    let missing = (vec![].into_iter(), vec![].into_iter());
    registry.update(files, dirs, missing).await.unwrap();

    let analyzer = WatcherRootAnalyzer::default();
    let watch_patterns = analyzer.analyze(registry.access());

    assert_eq!(watch_patterns.len(), 1);
    assert_eq!(watch_patterns[0].path, InternedPath::from(current_dir));
    assert_eq!(watch_patterns[0].mode, notify::RecursiveMode::Recursive);
  }

  #[tokio::test]
  async fn test_find_with_missing() {
    let current_dir = std::env::current_dir().expect("Failed to get current directory");

    let registry = Registry::default();
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

    registry.update(files, dirs, missing).await.unwrap();

    let analyzer = WatcherRootAnalyzer::default();
    let watch_patterns = analyzer.analyze(registry.access());

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
      tree
        .inner
        .get(&dir)
        .is_none_or(|node| !node.children.contains(&leaf)),
      "leaf must be detached from its parent's children"
    );
  }

  #[tokio::test]
  async fn test_analyze_cancels_cross_set_migration() {
    // `some-module` migrates missing -> directory on recovery: in the same cycle
    // it appears in both `missing.removed` and `directories.added`. The
    // final membership must preserve it, keeping the shared tree connected.
    let base = std::env::current_dir().expect("cwd");
    let sm = InternedPath::from(base.join("__mig__").join("some-module"));
    let sub = |s: &str| InternedPath::from(base.join("__mig__").join("some-module").join(s));

    let registry = Registry::default();
    let analyzer = WatcherRootAnalyzer::default();

    // STEP0: some-module is an unresolved missing dependency.
    registry
      .update(
        (std::iter::empty(), std::iter::empty()),
        (std::iter::empty(), std::iter::empty()),
        (std::iter::once(sm.clone()), std::iter::empty()),
      )
      .await
      .unwrap();
    analyzer.analyze(registry.access());
    registry.reset();

    // STEP1: recover. some-module migrates into `directories`; directory
    // resolution adds some-module/{index.js (file), package.json, index
    // (missing)}; some-module leaves `missing`.
    registry
      .update(
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
    let patterns = analyzer.analyze(registry.access());
    assert_eq!(patterns.len(), 1);
    assert!(
      analyzer.path_tree.inner.contains_key(&sm),
      "migrated some-module must be preserved, not deleted"
    );
  }
}
