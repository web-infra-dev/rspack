use std::{collections::HashMap, path::PathBuf};

use super::{Analyzer, WatchInfo};
use crate::watcher::register::PathRegister;

#[derive(Default)]
pub struct WatcherRootAnalyzer;

impl Analyzer for WatcherRootAnalyzer {
  fn analyze(&self, register: &PathRegister) -> Vec<WatchInfo> {
    let root = self.find_watch_root(register);
    vec![WatchInfo {
      path: root,
      mode: notify::RecursiveMode::Recursive,
    }]
  }
}

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

type PathTree = HashMap<PathBuf, TreeNode>;

impl WatcherRootAnalyzer {
  /// Returns the find watch root of this [`RootFinder`].
  fn find_watch_root(&self, register: &PathRegister) -> PathBuf {
    let (tree, root) = self.build_tree(register);
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

  fn build_tree(&self, register: &PathRegister) -> (PathTree, PathBuf) {
    let mut tree: PathTree = HashMap::new();
    let mut root: Option<PathBuf> = None;

    let all = register.all();
    for p in all {
      let path = p.key();

      if path.is_dir() {
        tree.insert(path.to_path_buf(), TreeNode::default());
      }

      if let Some(_root) = self.build_tree_recursive(path, &mut tree) {
        if root.is_none() {
          root = Some(_root);
        }
      }
    }

    (tree, root.unwrap())
  }

  fn build_tree_recursive(&self, path: &PathBuf, tree: &mut PathTree) -> Option<PathBuf> {
    match path.parent() {
      Some(parent) => {
        if let Some(node) = tree.get_mut(parent) {
          node.add_child(path.to_path_buf());
        } else {
          // If the parent is not in the tree, we need to add it
          let parent_node = TreeNode {
            children: vec![path.to_path_buf()],
          };

          tree.insert(parent.to_path_buf(), parent_node);

          return self.build_tree_recursive(&parent.to_path_buf(), tree);
        }
        None
      }
      None => {
        Some(path.to_path_buf()) // If there is no parent, we are at the rooth
      }
    }
  }
}
