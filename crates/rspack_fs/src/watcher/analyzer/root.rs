use dashmap::DashSet as HashSet;
use rspack_paths::ArcPath;
use rspack_util::fx_hash::FxDashMap as HashMap;

use super::{Analyzer, WatchPattern};
use crate::watcher::path_manager::PathAccessor;

#[derive(Default)]
pub struct WatcherRootAnalyzer {
  path_tree: PathTree,
}

impl Analyzer for WatcherRootAnalyzer {
  fn analyze<'a>(&self, path_accessor: PathAccessor<'a>) -> Vec<WatchPattern> {
    let incremental_files = path_accessor.incremental_files();
    let incremental_directories = path_accessor.incremental_directories();
    let incremental_missing = path_accessor.incremental_missing();

    for added in incremental_files.added().iter() {
      self.path_tree.add_path(&added);
    }

    for removed in incremental_files.removed().iter() {
      self.path_tree.remove_path(&removed);
    }

    for added in incremental_directories.added().iter() {
      self.path_tree.add_path(&added);
    }
    for removed in incremental_directories.removed().iter() {
      self.path_tree.remove_path(&removed);
    }
    for added in incremental_missing.added().iter() {
      self.path_tree.add_path(&added);
    }
    for removed in incremental_missing.removed().iter() {
      self.path_tree.remove_path(&removed);
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
  inner: HashMap<ArcPath, TreeNode>,
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

  pub fn add_path(&self, path: &ArcPath) {
    self.inner.entry(path.clone()).or_default();
    self.add_path_recursive(path);
  }

  pub fn remove_path(&self, path: &ArcPath) {
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
    let parent_path = match path.parent() {
      Some(parent) => {
        // If the parent exists in the tree, continue searching up
        if self.inner.get(&ArcPath::from(parent)).is_some() {
          self.find_root_recursive(ArcPath::from(parent))
        } else {
          path
        }
      }
      None => path,
    };
    parent_path
  }

  fn add_path_recursive(&self, path: &ArcPath) {
    let tree = &self.inner;
    if let Some(parent) = path.parent() {
      if let Some(node) = tree.get_mut(parent) {
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

#[cfg(test)]
mod tests {
  use rspack_paths::ArcPath;

  use super::*;
  use crate::watcher::path_manager::{PathManager, PathUpdater};

  #[tokio::test]
  async fn test_find_watch_root() {
    let current_dir = std::env::current_dir().expect("Failed to get current directory");
    let file_0 = ArcPath::from(current_dir.join("Cargo.toml"));
    let file_1 = ArcPath::from(current_dir.join("src/lib.rs"));
    let dir_0 = ArcPath::from(current_dir.clone());
    let dir_1 = ArcPath::from(current_dir.join("src"));
    let path_manager = PathManager::new(None);
    let file_updater = PathUpdater {
      added: vec![
        file_0.to_string_lossy().to_string(),
        file_1.to_string_lossy().to_string(),
      ],
      removed: vec![],
    };
    let directory_updater = PathUpdater {
      added: vec![
        dir_0.to_string_lossy().to_string(),
        dir_1.to_string_lossy().to_string(),
      ],
      removed: vec![],
    };
    let missing_updater = PathUpdater {
      added: vec![],
      removed: vec![],
    };
    path_manager
      .update_paths(file_updater, directory_updater, missing_updater)
      .await
      .unwrap();

    let analyzer = WatcherRootAnalyzer::default();
    let watch_patterns = analyzer.analyze(path_manager.access());

    assert_eq!(watch_patterns.len(), 1);
    assert_eq!(watch_patterns[0].path, ArcPath::from(current_dir));
    assert_eq!(watch_patterns[0].mode, notify::RecursiveMode::Recursive);
  }

  #[tokio::test]
  async fn test_find_with_missing() {
    let current_dir = std::env::current_dir().expect("Failed to get current directory");

    let path_manager = PathManager::new(None);
    let file_updater = PathUpdater {
      added: vec![],
      removed: vec![],
    };
    let directory_updater = PathUpdater {
      added: vec![],
      removed: vec![],
    };
    let missing_updater = PathUpdater {
      added: vec![
        current_dir
          .join("_missing")
          .join("a")
          .to_string_lossy()
          .to_string(),
        current_dir
          .join("_missing")
          .join("b")
          .to_string_lossy()
          .to_string(),
        current_dir
          .join("_missing")
          .join("c.js")
          .to_string_lossy()
          .to_string(),
      ],
      removed: vec![],
    };

    path_manager
      .update_paths(file_updater, directory_updater, missing_updater)
      .await
      .unwrap();

    let analyzer = WatcherRootAnalyzer::default();
    let watch_patterns = analyzer.analyze(path_manager.access());

    assert_eq!(watch_patterns.len(), 1);
    assert_eq!(watch_patterns[0].path, ArcPath::from(current_dir));
  }
}
