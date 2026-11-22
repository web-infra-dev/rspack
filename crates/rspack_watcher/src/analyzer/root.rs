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
    let (files, added_files, removed_files) = path_accessor.files();
    let (directories, added_directories, removed_directories) = path_accessor.directories();
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
    self
      .path_tree
      .set_boundaries(files, directories, path_accessor.missing().0);

    self
      .path_tree
      .find_common_roots()
      .into_iter()
      .map(|root| {
        let unfollowed = self.path_tree.is_unfollowed_link(&root);
        let path = if unfollowed {
          root
            .parent()
            .map_or_else(|| root.clone(), InternedPath::from)
        } else {
          root
        };
        WatchPattern {
          // A link whose target is not watched must not be followed by the
          // backend: watching it recursively would reach the ignored target.
          // FSEvents still reports through a non-recursive link watch, so the
          // link's *parent* is watched instead — it sees the link itself being
          // created, removed, or renamed.
          mode: if unfollowed {
            notify::RecursiveMode::NonRecursive
          } else {
            notify::RecursiveMode::Recursive
          },
          path,
        }
      })
      .collect()
  }

  fn set_unfollowed_links(&self, links: &InternedPathSet) {
    self.path_tree.set_unfollowed_links(links);
  }
}

/// Union of three path sets into a fresh set.
fn union3(
  a: &InternedPathDashSet,
  b: &InternedPathDashSet,
  c: &InternedPathDashSet,
) -> InternedPathDashSet {
  let out = InternedPathDashSet::default();
  for set in [a, b, c] {
    for path in set.iter() {
      out.insert(path.deref().clone());
    }
  }
  out
}

/// Set difference `a - b` into a fresh set.
fn difference(a: &InternedPathDashSet, b: &InternedPathDashSet) -> InternedPathDashSet {
  let out = InternedPathDashSet::default();
  for path in a.iter() {
    if !b.contains(path.deref()) {
      out.insert(path.deref().clone());
    }
  }
  out
}

/// Whether two paths share an ancestor worth watching recursively: one that is
/// not a filesystem top level (`/`, `/Users`, `/private`, `/var`, `/tmp` on
/// Unix; a drive root on Windows). Two repositories under one home directory
/// qualify; `/Users/me/app` and `/private/tmp/pkg` do not.
fn share_watchable_ancestor(a: &InternedPath, b: &InternedPath) -> bool {
  let b_ancestors: Vec<InternedPath> = b.ancestors().map(InternedPath::from).collect();
  a.ancestors()
    .map(InternedPath::from)
    .any(|ancestor| b_ancestors.contains(&ancestor) && !is_top_level(&ancestor))
}

fn is_top_level(path: &Path) -> bool {
  let depth = path.ancestors().count();
  if cfg!(windows) {
    return depth <= 1;
  }
  depth <= 2 || is_system_top_level(path)
}

/// Directories every macOS/Linux box has directly under `/`; watching one
/// recursively is watching most of the filesystem.
fn is_system_top_level(path: &Path) -> bool {
  matches!(
    path.to_str(),
    Some("/Users" | "/private" | "/var" | "/tmp" | "/home" | "/usr" | "/etc" | "/opt" | "/mnt")
  )
}

/// The deepest directory containing every path in `group`.
fn common_root(group: &[InternedPath]) -> InternedPath {
  let Some(first) = group.first() else {
    unreachable!("a group is never empty")
  };
  let mut common = first.clone();
  for path in &group[1..] {
    while !path.starts_with(common.as_ref() as &Path) {
      let Some(parent) = common.parent().map(InternedPath::from) else {
        return common;
      };
      if parent == common {
        return common;
      }
      common = parent;
    }
  }
  common
}

#[derive(Debug, Default)]
struct PathTree {
  inner: InternedPathDashMap<TreeNode>,
  /// Paths the watch must not shrink below: a registered context itself, or
  /// the directory holding a registered file or missing dependency. A single
  /// child below one of them cannot pull the recursive root down with it.
  boundaries: InternedPathDashSet,
  /// Registered links whose physical target is not watched (it is ignored).
  /// They are watched themselves, never followed.
  unfollowed_links: InternedPathDashSet,
}

impl PathTree {
  /// Recompute the protected boundaries from the current registration sets.
  /// A boundary is a registered context itself, or the nearest existing
  /// directory above a registered file or missing dependency: a missing path's
  /// own parent may not exist either, and watching it is not possible.
  pub fn set_boundaries(
    &self,
    files: &InternedPathDashSet,
    directories: &InternedPathDashSet,
    missing: &InternedPathDashSet,
  ) {
    self.boundaries.clear();
    for path in directories.iter() {
      self.boundaries.insert(path.deref().clone());
    }
    // A link whose target is ignored is a watch boundary of its own: it must
    // not be merged into a parent root, or the backend would follow it.
    for link in self.unfollowed_links.iter() {
      self.boundaries.insert(link.clone());
    }
    for path in files.iter().chain(missing.iter()) {
      // A link whose target is ignored is watched on its own; watching its
      // parent directory recursively would follow it into the ignored target.
      if self.unfollowed_links.contains(path.deref()) {
        continue;
      }
      let Some(parent) = path.parent().map(InternedPath::from) else {
        continue;
      };
      let existing = std::iter::once(parent.clone())
        .chain(parent.ancestors().skip(1).map(InternedPath::from))
        .find(|ancestor| ancestor.is_dir())
        .unwrap_or(parent);
      self.boundaries.insert(existing);
    }
  }

  /// Whether `path` is a link whose target is not in the physical watch set.
  /// The analyzer only sees physical paths, so a registered link whose target
  /// was dropped as ignored appears here on its own.
  pub fn is_unfollowed_link(&self, path: &InternedPath) -> bool {
    self.unfollowed_links.contains(path)
  }

  /// Record the links the caller decided not to follow.
  pub fn set_unfollowed_links(&self, links: &InternedPathSet) {
    self.unfollowed_links.clear();
    for link in links {
      self.unfollowed_links.insert(link.clone());
    }
  }

  /// Whether `path` must not be followed by the backend: it is a link whose
  /// target is ignored.
  fn is_unfollowed(&self, path: &InternedPath) -> bool {
    self.unfollowed_links.contains(path)
  }

  /// One root per disjoint registered subtree. Boundaries sharing a common
  /// ancestor below the filesystem's top level collapse into that ancestor, so
  /// a normal project still gets a single recursive watch; only boundaries
  /// that would force a watch of `/`, `/Users`, `C:\` and the like — a link to
  /// another drive or repository — stay split.
  pub fn find_common_roots(&self) -> Vec<InternedPath> {
    let mut boundaries: Vec<InternedPath> = self.boundaries.iter().map(|p| p.clone()).collect();
    boundaries.sort_by(|a, b| a.as_ref().cmp(b.as_ref()));
    // A boundary already inside another one adds nothing: the outer recursive
    // watch covers it.
    let mut outermost: Vec<InternedPath> = Vec::with_capacity(boundaries.len());
    for boundary in boundaries {
      // An unfollowed link is never covered by an outer root: the outer watch
      // would follow it.
      let protected = self.is_unfollowed(&boundary);
      if !protected
        && outermost
          .iter()
          .any(|outer| boundary.starts_with(outer.as_ref() as &Path))
      {
        continue;
      }
      if !protected {
        outermost.retain(|outer| !outer.starts_with(boundary.as_ref() as &Path));
      }
      outermost.push(boundary);
    }
    let mut groups: Vec<Vec<InternedPath>> = Vec::new();
    for boundary in outermost {
      match groups
        .iter_mut()
        .find(|group| share_watchable_ancestor(&group[0], &boundary))
      {
        Some(group) => group.push(boundary),
        None => groups.push(vec![boundary]),
      }
    }
    groups
      .into_iter()
      .flat_map(|group| {
        // A group with one boundary can shrink to its deepest single-child
        // directory, as before. Several boundaries stay separate: collapsing
        // them into their common ancestor would recursively watch a directory
        // that contains unrelated trees (a sibling checkout, a temp link
        // target), which both wastes watches and overlaps another watcher's
        // own root.
        if group.len() == 1 {
          vec![self.find_common_root_recursive(group[0].clone())]
        } else {
          group
            .into_iter()
            .map(|boundary| self.find_common_root_recursive(boundary))
            .collect()
        }
      })
      .collect()
  }

  fn find_common_root_recursive(&self, path: InternedPath) -> InternedPath {
    // A boundary is a root on its own, whether or not the tree indexed it: an
    // unfollowed link's parent may hold no watched path of its own.
    if self.boundaries.contains(&path) {
      return path;
    }

    let Some(node) = self.inner.get(&path) else {
      return path;
    };

    // We need make sure the path is exists. A missing dependency's directory
    // may not, and then it cannot pull the root down with it.
    if !path.exists() {
      return path;
    }

    if let Some(child) = node
      .value()
      .only_child()
      // Check if the child exists in the tree
      .filter(|child| child.is_dir())
    {
      self.find_common_root_recursive(child)
    } else {
      path // Return the current path if it has no single child
    }
  }

  pub fn update_paths(
    &self,
    added_paths: &InternedPathDashSet,
    removed_paths: &InternedPathDashSet,
  ) {
    for added in added_paths.iter() {
      self.add_path(added.deref());
    }
    for removed in removed_paths.iter() {
      self.remove_path(removed.deref());
    }
  }

  pub fn add_path(&self, path: &InternedPath) {
    self.inner.entry(path.clone()).or_default();
    self.add_path_recursive(path);
  }

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
