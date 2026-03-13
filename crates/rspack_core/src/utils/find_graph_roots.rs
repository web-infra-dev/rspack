// Port of https://github.com/webpack/webpack/blob/main/lib/util/findGraphRoots.js

use std::{fmt::Debug, hash::Hash, sync::atomic::AtomicU32};

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
struct CycleUkey<T: Hash + Eq + Copy>(u32, std::marker::PhantomData<Cycle<T>>);

impl<T: Hash + Eq + Copy> CycleUkey<T> {
  pub fn new() -> Self {
    Self(
      NEXT_CYCLE_UKEY.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
      std::marker::PhantomData,
    )
  }
}

struct Cycle<T: Hash + Eq + Copy> {
  pub ukey: CycleUkey<T>,
  pub nodes: FxHashSet<T>,
  pub is_root: bool,
}

impl<T: Hash + Eq + Copy> Default for Cycle<T> {
  fn default() -> Self {
    Self {
      ukey: CycleUkey::<T>::new(),
      nodes: Default::default(),
      is_root: false,
    }
  }
}

impl<T: Hash + Eq + Copy> Cycle<T> {
  fn ukey(&self) -> CycleUkey<T> {
    self.ukey
  }

  fn with_capacity(capacity: usize) -> Self {
    Self {
      ukey: CycleUkey::<T>::new(),
      nodes: FxHashSet::with_capacity_and_hasher(capacity, Default::default()),
      is_root: false,
    }
  }
}

static NEXT_NODE_UKEY: AtomicU32 = AtomicU32::new(0);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct NodeUkey<T: Hash + Eq + Copy>(u32, std::marker::PhantomData<Node<T>>);

impl<T: Hash + Eq + Copy> NodeUkey<T> {
  pub fn new() -> Self {
    Self(
      NEXT_NODE_UKEY.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
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

impl<T: Hash + Eq + Copy> Node<T> {
  fn ukey(&self) -> NodeUkey<T> {
    self.ukey
  }

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

fn expect_get<'a, K, V>(db: &'a FxHashMap<K, V>, key: &K, db_name: &str) -> &'a V
where
  K: Eq + Hash + Debug,
{
  db.get(key)
    .unwrap_or_else(|| panic!("{db_name}({key:?}) not found"))
}

fn expect_get_mut<'a, K, V>(db: &'a mut FxHashMap<K, V>, key: &K, db_name: &str) -> &'a mut V
where
  K: Eq + Hash + Debug,
{
  db.get_mut(key)
    .unwrap_or_else(|| panic!("{db_name}({key:?}) not found"))
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

  let mut db = FxHashMap::<NodeUkey<Item>, Node<Item>>::default();
  let mut cycle_db = FxHashMap::<CycleUkey<NodeUkey<Item>>, Cycle<NodeUkey<Item>>>::default();

  items
    .into_iter()
    .map(|item| Node::new(item))
    .for_each(|node| {
      db.insert(node.ukey(), node);
    });

  let item_to_node_ukey = db
    .values()
    .map(|node| (node.item, node.ukey()))
    .collect::<FxHashMap<_, _>>();

  // grab all the dependencies
  db.values_mut().par_bridge().for_each(|node| {
    node.dependencies = get_dependencies(node.item)
      .into_iter()
      .filter_map(|item| item_to_node_ukey.get(&item))
      .copied()
      .collect::<Vec<_>>();
  });

  // Set of current root modules
  // items will be removed if a new reference to it has been found
  let mut roots = FxHashSet::with_capacity_and_hasher(db.len(), Default::default());

  let mut keys = db.keys().copied().collect::<Vec<_>>();
  keys.sort_by(|a, b| {
    expect_get(&db, a, "Node")
      .item
      .cmp(&expect_get(&db, b, "Node").item)
  });

  // For all non-marked nodes
  for select_node in keys {
    if matches!(
      expect_get(&db, &select_node, "Node").marker,
      Marker::NoMarker
    ) {
      // deep-walk all referenced modules
      // in a non-recursive way

      // start by entering the selected node
      expect_get_mut(&mut db, &select_node, "Node").marker = Marker::InProgressMarker;

      // keep a stack to avoid recursive walk
      let mut stack = vec![StackEntry {
        node: select_node,
        open_edges: {
          let mut v: Vec<_> = expect_get(&db, &select_node, "Node").dependencies.clone();
          v.sort_by(|a, b| {
            expect_get(&db, a, "Node")
              .item
              .cmp(&expect_get(&db, b, "Node").item)
          });
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
            .map(|edge| expect_get(&db, edge, "Node"))
            .collect::<Vec<_>>();

          edges.sort_by(|a, b| a.item.cmp(&b.item));

          // Process one dependency
          let dependency = stack[top_of_stack_idx]
            .open_edges
            .pop()
            .expect("Should exist");
          match expect_get(&db, &dependency, "Node").marker {
            Marker::NoMarker => {
              // dependency has not be visited yet
              // mark it as in-progress and recurse
              stack.push(StackEntry {
                node: dependency,
                open_edges: {
                  let mut v: Vec<_> = expect_get(&db, &dependency, "Node").dependencies.clone();
                  v.sort_unstable();
                  v
                },
              });
              expect_get_mut(&mut db, &dependency, "Node").marker = Marker::InProgressMarker;
            }
            Marker::InProgressMarker => {
              // It's a in-progress cycle
              let cycle = &expect_get(&db, &dependency, "Node").cycle;
              if cycle.is_none() {
                let cycle_ukey = {
                  let item = Cycle::<NodeUkey<Item>>::with_capacity(stack.len());
                  let ukey = item.ukey();
                  cycle_db.insert(ukey, item);
                  ukey
                };
                expect_get_mut(&mut cycle_db, &cycle_ukey, "Cycle")
                  .nodes
                  .insert(dependency);
                expect_get_mut(&mut db, &dependency, "Node").cycle = Some(cycle_ukey);
              }
              let cycle = expect_get(&db, &dependency, "Node")
                .cycle
                .expect("Should exist");

              // set cycle property for each node in the cycle
              // if nodes are already part of a cycle
              // we merge the cycles to a shared cycle
              {
                let mut i = stack.len() - 1;
                while expect_get(&db, &stack[i].node, "Node").item
                  != expect_get(&db, &dependency, "Node").item
                {
                  let node = stack[i].node;
                  if let Some(node_cycle) = expect_get(&db, &node, "Node").cycle {
                    if node_cycle != cycle {
                      for cycle_node in expect_get(&cycle_db, &node_cycle, "Cycle").nodes.clone() {
                        expect_get_mut(&mut db, &cycle_node, "Node").cycle = Some(cycle);
                        expect_get_mut(&mut cycle_db, &cycle, "Cycle")
                          .nodes
                          .insert(cycle_node);
                      }
                    }
                  } else {
                    expect_get_mut(&mut db, &node, "Node").cycle = Some(cycle);
                    expect_get_mut(&mut cycle_db, &cycle, "Cycle")
                      .nodes
                      .insert(node);
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
              expect_get_mut(&mut db, &dependency, "Node").marker = Marker::DoneMarker;
              roots.remove(&dependency);
            }
            Marker::DoneMaybeRootCycleMarker => {
              if let Some(cycle) = expect_get(&db, &dependency, "Node").cycle {
                expect_get_mut(&mut cycle_db, &cycle, "Cycle").is_root = false;
              };
              expect_get_mut(&mut db, &dependency, "Node").marker = Marker::DoneMarker;
            }
            _ => {}
          }
        } else if let Some(top_of_stack) = stack.pop() {
          expect_get_mut(&mut db, &top_of_stack.node, "Node").marker = Marker::DoneMarker;
        }
      }
      let cycle = expect_get(&db, &select_node, "Node").cycle;
      if let Some(cycle) = cycle {
        for node in expect_get_mut(&mut cycle_db, &cycle, "Cycle").nodes.iter() {
          expect_get_mut(&mut db, node, "Node").marker = Marker::DoneMaybeRootCycleMarker;
        }
        expect_get_mut(&mut cycle_db, &cycle, "Cycle").is_root = true;
      } else {
        expect_get_mut(&mut db, &select_node, "Node").marker = Marker::DoneAndRootMarker;
        roots.insert(select_node);
      }
    }
  }

  // Extract roots from root cycles
  // We take the nodes with most incoming edges
  // inside of the cycle

  let root_cycles = cycle_db
    .values()
    .filter(|cycle| cycle.is_root)
    .map(|cycle| cycle.ukey)
    .collect::<Vec<_>>();

  for cycle in root_cycles {
    let mut max = 0;

    let nodes = &expect_get(&cycle_db, &cycle, "Cycle").nodes;
    let mut cycle_roots = FxHashSet::with_capacity_and_hasher(nodes.len(), Default::default());
    for node in nodes.iter() {
      for dep in expect_get(&db, node, "Node").dependencies.clone() {
        if nodes.contains(&dep) {
          expect_get_mut(&mut db, &dep, "Node").incoming += 1;
          if expect_get(&db, &dep, "Node").incoming < max {
            continue;
          }
          if expect_get(&db, &dep, "Node").incoming > max {
            cycle_roots.clear();
            max = expect_get(&db, &dep, "Node").incoming;
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
