use std::{cmp::Ordering, sync::Arc};

use rspack_core::{Chunk, ChunkGroupByUkey, ModuleIdentifier, SourceType};
use rspack_util::comparators::compare_ids;
use rustc_hash::FxHashMap as HashMap;

use crate::{
  CacheGroupByKey, ChunkFilterFn, ChunkType, ChunksInfoItem, SizeType, SplitChunkSizes,
  SplitChunksSizes,
};

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
  if a.chunks.len() >= b.chunks.len() {
    let diff_count = a.chunks.len() - b.chunks.len();
    if diff_count > 0 {
      return diff_count as f64;
    }
  }
  // 3. by size reduction
  let a_size_reduce = total_size(&a.sizes) * (a.chunks.len() - 1) as f64;
  let b_size_reduce = total_size(&b.sizes) * (b.chunks.len() - 1) as f64;
  let diff_size_reduce = a_size_reduce - b_size_reduce;
  if diff_size_reduce > 0f64 {
    return diff_size_reduce;
  }
  // 4. by cache group index
  if b.cache_group_index >= a.cache_group_index {
    let index_diff = b.cache_group_index - a.cache_group_index;
    if index_diff > 0 {
      return index_diff as f64;
    }
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

pub(crate) fn get_violating_min_sizes(
  sizes: &SplitChunkSizes,
  min_size: &SplitChunkSizes,
) -> Option<Vec<SourceType>> {
  let mut list: Option<Vec<SourceType>> = None;
  for key in min_size.keys() {
    let size = sizes.get(key).unwrap_or(&0f64);
    if size == &0f64 {
      continue;
    };
    let min_size = min_size.get(key).unwrap_or(&0f64);
    if size < min_size {
      list.get_or_insert_default().push(*key);
    }
  }
  list
}

pub(crate) fn check_min_size_reduction(
  sizes: &SplitChunkSizes,
  min_size_reduction: &SplitChunkSizes,
  chunk_count: usize,
) -> bool {
  for key in min_size_reduction.keys() {
    let size = sizes.get(key).unwrap_or(&0f64);
    if size == &0f64 {
      continue;
    };
    let min_size_reduction = min_size_reduction.get(key).unwrap_or(&0f64);
    if (size * chunk_count as f64) < *min_size_reduction {
      return false;
    }
  }
  true
}

pub(crate) fn get_requests(chunk: &Chunk, chunk_group_by_ukey: &ChunkGroupByUkey) -> u32 {
  let mut requests = 0;
  for group in &chunk.groups {
    let group = chunk_group_by_ukey
      .get(group)
      .expect("ChunkGroup not found");
    requests = u32::max(requests, group.chunks.len() as u32)
  }
  requests
}

pub(crate) fn combine_sizes(
  a: &SplitChunkSizes,
  b: &SplitChunkSizes,
  combine: impl Fn(f64, f64) -> f64,
) -> SplitChunkSizes {
  let a_keys = a.keys();
  let b_keys = b.keys();
  let mut res: SplitChunkSizes = Default::default();
  for key in a_keys {
    if b.contains_key(key) {
      res.insert(*key, combine(a[key], b[key]));
    } else {
      res.insert(*key, a[key]);
    }
  }

  for key in b_keys {
    if !a.contains_key(key) {
      res.insert(*key, b[key]);
    }
  }

  res
}

pub(crate) fn normalize_sizes<T: Clone>(
  value: Option<T>,
  default_size_types: &[SizeType],
) -> HashMap<SizeType, T> {
  value
    .map(|value| {
      default_size_types
        .iter()
        .cloned()
        .map(|size_type| (size_type, value.clone()))
        .collect::<HashMap<_, _>>()
    })
    .unwrap_or_default()
}

pub(crate) fn merge_sizes2(
  mut a: HashMap<SizeType, f64>,
  b: HashMap<SizeType, f64>,
) -> HashMap<SizeType, f64> {
  a.extend(b);
  a
}

pub(crate) fn merge_sizes(sizes: Vec<SplitChunksSizes>) -> SplitChunkSizes {
  let mut res: SplitChunkSizes = Default::default();
  for size in sizes {
    res.extend(size)
  }
  res
}

pub(crate) fn normalize_chunks_filter(chunk_type: ChunkType) -> ChunkFilterFn {
  Arc::new(move |chunk, chunk_group_by_ukey| chunk_type.is_selected(chunk, chunk_group_by_ukey))
}
