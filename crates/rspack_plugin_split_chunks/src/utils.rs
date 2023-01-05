use std::cmp::Ordering;

use rspack_core::ModuleIdentifier;
use rspack_util::comparators::compare_ids;

use crate::{plugin::ChunksInfoItem, CacheGroupByKey, SplitChunkSizes};

pub(crate) fn compare_entries(
  a: &ChunksInfoItem,
  b: &ChunksInfoItem,
  cache_group_by_key: &CacheGroupByKey,
) -> f64 {
  // 1. by priority
  let diff_priority =
    a.cache_group(cache_group_by_key).priority - b.cache_group(cache_group_by_key).priority;
  if diff_priority > 0 {
    return diff_priority as f64;
  }
  // 2. by number of chunks
  let diff_count = a.chunks.len() - b.chunks.len();
  if diff_count > 0 {
    return diff_count as f64;
  }
  // 3. by size reduction
  let a_size_reduce = total_size(&a.sizes) * (a.chunks.len() - 1) as f64;
  let b_size_reduce = total_size(&b.sizes) * (b.chunks.len() - 1) as f64;
  let diff_size_reduce = a_size_reduce - b_size_reduce;
  if diff_size_reduce > 0f64 {
    return diff_size_reduce;
  }
  // 4. by cache group index
  let index_diff = b.cache_group_index - a.cache_group_index;
  if index_diff > 0 {
    return index_diff as f64;
  }
  // 5. by number of modules (to be able to compare by identifier)
  let mut modules_a = a.modules.iter().collect::<Vec<_>>();
  let mut modules_b = b.modules.iter().collect::<Vec<_>>();
  let diff = modules_a.len() - modules_b.len();
  if diff > 0 {
    return diff as f64;
  }
  modules_a.sort();
  modules_b.sort();
  compare_modules(&modules_a, &modules_b) as usize as f64
}

fn total_size(sizes: &SplitChunkSizes) -> f64 {
  sizes.values().cloned().sum()
}

fn compare_modules(a: &[&ModuleIdentifier], b: &[&ModuleIdentifier]) -> Ordering {
  let mut a_i = a.iter();
  let mut b_i = b.iter();
  loop {
    let a_item = a_i.next();
    let b_item = b_i.next();
    if a_item.is_none() {
      return if b_item.is_none() {
        Ordering::Equal
      } else {
        Ordering::Less
      };
    } else if b_item.is_none() {
      return Ordering::Greater;
    }
    let res = compare_ids(
      a_item.expect("Should be Some"),
      b_item.expect("Should be Some"),
    );
    if res != Ordering::Equal {
      return res;
    }
  }
}

pub(crate) fn check_min_size(sizes: &SplitChunkSizes, min_size: &SplitChunkSizes) -> bool {
  for key in sizes.keys() {
    let size = sizes.get(key).expect("key should exist");
    if size == &0f64 {
      continue;
    }
    if min_size.get(key).map_or(false, |min_size| size < min_size) {
      return false;
    }
  }
  true
}
