// Port of https://github.com/webpack/webpack/blob/main/lib/util/findGraphRoots.js

use std::{fmt::Debug, hash::Hash};

use rustc_hash::{FxHashMap, FxHashSet};

#[allow(clippy::enum_variant_names)]
#[derive(Clone, Copy)]
enum Marker {
  NoMarker,
  InProgressMarker,
  DoneMarker,
  DoneMaybeRootCycleMarker,
  DoneAndRootMarker,
}

type NodeId = usize;
type CycleId = usize;

struct Cycle {
  pub nodes: FxHashSet<NodeId>,
  pub is_root: bool,
}

impl Cycle {
  fn with_capacity(capacity: usize) -> Self {
    Self {
      nodes: FxHashSet::with_capacity_and_hasher(capacity, Default::default()),
      is_root: false,
    }
  }
}

struct Node<T: Hash + Eq + Copy> {
  pub item: T,
  pub dependencies: Vec<NodeId>,
  pub marker: Marker,
  pub cycle: Option<CycleId>,
  pub incoming: usize,
}

impl<T: Hash + Eq + Copy> Node<T> {
  fn new(item: T) -> Self {
    Self {
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
  next_edge: usize,
}

pub fn find_graph_roots<
  Item: Clone + Copy + Debug + PartialEq + Eq + Hash + Send + Sync + Ord + 'static,
>(
  items: Vec<Item>,
  get_dependencies: impl Sync + Fn(Item, &mut dyn FnMut(Item)),
) -> Vec<Item> {
  use rayon::prelude::*;
  // early exit when there is only a single item
  if items.len() <= 1 {
    return items;
  }

  let mut nodes = items
    .into_iter()
    .map(|item| Node::new(item))
    .collect::<Vec<_>>();

  let mut cycle_db = Vec::<Cycle>::new();
  let mut item_to_node_id = FxHashMap::with_capacity_and_hasher(nodes.len(), Default::default());
  for (node_id, node) in nodes.iter().enumerate() {
    item_to_node_id.insert(node.item, node_id);
  }
  let items_by_node = nodes.iter().map(|node| node.item).collect::<Vec<_>>();

  // grab all the dependencies
  nodes.par_iter_mut().for_each(|node| {
    get_dependencies(node.item, &mut |item| {
      if let Some(node_id) = item_to_node_id.get(&item) {
        node.dependencies.push(*node_id);
      }
    });
    node
      .dependencies
      .sort_unstable_by_key(|node_id| items_by_node[*node_id]);
    node.dependencies.dedup();
  });

  // Set of current root modules
  // items will be removed if a new reference to it has been found
  let mut roots = FxHashSet::with_capacity_and_hasher(nodes.len(), Default::default());

  let mut keys = (0..nodes.len()).collect::<Vec<_>>();
  keys.sort_unstable_by_key(|node_id| nodes[*node_id].item);

  // For all non-marked nodes
  for select_node in keys {
    if matches!(nodes[select_node].marker, Marker::NoMarker) {
      // deep-walk all referenced modules
      // in a non-recursive way

      // start by entering the selected node
      nodes[select_node].marker = Marker::InProgressMarker;

      // keep a stack to avoid recursive walk
      let mut stack = vec![StackEntry {
        node: select_node,
        next_edge: nodes[select_node].dependencies.len(),
      }];

      // process the top item until stack is empty
      while !stack.is_empty() {
        let top_of_stack_idx = stack.len() - 1;

        // Are there still edges unprocessed in the current node?
        if stack[top_of_stack_idx].next_edge > 0 {
          // Process one dependency
          let dependency = {
            let top_of_stack = &mut stack[top_of_stack_idx];
            top_of_stack.next_edge -= 1;
            nodes[top_of_stack.node].dependencies[top_of_stack.next_edge]
          };
          match nodes[dependency].marker {
            Marker::NoMarker => {
              // dependency has not be visited yet
              // mark it as in-progress and recurse
              stack.push(StackEntry {
                node: dependency,
                next_edge: nodes[dependency].dependencies.len(),
              });
              nodes[dependency].marker = Marker::InProgressMarker;
            }
            Marker::InProgressMarker => {
              // It's a in-progress cycle
              if nodes[dependency].cycle.is_none() {
                let cycle_id = cycle_db.len();
                cycle_db.push(Cycle::with_capacity(stack.len()));
                cycle_db[cycle_id].nodes.insert(dependency);
                nodes[dependency].cycle = Some(cycle_id);
              }
              let cycle = nodes[dependency].cycle.expect("Should exist");

              // set cycle property for each node in the cycle
              // if nodes are already part of a cycle
              // we merge the cycles to a shared cycle
              {
                let mut i = stack.len() - 1;
                while stack[i].node != dependency {
                  let node = stack[i].node;
                  if let Some(node_cycle) = nodes[node].cycle {
                    if node_cycle != cycle {
                      let old_cycle_nodes = std::mem::take(&mut cycle_db[node_cycle].nodes);
                      for cycle_node in old_cycle_nodes {
                        nodes[cycle_node].cycle = Some(cycle);
                        cycle_db[cycle].nodes.insert(cycle_node);
                      }
                    }
                  } else {
                    nodes[node].cycle = Some(cycle);
                    cycle_db[cycle].nodes.insert(node);
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
              nodes[dependency].marker = Marker::DoneMarker;
              roots.remove(&dependency);
            }
            Marker::DoneMaybeRootCycleMarker => {
              if let Some(cycle) = nodes[dependency].cycle {
                cycle_db[cycle].is_root = false;
              };
              nodes[dependency].marker = Marker::DoneMarker;
            }
            _ => {}
          }
        } else if let Some(top_of_stack) = stack.pop() {
          nodes[top_of_stack.node].marker = Marker::DoneMarker;
        }
      }
      let cycle = nodes[select_node].cycle;
      if let Some(cycle) = cycle {
        for &node in &cycle_db[cycle].nodes {
          nodes[node].marker = Marker::DoneMaybeRootCycleMarker;
        }
        cycle_db[cycle].is_root = true;
      } else {
        nodes[select_node].marker = Marker::DoneAndRootMarker;
        roots.insert(select_node);
      }
    }
  }

  // Extract roots from root cycles
  // We take the nodes with most incoming edges
  // inside of the cycle

  for cycle in &cycle_db {
    if !cycle.is_root {
      continue;
    }
    let mut max = 0;

    let mut cycle_roots = Vec::new();
    for &node in &cycle.nodes {
      let dependency_len = nodes[node].dependencies.len();
      for dependency_idx in 0..dependency_len {
        let dep = nodes[node].dependencies[dependency_idx];
        if cycle.nodes.contains(&dep) {
          nodes[dep].incoming += 1;
          let incoming = nodes[dep].incoming;
          if incoming < max {
            continue;
          }
          if incoming > max {
            cycle_roots.clear();
            max = incoming;
          }
          cycle_roots.push(dep);
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

  roots.into_iter().map(|root| nodes[root].item).collect()
}

#[cfg(test)]
mod tests {
  use super::find_graph_roots;

  fn collect_roots(
    items: Vec<u32>,
    dependencies: impl Sync + Fn(u32) -> &'static [u32],
  ) -> Vec<u32> {
    let mut roots = find_graph_roots(items, |item, add_dependency| {
      for dependency in dependencies(item) {
        add_dependency(*dependency);
      }
    });
    roots.sort_unstable();
    roots
  }

  #[test]
  fn finds_roots_for_linear_graph() {
    let roots = collect_roots(vec![1, 2, 3], |item| match item {
      1 => &[2],
      2 => &[3],
      _ => &[],
    });

    assert_eq!(roots, vec![1]);
  }

  #[test]
  fn finds_roots_for_disconnected_graph() {
    let roots = collect_roots(vec![1, 2, 3], |item| match item {
      1 => &[2],
      _ => &[],
    });

    assert_eq!(roots, vec![1, 3]);
  }

  #[test]
  fn extracts_roots_from_root_cycle() {
    let roots = collect_roots(vec![1, 2, 3], |item| match item {
      1 => &[2],
      2 => &[1],
      3 => &[1],
      _ => &[],
    });

    assert_eq!(roots, vec![3]);
  }
}
