use std::hash::Hash;

use rustc_hash::FxHashMap;

/// Topologically sort `items` with source-order tie-breaking.
///
/// Nodes that are part of a cycle are omitted from the result, matching
/// webpack's topologicalSort utility. Callers that use this for dependency
/// source ordering should leave omitted nodes without source order so they
/// keep their natural source position.
pub fn topological_sort<Item>(
  items: Vec<Item>,
  get_successors: impl Fn(Item) -> Vec<Item>,
) -> Vec<Item>
where
  Item: Copy + Eq + Hash,
{
  if items.len() <= 1 {
    return items;
  }

  let item_to_index = items
    .iter()
    .enumerate()
    .map(|(index, item)| (*item, index))
    .collect::<FxHashMap<_, _>>();
  debug_assert_eq!(item_to_index.len(), items.len());

  let mut in_degree = vec![0usize; items.len()];
  let mut successors_by_index = Vec::with_capacity(items.len());
  for item in &items {
    let successors = get_successors(*item)
      .into_iter()
      .filter_map(|successor| item_to_index.get(&successor).copied())
      .collect::<Vec<_>>();
    for successor in &successors {
      in_degree[*successor] += 1;
    }
    successors_by_index.push(successors);
  }

  let mut ready = in_degree
    .iter()
    .enumerate()
    .filter_map(|(index, degree)| (*degree == 0).then_some(index))
    .collect::<Vec<_>>();
  let mut ordered_indices = Vec::with_capacity(items.len());

  while !ready.is_empty() {
    let mut min_position = 0;
    for position in 1..ready.len() {
      if ready[position] < ready[min_position] {
        min_position = position;
      }
    }

    let index = ready.swap_remove(min_position);
    ordered_indices.push(index);

    for successor in &successors_by_index[index] {
      in_degree[*successor] -= 1;
      if in_degree[*successor] == 0 {
        ready.push(*successor);
      }
    }
  }

  ordered_indices
    .into_iter()
    .map(|index| items[index])
    .collect()
}

#[cfg(test)]
mod tests {
  use rustc_hash::FxHashMap;

  use super::topological_sort;

  #[test]
  fn sorts_with_source_order_tie_break() {
    let graph = FxHashMap::from_iter([(1, vec![3]), (2, vec![3])]);

    let ordered = topological_sort(vec![2, 1, 3], |item| {
      graph.get(&item).cloned().unwrap_or_default()
    });

    assert_eq!(ordered, vec![2, 1, 3]);
  }

  #[test]
  fn preserves_cycle_slots() {
    let graph = FxHashMap::from_iter([(1, vec![2]), (2, vec![1]), (3, vec![4])]);

    let ordered = topological_sort(vec![1, 2, 3, 4], |item| {
      graph.get(&item).cloned().unwrap_or_default()
    });

    assert_eq!(ordered, vec![3, 4]);
  }

  #[test]
  fn keeps_acyclic_order_when_cycles_exist() {
    let graph = FxHashMap::from_iter([(1, vec![2]), (2, vec![1]), (4, vec![3])]);

    let ordered = topological_sort(vec![3, 1, 2, 4], |item| {
      graph.get(&item).cloned().unwrap_or_default()
    });

    assert_eq!(ordered, vec![4, 3]);
  }

  #[test]
  fn omits_fully_cyclic_items() {
    let graph = FxHashMap::from_iter([(1, vec![2]), (2, vec![1])]);

    let ordered = topological_sort(vec![1, 2], |item| {
      graph.get(&item).cloned().unwrap_or_default()
    });

    assert_eq!(ordered, Vec::<i32>::new());
  }
}
