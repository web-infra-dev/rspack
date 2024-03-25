// Port of https://github.com/webpack/webpack/blob/main/lib/util/findGraphRoots.js

use std::hash::Hash;

use rspack_database::{Database, DatabaseItem, Ukey};
use rustc_hash::{FxHashMap, FxHashSet};

#[allow(clippy::enum_variant_names)]
enum Marker {
  NoMarker,
  InProgressMarker,
  DoneMarker,
  DoneMaybeRootCycleMarker,
  DoneAndRootMarker,
}

struct Cycle<T: Hash + PartialEq + Eq> {
  pub ukey: Ukey<Cycle<T>>,
  pub nodes: FxHashSet<T>,
}

impl<T: 'static + Hash + PartialEq + Eq> Default for Cycle<T> {
  fn default() -> Self {
    Self {
      ukey: Ukey::<Cycle<T>>::new(),
      nodes: Default::default(),
    }
  }
}

impl<T: Hash + PartialEq + Eq> DatabaseItem for Cycle<T> {
  fn ukey(&self) -> Ukey<Self> {
    self.ukey
  }
}

struct Node<T> {
  pub ukey: Ukey<Node<T>>,
  pub item: T,
  pub dependencies: Vec<Ukey<Node<T>>>,
  pub marker: Marker,
  pub cycle: Option<Ukey<Cycle<Ukey<Node<T>>>>>,
  pub incoming: usize,
}

impl<T> DatabaseItem for Node<T> {
  fn ukey(&self) -> Ukey<Self> {
    self.ukey
  }
}

impl<T: 'static> Node<T> {
  fn new(item: T) -> Self {
    Self {
      ukey: Ukey::<Node<T>>::new(),
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
  Item: Clone + std::fmt::Debug + PartialEq + Eq + Hash + Send + Sync + Ord + 'static,
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
  let mut cycle_db = Database::<Cycle<Ukey<Node<Item>>>>::new();

  items
    .into_iter()
    .map(|item| Node::new(item))
    .for_each(|node| {
      db.add(node);
    });

  let item_to_node_ukey = db
    .values()
    .map(|node| (node.item.clone(), node.ukey))
    .collect::<FxHashMap<_, _>>();

  // grab all the dependencies
  db.par_values_mut().for_each(|node| {
    node.dependencies = get_dependencies(node.item.clone())
      .into_iter()
      .filter_map(|item| item_to_node_ukey.get(&item))
      .cloned()
      .collect::<Vec<_>>();
  });

  // Set of current root modules
  // items will be removed if a new reference to it has been found
  let mut roots: FxHashSet<Ukey<Node<Item>>> = FxHashSet::default();

  // Set of current cycles without references to it
  // cycles will be removed if a new reference to it has been found
  // that is not part of the cycle
  let mut root_cycles: FxHashSet<Ukey<Cycle<Ukey<Node<Item>>>>> = FxHashSet::default();

  let mut keys = db.keys().cloned().collect::<Vec<_>>();
  keys.sort_by(|a, b| a.as_ref(&db).item.cmp(&b.as_ref(&db).item));

  // For all non-marked nodes
  for select_node in keys {
    if matches!(select_node.as_ref(&db).marker, Marker::NoMarker) {
      // deep-walk all referenced modules
      // in a non-recursive way

      // start by entering the selected node
      select_node.as_mut(&mut db).marker = Marker::InProgressMarker;

      // keep a stack to avoid recursive walk
      let mut stack = vec![StackEntry {
        node: select_node,
        open_edges: {
          let mut v: Vec<_> = select_node.as_ref(&db).dependencies.to_vec();
          v.sort_by(|a, b| a.as_ref(&db).item.cmp(&b.as_ref(&db).item));
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
            .map(|edge| edge.as_ref(&db))
            .collect::<Vec<_>>();

          edges.sort_by(|a, b| a.item.cmp(&b.item));

          // Process one dependency
          let dependency = stack[top_of_stack_idx]
            .open_edges
            .pop()
            .expect("Should exist");
          match dependency.as_ref(&db).marker {
            Marker::NoMarker => {
              // dependency has not be visited yet
              // mark it as in-progress and recurse
              stack.push(StackEntry {
                node: dependency,
                open_edges: {
                  let mut v: Vec<_> = dependency.as_ref(&db).dependencies.to_vec();
                  v.sort_unstable();
                  v
                },
              });
              dependency.as_mut(&mut db).marker = Marker::InProgressMarker;
            }
            Marker::InProgressMarker => {
              // It's a in-progress cycle
              let cycle = &dependency.as_ref(&db).cycle;
              if cycle.is_none() {
                let cycle = cycle_db.create_default_item();
                cycle.nodes.insert(dependency);
                dependency.as_mut(&mut db).cycle = Some(cycle.ukey);
              }
              let cycle = dependency.as_ref(&db).cycle.expect("Should exist");

              // set cycle property for each node in the cycle
              // if nodes are already part of a cycle
              // we merge the cycles to a shared cycle
              {
                let mut i = stack.len() - 1;
                while stack[i].node.as_ref(&db).item != dependency.as_ref(&db).item {
                  let node = stack[i].node;
                  if let Some(node_cycle) = node.as_ref(&db).cycle {
                    if node_cycle != cycle {
                      for cycle_node in node_cycle.as_ref(&cycle_db).nodes.clone() {
                        cycle_node.as_mut(&mut db).cycle = Some(cycle);
                        cycle.as_mut(&mut cycle_db).nodes.insert(cycle_node);
                      }
                    }
                  } else {
                    node.as_mut(&mut db).cycle = Some(cycle);
                    cycle.as_mut(&mut cycle_db).nodes.insert(node);
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
              dependency.as_mut(&mut db).marker = Marker::DoneMarker;
              roots.remove(&dependency);
            }
            Marker::DoneMaybeRootCycleMarker => {
              if let Some(cycle) = dependency.as_ref(&db).cycle {
                root_cycles.remove(&cycle);
              };
              dependency.as_mut(&mut db).marker = Marker::DoneMarker;
            }
            _ => {}
          }
        } else if let Some(top_of_stack) = stack.pop() {
          top_of_stack.node.as_mut(&mut db).marker = Marker::DoneMarker;
        }
      }
      let cycle = select_node.as_ref(&db).cycle;
      if let Some(cycle) = cycle {
        for node in cycle.as_mut(&mut cycle_db).nodes.iter() {
          node.as_mut(&mut db).marker = Marker::DoneMaybeRootCycleMarker;
        }
        root_cycles.insert(cycle);
      } else {
        select_node.as_mut(&mut db).marker = Marker::DoneAndRootMarker;
        roots.insert(select_node);
      }
    }
  }

  // Extract roots from root cycles
  // We take the nodes with most incoming edges
  // inside of the cycle

  for cycle in root_cycles {
    let mut max = 0;

    let mut cycle_roots: FxHashSet<Ukey<Node<Item>>> = Default::default();
    let nodes = &cycle.as_ref(&cycle_db).nodes;
    for node in nodes.iter() {
      for dep in node.as_ref(&db).dependencies.clone().into_iter() {
        if nodes.contains(&dep) {
          dep.as_mut(&mut db).incoming += 1;
          if dep.as_ref(&db).incoming < max {
            continue;
          }
          if dep.as_ref(&db).incoming > max {
            cycle_roots.clear();
            max = dep.as_ref(&db).incoming;
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
