use std::collections::HashMap;

use dashmap::DashSet as HashSet;
use rspack_paths::ArcPath;

// use rspack_util::fx_hash::FxDashMap as HashMap;
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
  fn find_watch_root(&self) -> ArcPath {
    let tree_builder = TreeBuilder::new(self.path_accessor.all());
    let (tree, root) = tree_builder.build_tree();
    find_common_root_from_tree(&tree, &root)
  }
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
fn find_common_root_from_tree(tree: &PathTree, path: &ArcPath) -> ArcPath {
  let node = tree.get(path).expect("Path should exist in the tree");
  // We need make sure the path is exists
  assert!(path.exists(), "Path should exist");

  if let Some(child) = node
    .only_child()
    // Check if the child exists in the tree
    .and_then(|child| if child.exists() { Some(child) } else { None })
  {
    find_common_root_from_tree(tree, &child)
  } else {
    path.clone() // Return the current path if it has no single child
  }
}

type PathTree = HashMap<ArcPath, TreeNode>;

#[derive(Debug, Default)]
struct TreeNode {
  children: HashSet<ArcPath>,
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
  fn build_tree(self) -> (PathTree, ArcPath) {
    let mut tree_map: PathTree = HashMap::default();
    let mut root: Option<ArcPath> = None;

    for path in self.watch_paths {
      if path.exists() && path.is_dir() {
        tree_map.entry(path.clone()).or_default();
      }

      if let Some(r) = Self::build_tree_recursive(&path, &mut tree_map) {
        if root.is_none() {
          root = Some(r);
        }
      }
    }

    (tree_map, root.expect("Root path should exist"))
  }

  fn build_tree_recursive(path: &ArcPath, tree: &mut PathTree) -> Option<ArcPath> {
    match path.parent() {
      Some(parent) => {
        if let Some(node) = tree.get_mut(parent) {
          node.add_child(path.clone());
          None
        } else {
          let parent_node = TreeNode::default();
          parent_node.add_child(path.clone());
          tree.insert(ArcPath::from(parent), parent_node);
          Self::build_tree_recursive(&ArcPath::from(parent), tree)
        }
      }
      None => Some(path.clone()),
    }
  }
}

#[cfg(test)]
mod tests {

  use dashmap::DashSet as HashSet;
  use rspack_paths::ArcPath;

  use super::*;

  #[test]
  fn test_find_watch_root() {
    let current_dir = std::env::current_dir().expect("Failed to get current directory");
    let files = HashSet::with_capacity(3);
    let file_0 = ArcPath::from(current_dir.join("Cargo.toml"));
    let file_1 = ArcPath::from(current_dir.join("src"));
    let file_2 = ArcPath::from(current_dir.join("src/lib.rs"));
    files.insert(file_0.clone());
    files.insert(file_1.clone());
    files.insert(file_2.clone());
    let directories = HashSet::with_capacity(1);
    let dir_0 = ArcPath::from(current_dir.clone());

    directories.insert(dir_0.clone());
    let missing = HashSet::new();

    let path_accessor = PathAccessor::new(&files, &directories, &missing);

    let analyzer = WatcherRootAnalyzer::new(path_accessor);
    let watch_patterns = analyzer.analyze();

    assert_eq!(watch_patterns.len(), 1);
    assert_eq!(watch_patterns[0].path, ArcPath::from(current_dir));
    assert_eq!(watch_patterns[0].mode, notify::RecursiveMode::Recursive);
  }

  #[test]
  fn test_find_with_missing() {
    let current_dir = std::env::current_dir().expect("Failed to get current directory");
    let files = HashSet::new();
    let directories = HashSet::new();
    let missing = HashSet::with_capacity(3);

    let missing_0 = ArcPath::from(current_dir.join("_missing").join("a"));
    let missing_1 = ArcPath::from(current_dir.join("_missing").join("b"));
    let missing_2 = ArcPath::from(current_dir.join("_missing").join("c.js"));
    missing.insert(missing_0);
    missing.insert(missing_1);
    missing.insert(missing_2);

    let path_accessor = PathAccessor::new(&files, &directories, &missing);

    let analyzer = WatcherRootAnalyzer::new(path_accessor);
    let watch_patterns = analyzer.analyze();

    assert_eq!(watch_patterns.len(), 1);
    assert_eq!(watch_patterns[0].path, ArcPath::from(current_dir));
  }
}
