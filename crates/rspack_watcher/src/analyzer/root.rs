#![allow(unused)]
use std::{ops::Deref, path::Path};

use dashmap::DashSet as HashSet;
use rspack_paths::{InternedPath, InternedPathDashMap, InternedPathDashSet, InternedPathSet};
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
    let (all_files, added_files, removed_files) = path_accessor.files();
    let (all_directories, added_directories, removed_directories) = path_accessor.directories();
    let (all_missing, added_missing, removed_missing) = path_accessor.missing();

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
    self
      .path_tree
      .sync_paths(&added, &removed, [all_files, all_directories, all_missing]);

    self.path_tree.find_common_patterns()
  }
}

/// Union of three path sets into a fresh set.
fn union3(
  a: &InternedPathDashSet,
  b: &InternedPathDashSet,
  c: &InternedPathDashSet,
) -> InternedPathSet {
  let mut out = InternedPathSet::default();
  for set in [a, b, c] {
    for path in set.iter() {
      out.insert(path.deref().clone());
    }
  }
  out
}

/// Set difference `a - b` into a fresh set.
fn difference(a: &InternedPathSet, b: &InternedPathSet) -> InternedPathSet {
  let mut out = InternedPathSet::default();
  for path in a {
    if !b.contains(path) {
      out.insert(path.clone());
    }
  }
  out
}

fn is_real_directory(path: &Path) -> bool {
  if path.parent().is_none() {
    path.is_dir()
  } else {
    std::fs::symlink_metadata(path).is_ok_and(|meta| meta.is_dir())
  }
}

#[derive(Debug, Default)]
struct PathTree {
  inner: InternedPathDashMap<TreeNode>,
}

impl PathTree {
  pub fn find_common_patterns(&self) -> Vec<WatchPattern> {
    let mut top_roots = InternedPathSet::default();
    for entry in self.inner.iter() {
      if entry.value().is_explicit {
        top_roots.insert(self.find_root_recursive(entry.key().clone()));
      }
    }
    let mut out = Vec::new();
    let mut seen = InternedPathSet::default();
    for root in top_roots {
      self.collect_roots_recursive(root, &mut out, &mut seen);
    }
    out
  }

  pub fn find_common_roots(&self) -> Vec<InternedPath> {
    self
      .find_common_patterns()
      .into_iter()
      .map(|p| p.path)
      .collect()
  }

  fn has_active_descendant(&self, path: &InternedPath) -> bool {
    let Some(node) = self.inner.get(path) else {
      return false;
    };
    if node.is_explicit {
      return true;
    }
    let children: Vec<InternedPath> = node.children.iter().cloned().collect();
    drop(node);
    children
      .iter()
      .any(|child| self.has_active_descendant(child))
  }

  fn collect_roots_recursive(
    &self,
    path: InternedPath,
    out: &mut Vec<WatchPattern>,
    seen: &mut InternedPathSet,
  ) {
    let Some((is_explicit, children)) = self.inner.get(&path).map(|node| {
      (
        node.is_explicit,
        node.children.iter().cloned().collect::<Vec<_>>(),
      )
    }) else {
      return;
    };

    if !is_real_directory(path.as_path()) {
      if let Some(existing_ancestor) = path
        .ancestors()
        .skip(1)
        .find(|ancestor| is_real_directory(ancestor))
        .map(InternedPath::from)
        && seen.insert(existing_ancestor.clone())
      {
        out.push(WatchPattern {
          path: existing_ancestor,
          mode: notify::RecursiveMode::Recursive,
        });
      }
      return;
    }

    let active_children: Vec<InternedPath> = children
      .into_iter()
      .filter(|child| self.has_active_descendant(child))
      .collect();

    if !is_explicit && active_children.len() == 1 && is_real_directory(active_children[0].as_path())
    {
      self.collect_roots_recursive(
        active_children.into_iter().next().expect("single child"),
        out,
        seen,
      );
      return;
    }

    if seen.insert(path.clone()) {
      out.push(WatchPattern {
        path,
        mode: notify::RecursiveMode::Recursive,
      });
    }
  }

  pub fn sync_paths(
    &self,
    added_paths: &InternedPathSet,
    removed_paths: &InternedPathSet,
    active_sets: [&InternedPathDashSet; 3],
  ) {
    let is_active = |p: &InternedPath| active_sets.iter().any(|set| set.contains(p));
    for added in added_paths {
      if is_active(added) {
        self.add_path(added);
      }
    }
    for removed in removed_paths {
      if !is_active(removed) {
        self.unmark_and_prune(removed);
      }
    }
  }

  pub fn add_path(&self, path: &InternedPath) {
    self.inner.entry(path.clone()).or_default().is_explicit = true;
    self.add_path_recursive(path);
  }

  fn unmark_and_prune(&self, path: &InternedPath) {
    let should_prune = if let Some(mut node) = self.inner.get_mut(path) {
      node.is_explicit = false;
      node.children.is_empty()
    } else {
      false
    };
    if !should_prune {
      return;
    }
    let mut curr = path.clone();
    loop {
      let removable = self
        .inner
        .get(&curr)
        .is_some_and(|n| !n.is_explicit && n.children.is_empty());
      if !removable {
        break;
      }
      self.inner.remove(&curr);
      let Some(parent) = curr.parent().map(InternedPath::from) else {
        break;
      };
      let parent_empty = if let Some(mut parent_node) = self.inner.get_mut(&parent) {
        parent_node.children.remove(&curr);
        !parent_node.is_explicit && parent_node.children.is_empty()
      } else {
        false
      };
      if !parent_empty {
        break;
      }
      curr = parent;
    }
  }

  #[cfg(test)]
  pub fn remove_path(&self, path: &InternedPath) {
    self.inner.remove(path);
    // Detach from the PARENT's child set. The previous code removed `path` from
    // its own set (a no-op), leaving a stale child reference on the parent that
    // could later surface as a tree node that no longer exists.
    if let Some(parent) = path.parent().map(InternedPath::from)
      && let Some(mut parent_node) = self.inner.get_mut(&parent)
    {
      parent_node.children.remove(path);
    }
  }

  fn find_root_recursive(&self, path: InternedPath) -> InternedPath {
    match path.parent() {
      Some(parent) => {
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
  children: InternedPathSet,
  is_explicit: bool,
}

impl TreeNode {
  fn add_child(&mut self, child: InternedPath) {
    self.children.insert(child);
  }
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
