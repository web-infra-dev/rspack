// Port of https://github.com/webpack/webpack/blob/main/lib/util/findGraphRoots.js

use std::{hash::Hash, sync::atomic::AtomicU32};

use rspack_collections::{Database, DatabaseItem, ItemUkey, Ukey, UkeySet};
use rustc_hash::{FxHashMap, FxHashSet};

#[allow(clippy::enum_variant_names)]
enum Marker {
  NoMarker,
  InProgressMarker,
  DoneMarker,
  DoneMaybeRootCycleMarker,
  DoneAndRootMarker,
}

static NEXT_CYCLE_UKEY: AtomicU32 = AtomicU32::new(0);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct CycleUkey<T: Hash + Eq + Copy>(Ukey, std::marker::PhantomData<Cycle<T>>);

impl<T: Hash + Eq + Copy> ItemUkey for CycleUkey<T> {
  fn ukey(&self) -> Ukey {
    self.0
  }
}

impl<T: Hash + Eq + Copy> CycleUkey<T> {
  pub(crate) fn new() -> Self {
    Self(
      NEXT_CYCLE_UKEY
        .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        .into(),
      std::marker::PhantomData,
    )
  }
}

struct Cycle<T: Hash + Eq + Copy> {
  pub ukey: CycleUkey<T>,
  pub nodes: FxHashSet<T>,
}

impl<T: Hash + Eq + Copy> Default for Cycle<T> {
  fn default() -> Self {
    Self {
      ukey: CycleUkey::<T>::new(),
      nodes: Default::default(),
    }
  }
}

impl<T: Hash + Eq + Copy> DatabaseItem for Cycle<T> {
  type ItemUkey = CycleUkey<T>;
  fn ukey(&self) -> Self::ItemUkey {
    self.ukey
  }
}

static NEXT_NODE_UKEY: AtomicU32 = AtomicU32::new(0);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct NodeUkey<T: Hash + Eq + Copy>(Ukey, std::marker::PhantomData<Node<T>>);

impl<T: Hash + Eq + Copy> NodeUkey<T> {
  pub(crate) fn new() -> Self {
    Self(
      NEXT_NODE_UKEY
        .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        .into(),
      std::marker::PhantomData,
    )
  }
}

struct Node<T: Hash + Eq + Copy> {
  pub ukey: NodeUkey<T>,
  pub item: T,
  pub dependencies: Vec<NodeUkey<T>>,
  pub marker: Marker,
  pub cycle: Option<CycleUkey<NodeUkey<T>>>,
  pub incoming: usize,
}

impl<T: Hash + Eq + Copy> DatabaseItem for Node<T> {
  type ItemUkey = NodeUkey<T>;

  fn ukey(&self) -> Self::ItemUkey {
    self.ukey
  }
}

impl<T: Hash + Eq + Copy> Node<T> {
  fn new(item: T) -> Self {
    Self {
      ukey: NodeUkey::new(),
      item,
      dependencies: Default::default(),
      marker: Marker::NoMarker,
      incoming: 0,
      cycle: None,
    }
  }
}

struct StackEntry<T> {
  node: T,
  open_edges: Vec<T>,
}

pub fn find_graph_roots<
  Item: Clone + Copy + std::fmt::Debug + PartialEq + Eq + Hash + Send + Sync + Ord + 'static,
>(
  items: Vec<Item>,
  get_dependencies: impl Sync + Fn(Item) -> Vec<Item>,
) -> Vec<Item> {
  use rayon::prelude::*;
  // early exit when there is only a single item
  if items.len() <= 1 {
    return items;
  }

  let mut db = Database::<Node<Item>>::new();
  let mut cycle_db = Database::<Cycle<NodeUkey<Item>>>::new();

  items
    .into_iter()
    .map(|item| Node::new(item))
    .for_each(|node| {
      db.add(node);
    });

  let item_to_node_ukey = db
    .values()
    .map(|node| (node.item, node.ukey))
    .collect::<FxHashMap<_, _>>();

  // grab all the dependencies
  db.par_values_mut().for_each(|node| {
    node.dependencies = get_dependencies(node.item)
      .into_iter()
      .filter_map(|item| item_to_node_ukey.get(&item))
      .copied()
      .collect::<Vec<_>>();
  });

  // Set of current root modules
  // items will be removed if a new reference to it has been found
  let mut roots: UkeySet<NodeUkey<Item>> = UkeySet::default();

  // Set of current cycles without references to it
  // cycles will be removed if a new reference to it has been found
  // that is not part of the cycle
  let mut root_cycles: UkeySet<CycleUkey<NodeUkey<Item>>> = UkeySet::default();

  let mut keys = db.keys().copied().collect::<Vec<_>>();
  keys.sort_by(|a, b| db.expect_get(a).item.cmp(&db.expect_get(b).item));

  // For all non-marked nodes
  for select_node in keys {
    if matches!(db.expect_get(&select_node).marker, Marker::NoMarker) {
      // deep-walk all referenced modules
      // in a non-recursive way

      // start by entering the selected node
      db.expect_get_mut(&select_node).marker = Marker::InProgressMarker;

      // keep a stack to avoid recursive walk
      let mut stack = vec![StackEntry {
        node: select_node,
        open_edges: {
          let mut v: Vec<_> = db.expect_get(&select_node).dependencies.clone();
          v.sort_by(|a, b| db.expect_get(a).item.cmp(&db.expect_get(b).item));
          v
        },
      }];

      // process the top item until stack is empty
      while !stack.is_empty() {
        let top_of_stack_idx = stack.len() - 1;

        // Are there still edges unprocessed in the current node?
        if !stack[top_of_stack_idx].open_edges.is_empty() {
          let mut edges = stack[top_of_stack_idx]
            .open_edges
            .iter()
            .map(|edge| db.expect_get(edge))
            .collect::<Vec<_>>();

          edges.sort_by(|a, b| a.item.cmp(&b.item));

          // Process one dependency
          let dependency = stack[top_of_stack_idx]
            .open_edges
            .pop()
            .expect("Should exist");
          match db.expect_get(&dependency).marker {
            Marker::NoMarker => {
              // dependency has not be visited yet
              // mark it as in-progress and recurse
              stack.push(StackEntry {
                node: dependency,
                open_edges: {
                  let mut v: Vec<_> = db.expect_get(&dependency).dependencies.clone();
                  v.sort_unstable();
                  v
                },
              });
              db.expect_get_mut(&dependency).marker = Marker::InProgressMarker;
            }
            Marker::InProgressMarker => {
              // It's a in-progress cycle
              let cycle = &db.expect_get(&dependency).cycle;
              if cycle.is_none() {
                let cycle = {
                  let item = Cycle::<NodeUkey<Item>>::default();
                  let ukey = item.ukey();
                  cycle_db.add(item);
                  cycle_db.get_mut(&ukey).expect("should have item")
                };
                cycle.nodes.insert(dependency);
                db.expect_get_mut(&dependency).cycle = Some(cycle.ukey);
              }
              let cycle = db.expect_get(&dependency).cycle.expect("Should exist");

              // set cycle property for each node in the cycle
              // if nodes are already part of a cycle
              // we merge the cycles to a shared cycle
              {
                let mut i = stack.len() - 1;
                while db.expect_get(&stack[i].node).item != db.expect_get(&dependency).item {
                  let node = stack[i].node;
                  if let Some(node_cycle) = db.expect_get(&node).cycle {
                    if node_cycle != cycle {
                      for cycle_node in cycle_db.expect_get(&node_cycle).nodes.clone() {
                        db.expect_get_mut(&cycle_node).cycle = Some(cycle);
                        cycle_db.expect_get_mut(&cycle).nodes.insert(cycle_node);
                      }
                    }
                  } else {
                    db.expect_get_mut(&node).cycle = Some(cycle);
                    cycle_db.expect_get_mut(&cycle).nodes.insert(node);
                  }

                  if i == 0 {
                    break;
                  } else {
                    i -= 1;
                  }
                }
              }
              // don't recurse into dependencies
              // these are already on the stack
            }
            Marker::DoneAndRootMarker => {
              db.expect_get_mut(&dependency).marker = Marker::DoneMarker;
              roots.remove(&dependency);
            }
            Marker::DoneMaybeRootCycleMarker => {
              if let Some(cycle) = db.expect_get(&dependency).cycle {
                root_cycles.remove(&cycle);
              };
              db.expect_get_mut(&dependency).marker = Marker::DoneMarker;
            }
            _ => {}
          }
        } else if let Some(top_of_stack) = stack.pop() {
          db.expect_get_mut(&top_of_stack.node).marker = Marker::DoneMarker;
        }
      }
      let cycle = db.expect_get(&select_node).cycle;
      if let Some(cycle) = cycle {
        for node in cycle_db.expect_get_mut(&cycle).nodes.iter() {
          db.expect_get_mut(node).marker = Marker::DoneMaybeRootCycleMarker;
        }
        root_cycles.insert(cycle);
      } else {
        db.expect_get_mut(&select_node).marker = Marker::DoneAndRootMarker;
        roots.insert(select_node);
      }
    }
  }

  // Extract roots from root cycles
  // We take the nodes with most incoming edges
  // inside of the cycle

  for cycle in root_cycles {
    let mut max = 0;

    let mut cycle_roots: UkeySet<NodeUkey<Item>> = Default::default();
    let nodes = &cycle_db.expect_get(&cycle).nodes;
    for node in nodes.iter() {
      for dep in db.expect_get(node).dependencies.clone() {
        if nodes.contains(&dep) {
          db.expect_get_mut(&dep).incoming += 1;
          if db.expect_get(&dep).incoming < max {
            continue;
          }
          if db.expect_get(&dep).incoming > max {
            cycle_roots.clear();
            max = db.expect_get(&dep).incoming;
          }
          cycle_roots.insert(dep);
        }
      }
    }
    for cycle_root in cycle_roots {
      roots.insert(cycle_root);
    }
  }

  if roots.is_empty() {
    panic!("Implementation of findGraphRoots is broken")
  }

  roots
    .into_iter()
    .map(|root| db.remove(&root).expect("should exist"))
    .map(|node| node.item)
    .collect()
}
