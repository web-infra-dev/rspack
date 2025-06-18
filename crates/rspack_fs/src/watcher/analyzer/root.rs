use std::{collections::HashMap, path::PathBuf};

use dashmap::DashSet as HashSet;

use super::{Analyzer, WatchPattern};
use crate::watcher::manager::{All, PathAccessor};

pub struct WatcherRootAnalyzer<'a> {
  path_accessor: PathAccessor<'a>,
}

impl<'a> Analyzer<'a> for WatcherRootAnalyzer<'a> {
  fn new(path_accessor: PathAccessor<'a>) -> Self {
    Self { path_accessor }
  }

  fn analyze(&self) -> Vec<WatchPattern> {
    let root = self.find_watch_root();
    vec![WatchPattern {
      path: root,
      mode: notify::RecursiveMode::Recursive,
    }]
  }
}

impl<'a> WatcherRootAnalyzer<'a> {
  /// Returns the find watch root of this [`RootFinder`].
  fn find_watch_root(&self) -> PathBuf {
    let tree_builder = TreeBuilder::new(self.path_accessor.all());
    let (tree, root) = tree_builder.build_tree();
    self.find_common_root(&tree, &root)
  }

  /// Finds the deepest common root path from the given path in the tree
  ///
  /// Recursively traverses down the tree while the current path has exactly one child.
  /// Returns the current path when it either has multiple children or no children.
  ///
  /// # Arguments
  /// * `tree` - The path tree structure built from watch paths
  /// * `path` - Current path to check (starts from root)
  ///
  /// # Panics
  /// Panics if the given path doesn't exist in the tree
  fn find_common_root(&self, tree: &PathTree, path: &PathBuf) -> PathBuf {
    let node = tree.get(path).expect("Path should exist in the tree");
    // We need make sure the path is exists
    assert!(path.exists(), "Path should exist");

    if let Some(child) = node
      .only_child()
      // Check if the child exists in the tree
      .and_then(|child| if child.exists() { Some(child) } else { None })
    {
      self.find_common_root(tree, &child)
    } else {
      path.to_path_buf() // Return the current path if it has no single child
    }
  }
}

type PathTree = HashMap<PathBuf, TreeNode>;

#[derive(Debug, Default)]
struct TreeNode {
  children: HashSet<PathBuf>,
}

impl TreeNode {
  fn add_child(&self, child: PathBuf) {
    self.children.insert(child);
  }

  fn only_child(&self) -> Option<PathBuf> {
    if self.children.len() == 1 {
      self.children.iter().next().map(|c| c.key().clone())
    } else {
      None
    }
  }
}

/// A helper struct to build a tree representation of watched paths.
struct TreeBuilder<'a> {
  watch_paths: All<'a>,
}

impl<'a> TreeBuilder<'a> {
  /// Creates a new [`TreeBuilder`] with the given watched paths.
  fn new(watch_paths: All<'a>) -> Self {
    Self { watch_paths }
  }

  /// Builds the tree of watch paths.
  fn build_tree(self) -> (PathTree, PathBuf) {
    let mut tree_map: PathTree = HashMap::new();
    let mut root: Option<PathBuf> = None;

    for p in self.watch_paths {
      let path = p.key();

      if path.exists() && path.is_dir() {
        tree_map
          .entry(path.to_path_buf())
          .or_insert_with(TreeNode::default);
      }

      if let Some(r) = Self::build_tree_recursive(&path, &mut tree_map) {
        if root.is_none() {
          root = Some(r);
        }
      }
    }

    (tree_map, root.expect("Root path should exist"))
  }

  fn build_tree_recursive(path: &PathBuf, tree: &mut PathTree) -> Option<PathBuf> {
    match path.parent() {
      Some(parent) => {
        if let Some(node) = tree.get_mut(parent) {
          node.add_child(path.to_path_buf());
          None
        } else {
          let parent_node = TreeNode::default();
          parent_node.add_child(path.to_path_buf());
          tree.insert(parent.to_path_buf(), parent_node);
          Self::build_tree_recursive(&parent.to_path_buf(), tree)
        }
      }
      None => Some(path.to_path_buf()),
    }
  }
}

#[cfg(test)]
mod tests {
  use dashmap::DashSet as HashSet;

  use super::*;

  #[test]
  fn test_find_watch_root() {
    let current_dir = std::env::current_dir().expect("Failed to get current directory");
    let files = HashSet::with_capacity(3);
    files.insert(current_dir.join("Cargo.toml"));
    files.insert(current_dir.join("src"));
    files.insert(current_dir.join("src/lib.rs"));
    let directories = HashSet::with_capacity(1);
    directories.insert(current_dir.clone());
    let missing = HashSet::new();

    let path_accessor = PathAccessor::new(&files, &directories, &missing);

    let analyzer = WatcherRootAnalyzer::new(path_accessor);
    let watch_patterns = analyzer.analyze();

    assert_eq!(watch_patterns.len(), 1);
    assert_eq!(watch_patterns[0].path, current_dir);
    assert_eq!(watch_patterns[0].mode, notify::RecursiveMode::Recursive);
  }

  #[test]
  fn test_find_with_missing() {
    let current_dir = std::env::current_dir().expect("Failed to get current directory");
    let files = HashSet::new();
    let directories = HashSet::new();
    let missing = HashSet::with_capacity(2);

    missing.insert(current_dir.join("_missing").join("a"));
    missing.insert(current_dir.join("_missing").join("b"));
    missing.insert(current_dir.join("_missing").join("c.js"));

    let path_accessor = PathAccessor::new(&files, &directories, &missing);

    let analyzer = WatcherRootAnalyzer::new(path_accessor);
    let watch_patterns = analyzer.analyze();

    assert_eq!(watch_patterns.len(), 1);
    assert_eq!(watch_patterns[0].path, current_dir);
  }
}
