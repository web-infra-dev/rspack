/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is dual-licensed under either the MIT license found in the
 * LICENSE-MIT file in the root directory of this source tree or the Apache
 * License, Version 2.0 found in the LICENSE-APACHE file in the root directory
 * of this source tree. You may select, at your option, one of the
 * above-listed licenses.
 */

use std::{
  collections::hash_map,
  fmt::Write as _,
  mem,
  ops::{Index, IndexMut},
};

use rustc_hash::{FxHashMap, FxHashSet};

use crate::{
  Allocative,
  global_root::roots,
  key::Key,
  visitor::{NodeKind, Visitor, VisitorImpl},
};

/// Node in flamegraph tree.
///
/// Can be written to flamegraph format with [`write`](FlameGraph::write).
#[derive(Debug, Default, Clone)]
pub struct FlameGraph {
  children: FxHashMap<Key, FlameGraph>,
  /// Total size of all children, cached.
  children_size: usize,
  /// Node size excluding children.
  node_size: usize,
}

impl FlameGraph {
  pub fn total_size(&self) -> usize {
    self.node_size + self.children_size
  }

  /// Add another flamegraph to this one.
  pub fn add(&mut self, other: FlameGraph) {
    self.node_size += other.node_size;
    for (key, child) in other.children {
      self.add_child(key, child);
    }
  }

  /// Add a child node to the flamegraph, merging if it already exists.
  pub fn add_child(&mut self, key: Key, child: FlameGraph) {
    self.children_size += child.total_size();
    match self.children.entry(key) {
      hash_map::Entry::Occupied(mut entry) => {
        entry.get_mut().add(child);
      }
      hash_map::Entry::Vacant(entry) => {
        entry.insert(child);
      }
    }
  }

  /// Add size to this node.
  pub fn add_self(&mut self, size: usize) {
    self.node_size += size;
  }

  fn write_flame_graph_impl(&self, stack: &[&str], w: &mut String) {
    if self.node_size != 0 {
      if !stack.is_empty() {
        writeln!(w, "{} {}", stack.join(";"), self.node_size)
          .expect("writing to a String cannot fail");
      } else {
        // Don't care.
      }
    }
    let mut stack = stack.to_vec();
    let mut children = self.children.iter().collect::<Vec<_>>();
    children.sort_by_key(|(key, _)| *key);
    for (key, child) in children {
      stack.push(key);
      child.write_flame_graph_impl(&stack, w);
      stack.pop().expect("the traversal just pushed a child");
    }
  }

  /// Write flamegraph in format suitable for [`flamegraph.pl`] or [inferno].
  ///
  /// [flamegraph.pl]: https://github.com/brendangregg/FlameGraph
  /// [inferno]: https://github.com/jonhoo/inferno
  pub fn write(&self) -> String {
    let mut r = String::new();
    self.write_flame_graph_impl(&[], &mut r);
    r
  }
}

#[derive(Debug)]
pub struct FlameGraphOutput {
  flamegraph: FlameGraph,
  warnings: String,
}

impl FlameGraphOutput {
  /// Flamegraph source, can be fed to `flamegraph.pl` or `inferno`.
  pub fn flamegraph(&self) -> &FlameGraph {
    &self.flamegraph
  }

  /// Warnings. Text file in unspecified format.
  pub fn warnings(&self) -> String {
    self.warnings.clone()
  }
}

#[derive(Default, Eq, PartialEq, Clone, Debug)]
struct TreeData {
  /// Size of this node including children but excluding unique/shared children.
  /// For example for `String` this would be `size_of::<String>()`.
  size: usize,
  /// Size excluding children. This value is output to flamegraph for given stack.
  /// Can be negative if nodes provides sizes incorrectly.
  rem_size: isize,
  /// Whether this node is `Box` something.
  unique: bool,
  /// Child nodes.
  children: FxHashMap<Key, TreeId>,
}

impl TreeData {
  fn inline_children_size(&self) -> isize {
    self.size as isize - self.rem_size
  }
}

struct TreeRef<'a> {
  trees: &'a Trees,
  tree_id: TreeId,
}

impl TreeRef<'_> {
  fn write_flame_graph(&self, stack: &[&str], warnings: &mut String) -> FlameGraph {
    let mut flame_graph = FlameGraph::default();
    let tree = &self.trees[self.tree_id];
    if tree.rem_size > 0 {
      if stack.is_empty() {
        // don't care.
      } else {
        flame_graph.node_size = tree.rem_size as usize;
      }
    } else if tree.rem_size < 0 && !stack.is_empty() {
      writeln!(
        warnings,
        "Incorrect size declaration for node `{}`, size of self: {}, size of inline children: {}",
        stack.join(";"),
        tree.size,
        tree.inline_children_size()
      )
      .expect("writing to a String cannot fail");
    }
    let mut children: Vec<(&Key, &TreeId)> = Vec::from_iter(&tree.children);
    let mut stack = stack.to_vec();
    children.sort_by_key(|(k, _)| *k);
    for (key, child) in children {
      stack.push(key);
      let child = TreeRef {
        trees: self.trees,
        tree_id: *child,
      };
      let child_framegraph = child.write_flame_graph(&stack, warnings);
      flame_graph.add_child(key.clone(), child_framegraph);
      stack.pop().expect("the traversal just pushed a child");
    }
    flame_graph
  }

  fn to_flame_graph(&self) -> (FlameGraph, String) {
    let mut warnings = String::new();
    let flame_graph = self.write_flame_graph(&[], &mut warnings);
    (flame_graph, warnings)
  }
}

#[derive(Debug, Eq, PartialEq)]
struct Tree {
  trees: Trees,
  tree_id: TreeId,
}

impl Tree {
  fn as_ref(&self) -> TreeRef<'_> {
    TreeRef {
      trees: &self.trees,
      tree_id: self.tree_id,
    }
  }

  fn to_flame_graph(&self) -> (FlameGraph, String) {
    self.as_ref().to_flame_graph()
  }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
struct TreeId(usize);

#[derive(Default, Clone, Debug, Eq, PartialEq)]
struct Trees {
  trees: Vec<TreeData>,
}

impl Trees {
  fn new_tree(&mut self) -> TreeId {
    let id = TreeId(self.trees.len());
    self.trees.push(TreeData::default());
    id
  }
}

impl Index<TreeId> for Trees {
  type Output = TreeData;

  fn index(&self, index: TreeId) -> &Self::Output {
    &self.trees[index.0]
  }
}

impl IndexMut<TreeId> for Trees {
  fn index_mut(&mut self, index: TreeId) -> &mut Self::Output {
    &mut self.trees[index.0]
  }
}

#[derive(Clone, Debug)]
struct TreeStack {
  stack: Vec<TreeId>,
  tree: TreeId,
}

struct TreeStackRef<'t, 's> {
  trees: &'t mut Trees,
  stack: &'s mut TreeStack,
}

impl<'t> TreeStackRef<'t, '_> {
  fn current_data(&'t mut self) -> &'t mut TreeData {
    &mut self.trees[self.stack.tree]
  }

  fn down(&mut self, key: Key) {
    self.stack.stack.push(self.stack.tree);
    let next_tree_id = TreeId(self.trees.trees.len());
    let child = match self.trees[self.stack.tree].children.entry(key) {
      hash_map::Entry::Occupied(e) => *e.get(),
      hash_map::Entry::Vacant(e) => {
        e.insert(next_tree_id);
        let child = self.trees.new_tree();
        assert_eq!(child, next_tree_id);
        child
      }
    };
    self.stack.tree = child;
  }

  #[must_use]
  fn up(&mut self) -> bool {
    if let Some(pop) = self.stack.stack.pop() {
      self.stack.tree = pop;
      true
    } else {
      false
    }
  }
}

#[derive(Eq, PartialEq, Hash, Clone, Copy, Debug)]
struct VisitedSharedPointer(*const ());
unsafe impl Send for VisitedSharedPointer {}

/// Build a flamegraph from given root objects.
///
/// # Example
///
/// ```
/// use allocative::{Allocative, FlameGraphBuilder};
///
/// #[derive(Allocative)]
/// struct Foo {
///   data: String,
/// };
///
/// let foo1 = Foo {
///   data: "Hello, world!".to_owned(),
/// };
/// let foo2 = Foo {
///   data: "Another message!".to_owned(),
/// };
///
/// let mut flamegraph = FlameGraphBuilder::default();
/// flamegraph.visit_root(&foo1);
/// flamegraph.visit_root(&foo2);
/// let flamegraph_src = flamegraph.finish().flamegraph();
/// ```
///
/// And now `flamegraph_src` can be fed to either [flamegraph.pl] or [inferno].
///
/// [flamegraph.pl]: https://github.com/brendangregg/FlameGraph
/// [inferno]: https://github.com/jonhoo/inferno
#[derive(Debug)]
pub struct FlameGraphBuilder {
  shared_under_first_owner: bool,
  opaque: std::collections::BTreeSet<&'static str>,
  /// Visited shared pointers.
  visited_shared: FxHashSet<VisitedSharedPointer>,
  /// Tree data storage.
  trees: Trees,
  /// Current node we are processing in `Visitor`.
  current: TreeStack,
  /// Previous stack when entering shared pointer.
  shared: Vec<TreeStack>,
  /// Data root.
  root: TreeId,
  /// Is root visitor created?
  entered_root_visitor: bool,
}

fn _assert_flame_graph_builder_is_send() {
  fn assert_send<T: Send>() {}
  assert_send::<FlameGraphBuilder>();
}

impl Default for FlameGraphBuilder {
  fn default() -> FlameGraphBuilder {
    let mut trees = Trees::default();
    let root = trees.new_tree();
    FlameGraphBuilder {
      trees,
      visited_shared: FxHashSet::default(),
      current: TreeStack {
        stack: Vec::new(),
        tree: root,
      },
      shared: Vec::new(),
      root,
      entered_root_visitor: false,
      shared_under_first_owner: false,
      opaque: Default::default(),
    }
  }
}

impl FlameGraphBuilder {
  /// Attribute each shared allocation to the first owning edge encountered.
  /// Subsequent owners contribute only their pointer, including cycles.
  pub fn with_shared_ownership() -> Self {
    Self {
      shared_under_first_owner: true,
      ..Self::default()
    }
  }

  pub fn root_visitor(&mut self) -> Visitor<'_> {
    assert!(!self.entered_root_visitor);
    self.entered_root_visitor = true;
    Visitor {
      visitor: self,
      node_kind: NodeKind::Root,
    }
  }

  /// Collect tree sizes starting from given root.
  pub fn visit_root(&mut self, root: &dyn Allocative) {
    let mut visitor = self.root_visitor();
    root.visit(&mut visitor);
    visitor.exit();
  }

  /// Collect data from global roots registered with
  /// [`register_root`](crate::register_root).
  pub fn visit_global_roots(&mut self) {
    for root in roots() {
      self.visit_root(root);
    }
  }

  fn finish_impl(mut self) -> Tree {
    assert!(self.shared.is_empty());
    assert!(self.current.stack.is_empty());
    assert!(!self.entered_root_visitor);
    Self::update_sizes(self.root, &mut self.trees);
    Tree {
      trees: self.trees,
      tree_id: self.root,
    }
  }

  /// Finish building the flamegraph.
  pub fn finish(self) -> FlameGraphOutput {
    let opaque = self
      .opaque
      .iter()
      .map(|name| format!("Unobserved nested allocations: {name}\n"))
      .collect::<String>();
    let tree = self.finish_impl();
    let (flamegraph, mut warnings) = tree.to_flame_graph();
    warnings.push_str(&opaque);
    FlameGraphOutput {
      flamegraph,
      warnings,
    }
  }

  /// Finish building the flamegraph and return the flamegraph output.
  pub fn finish_and_write_flame_graph(self) -> String {
    self.finish().flamegraph.write()
  }

  fn update_sizes(tree_id: TreeId, trees: &mut Trees) {
    let tree = &mut trees[tree_id];
    for child in tree.children.values().copied().collect::<Vec<_>>() {
      Self::update_sizes(child, trees);
    }
    let tree = &mut trees[tree_id];
    let children_size = if tree.unique {
      0
    } else {
      let tree = &trees[tree_id];
      tree
        .children
        .values()
        .map(|child| trees[*child].size)
        .sum::<usize>()
    };
    let tree = &mut trees[tree_id];
    let size = tree.size;
    tree.rem_size = (size as isize).saturating_sub(children_size as isize);
  }

  fn current(&mut self) -> TreeStackRef<'_, '_> {
    TreeStackRef {
      trees: &mut self.trees,
      stack: &mut self.current,
    }
  }

  fn exit_impl(&mut self) {
    assert!(self.entered_root_visitor);

    let up = self.current().up();
    if !up {
      if let Some(shared) = self.shared.pop() {
        self.current = shared;
        assert!(self.current().up());
      } else {
        self.entered_root_visitor = false;
      }
    }
  }
}

impl VisitorImpl for FlameGraphBuilder {
  fn opaque_impl(&mut self, name: &'static str) {
    self.opaque.insert(name);
  }

  fn enter_inline_impl(&mut self, name: Key, size: usize, _parent: NodeKind) {
    self.current().down(name);
    self.current().current_data().size += size;
  }

  fn enter_unique_impl(&mut self, name: Key, size: usize, _parent: NodeKind) {
    self.current().down(name);
    self.current().current_data().size += size;
    // TODO: deal with potential issue when node is both unique and not.
    // TODO: record some malloc overhead.
    self.current().current_data().unique = true;
  }

  fn enter_shared_impl(
    &mut self,
    name: Key,
    size: usize,
    ptr: *const (),
    _parent: NodeKind,
  ) -> bool {
    self.current().down(name);
    self.current().current_data().size += size;

    if !self.visited_shared.insert(VisitedSharedPointer(ptr)) {
      self.exit_impl();
      return false;
    }

    if self.shared_under_first_owner {
      self.current().current_data().unique = true;
      return true;
    }
    self.shared.push(mem::replace(
      &mut self.current,
      TreeStack {
        stack: Vec::new(),
        tree: self.root,
      },
    ));
    true
  }

  fn exit_inline_impl(&mut self) {
    self.exit_impl();
  }

  fn exit_unique_impl(&mut self) {
    self.exit_impl();
  }

  fn exit_shared_impl(&mut self) {
    self.exit_impl();
  }

  fn exit_root_impl(&mut self) {
    self.exit_impl();
  }
}
