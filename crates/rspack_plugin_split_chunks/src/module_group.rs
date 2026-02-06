use std::cmp::Ordering;

use derive_more::Debug;
use itertools::Itertools;
use rspack_collections::{IdentifierSet, UkeySet};
use rspack_core::{ChunkUkey, ModuleIdentifier, SourceType};
use rustc_hash::FxHashMap;

use crate::{
  CacheGroup,
  common::{ModuleSizes, SplitChunkSizes},
};

pub(crate) struct IndexedCacheGroup<'a> {
  pub cache_group_index: usize,
  pub cache_group: &'a CacheGroup,
}

impl<'a> IndexedCacheGroup<'a> {
  pub(crate) fn compare_by_priority(&self, other: &Self) -> Ordering {
    self
      .cache_group
      .priority
      .partial_cmp(&other.cache_group.priority)
      .unwrap_or(Ordering::Equal)
  }

  pub(crate) fn compare_by_index(&self, other: &Self) -> Ordering {
    self.cache_group_index.cmp(&other.cache_group_index)
  }
}

/// `ModuleGroup` is a abstraction of middle step for splitting chunks.
///
/// `ModuleGroup` captures/contains a bunch of modules due to the `optimization.splitChunks` configuration.
///
/// `ModuleGroup` would be transform into `Chunk`s in the end.
///
///  A `ModuleGroup` would be transform into multiple `Chunk`s if the `name` dynamic computed
///
/// The original name of `ModuleGroup` is `ChunkInfoItem` borrowed from Webpack
#[derive(Debug)]
pub(crate) struct ModuleGroup {
  pub modules: IdentifierSet,
  /// the real index used for mapping the ModuleGroup to corresponding CacheGroup
  pub cache_group_index: usize,
  pub cache_group_reuse_existing_chunk: bool,
  /// If the `ModuleGroup` is going to create a chunk, which will be named using `chunk_name`
  /// A module
  pub chunk_name: Option<String>,

  pub source_types_modules: FxHashMap<SourceType, IdentifierSet>,
  /// `Chunk`s which `Module`s in this ModuleGroup belong to
  #[debug(skip)]
  pub chunks: UkeySet<ChunkUkey>,
  added: Vec<ModuleIdentifier>,
  removed: Vec<ModuleIdentifier>,
  sizes: SplitChunkSizes,
  total_size: f64,
}

impl ModuleGroup {
  pub(crate) fn new(
    chunk_name: Option<String>,
    cache_group_index: usize,
    cache_group: &CacheGroup,
  ) -> Self {
    Self {
      modules: Default::default(),
      cache_group_index,
      cache_group_reuse_existing_chunk: cache_group.reuse_existing_chunk,
      sizes: Default::default(),
      source_types_modules: Default::default(),
      chunks: Default::default(),
      chunk_name,
      added: Default::default(),
      removed: Default::default(),
      total_size: 0.0,
    }
  }

  pub(crate) fn get_source_types_modules(
    &self,
    ty: &[SourceType],
    module_sizes: &ModuleSizes,
  ) -> IdentifierSet {
    // if there is only one source type, we can just use the `source_types_modules` directly
    // instead of iterating over all modules
    if ty.len() == 1 {
      self
        .source_types_modules
        .get(ty.first().expect("should have at least one source type"))
        .cloned()
        .unwrap_or_default()
    } else {
      self
        .modules
        .iter()
        .filter_map(|module| {
          let sizes = module_sizes.get(module).expect("should have module size");
          if ty.iter().any(|ty| sizes.contains_key(ty)) {
            Some(*module)
          } else {
            None
          }
        })
        .collect()
    }
  }

  pub(crate) fn add_module(&mut self, module: ModuleIdentifier) {
    if self.modules.insert(module) {
      self.added.push(module);
    }
  }

  pub(crate) fn remove_module(&mut self, module: ModuleIdentifier) {
    if self.modules.remove(&module) {
      self.removed.push(module);
    }
  }

  pub(crate) fn get_cache_group<'a>(&self, cache_groups: &'a [CacheGroup]) -> &'a CacheGroup {
    &cache_groups[self.cache_group_index]
  }

  pub(crate) fn get_total_size(&self) -> f64 {
    if !self.added.is_empty() || !self.removed.is_empty() {
      unreachable!("should update sizes before get total size");
    }
    self.total_size
  }

  pub(crate) fn get_sizes(&mut self, module_sizes: &ModuleSizes) -> SplitChunkSizes {
    if !self.added.is_empty() {
      let added = std::mem::take(&mut self.added);
      for module in added {
        let module_sizes = module_sizes.get(&module).expect("should have module size");
        for (ty, s) in module_sizes.iter() {
          let size = self.sizes.entry(*ty).or_default();
          *size += s;
          self.total_size += s;
          self
            .source_types_modules
            .entry(*ty)
            .or_default()
            .insert(module);
        }
      }
    }
    if !self.removed.is_empty() {
      let removed = std::mem::take(&mut self.removed);
      for module in removed {
        let module_sizes = module_sizes.get(&module).expect("should have module size");
        for (ty, s) in module_sizes.iter() {
          let size = self.sizes.entry(*ty).or_default();
          *size -= s;
          *size = size.max(0.0);
          self.total_size -= s;
          self
            .source_types_modules
            .entry(*ty)
            .or_default()
            .remove(&module);
        }
      }
    }

    self.sizes.clone()
  }
}

pub(crate) fn compare_entries(
  (a_key, a): (&String, &ModuleGroup),
  (b_key, b): (&String, &ModuleGroup),
) -> f64 {
  // 1. by priority
  // no need to compare priority anymore because we already pick all cache groups with same priority
  // let diff_priority = a.cache_group_priority - b.cache_group_priority;
  // if diff_priority != 0f64 {
  //   return diff_priority;
  // }
  // 2. by number of chunks
  let a_chunks_len = a.chunks.len();
  let b_chunks_len = b.chunks.len();
  let diff_count = a_chunks_len as f64 - b_chunks_len as f64;
  if diff_count != 0f64 {
    return diff_count;
  }

  // 3. by size reduction
  let a_size_reduce = a.get_total_size() * (a_chunks_len - 1) as f64;
  let b_size_reduce = b.get_total_size() * (b_chunks_len - 1) as f64;
  let diff_size_reduce = a_size_reduce - b_size_reduce;
  if diff_size_reduce != 0f64 {
    return diff_size_reduce;
  }

  // 4. by cache group index
  let index_diff = b.cache_group_index as f64 - a.cache_group_index as f64;
  if index_diff != 0f64 {
    return index_diff;
  }

  // 5. by number of modules (to be able to compare by identifier)
  let modules_a_len = a.modules.len();
  let modules_b_len = b.modules.len();
  let diff = modules_a_len as f64 - modules_b_len as f64;
  if diff != 0f64 {
    return diff;
  }

  let mut modules_a = a
    .modules
    .iter()
    .sorted_unstable_by(|a, b| a.precomputed_hash().cmp(&b.precomputed_hash()));
  let mut modules_b = b
    .modules
    .iter()
    .sorted_unstable_by(|a, b| a.precomputed_hash().cmp(&b.precomputed_hash()));

  loop {
    match (modules_a.next(), modules_b.next()) {
      (None, None) => break,
      (Some(a), Some(b)) => {
        let res = a.cmp(b);
        if !res.is_eq() {
          return res as i32 as f64;
        }
      }
      _ => unreachable!(),
    }
  }

  a_key.cmp(b_key) as i32 as f64
}
