use std::{collections::HashMap, path::PathBuf};

use super::{Analyzer, WatchTarget};
use crate::watcher::register::{All, PathRegister};

#[derive(Default)]
pub struct WatcherRootAnalyzer;

impl Analyzer for WatcherRootAnalyzer {
  fn analyze(&self, register: &PathRegister) -> Vec<WatchTarget> {
    let root = self.find_watch_root(register);
    vec![WatchTarget {
      path: root,
      mode: notify::RecursiveMode::Recursive,
    }]
  }
}

impl WatcherRootAnalyzer {
  /// Returns the find watch root of this [`RootFinder`].
  fn find_watch_root(&self, register: &PathRegister) -> PathBuf {
    let tree_builder = TreeBuilder::new(register.all());
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
    if let Some(child) = node.only_child() {
      self.find_common_root(tree, child)
    } else {
      path.to_path_buf() // Return the current path if it has no single child
    }
  }
}

type PathTree = HashMap<PathBuf, TreeNode>;

#[derive(Debug, Default)]
struct TreeNode {
  children: Vec<PathBuf>,
}

impl TreeNode {
  fn add_child(&mut self, child: PathBuf) {
    self.children.push(child);
  }

  fn only_child(&self) -> Option<&PathBuf> {
    if self.children.len() == 1 {
      self.children.first()
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

      if path.is_dir() {
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
          let mut parent_node = TreeNode::default();
          parent_node.add_child(path.to_path_buf());
          tree.insert(parent.to_path_buf(), parent_node);
          Self::build_tree_recursive(&parent.to_path_buf(), tree)
        }
      }
      None => Some(path.to_path_buf()),
    }
  }
}
