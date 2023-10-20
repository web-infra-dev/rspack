use async_scoped::TokioScope;
use dashmap::DashMap;
use num_bigint::BigUint as ChunksKey;
use rayon::prelude::*;
use rspack_core::{Chunk, ChunkByUkey, ChunkGraph, ChunkUkey, Compilation, Module, ModuleGraph};
use rustc_hash::{FxHashMap, FxHashSet};

use super::ModuleGroupMap;
use crate::SplitChunksPlugin;
use crate::{
  cache_group::CacheGroup,
  module_group::{compare_entries, ModuleGroup},
};

impl SplitChunksPlugin {
  #[tracing::instrument(skip_all)]
  pub(crate) fn find_best_module_group(
    &self,
    module_group_map: &mut ModuleGroupMap,
  ) -> (String, ModuleGroup) {
    // perf(hyf): I wonder if we could use BinaryHeap to avoid sorting for find_best_module_group call
    debug_assert!(!module_group_map.is_empty());
    let mut iter: std::collections::hash_map::Iter<String, ModuleGroup> = module_group_map.iter();
    let (key, mut best_module_group) = iter.next().expect("at least have one item");

    let mut best_entry_key = key.clone();
    for (key, each_module_group) in iter {
      if compare_entries(best_module_group, each_module_group) < 0f64 {
        best_entry_key = key.clone();
        best_module_group = each_module_group;
      }
    }

    let best_module_group = module_group_map
      .remove(&best_entry_key)
      .expect("This should never happen, please file an issue");
    (best_entry_key, best_module_group)
  }

  fn create_chunk_index_map(&self, chunk_db: &ChunkByUkey) -> FxHashMap<ChunkUkey, ChunksKey> {
    let mut chunk_index_map: FxHashMap<ChunkUkey, ChunksKey> = Default::default();

    let mut idx: ChunksKey = 1usize.into();

    let mut chunks: Vec<_> = chunk_db.keys().collect();
    chunks.sort_unstable();

    for key in chunks {
      chunk_index_map.insert(*key, idx.clone());
      idx <<= 1;
    }

    chunk_index_map
  }

  #[tracing::instrument(skip_all)]
  pub(crate) async fn prepare_module_group_map(
    &self,
    compilation: &mut Compilation,
  ) -> ModuleGroupMap {
    let chunk_db = &compilation.chunk_by_ukey;
    let chunk_group_db = &compilation.chunk_group_by_ukey;

    /// If a module meets requirements of a `ModuleGroup`. We consider the `Module` and the `CacheGroup`
    /// to be a `MatchedItem`, which are consumed later to calculate `ModuleGroup`.
    struct MatchedItem<'a> {
      module: &'a dyn Module,
      cache_group_index: usize,
      cache_group: &'a CacheGroup,
      selected_chunks: Box<[&'a Chunk]>,
      selected_chunks_key: ChunksKey,
    }

    let module_group_map: DashMap<String, ModuleGroup> = DashMap::default();

    let chunk_idx_map = self.create_chunk_index_map(chunk_db);

    // chunk_sets_in_graph: key: module, value: multiple chunks contains the module
    // single_chunk_sets: chunkset of module that belongs to only one chunk
    // chunk_sets_by_count: use chunkset len as key
    let (chunk_sets_in_graph, chunk_sets_by_count) = Self::prepare_combination_maps(
      &compilation.module_graph,
      &compilation.chunk_graph,
      &chunk_idx_map,
    );

    let combinations_cache = DashMap::<ChunksKey, Vec<FxHashSet<ChunkUkey>>>::default();

    let get_combination = |chunks_key: ChunksKey| {
      if let Some(combs) = combinations_cache.get(&chunks_key) {
        return combs.clone();
      }
      let chunks_set = chunk_sets_in_graph
        .get(&chunks_key)
        .expect("This should never happen, please file an issue");
      let mut result = vec![chunks_set.clone()];

      for (count, array_of_set) in &chunk_sets_by_count {
        if *count < chunks_set.len() {
          for set in array_of_set {
            if set.is_subset(chunks_set) {
              result.push(set.clone());
            }
          }
        }
      }

      combinations_cache.insert(chunks_key.clone(), result);
      combinations_cache
        .get(&chunks_key)
        .expect("This should never happen, please file an issue")
        .clone()
    };

    let chunk_idx_map = &chunk_idx_map;

    async_scoped::Scope::scope_and_block(|scope: &mut TokioScope<'_, _>| {
      for module in compilation.module_graph.modules().values() {
        let module = &**module;

        let belong_to_chunks = compilation
          .chunk_graph
          .get_module_chunks(module.identifier());

        let chunks_key = Self::get_key(belong_to_chunks.iter(), chunk_idx_map);
        let module_group_map = &module_group_map;

        for (cache_group_index, cache_group) in self.cache_groups.iter().filter(|cache_group| {
          // Filter by `splitChunks.cacheGroups.{cacheGroup}.test`
          let is_match_the_test: bool = (cache_group.test)(module);
          let is_match_the_type: bool = (cache_group.r#type)(module);
          let is_match = is_match_the_test && is_match_the_type;

          if !is_match {
            tracing::trace!(
              "Module({:?}) is ignored by CacheGroup({:?}). Reason: !(is_match_the_test({:?}) && is_match_the_type({:?}))",
              module.identifier(),
              cache_group.key,
              is_match_the_test,
              is_match_the_type
            );
          }
          return is_match
        }).enumerate() {
          let chunks_key = chunks_key.clone();
          scope.spawn(async move {
            let combs = get_combination(chunks_key.clone());

            for chunk_combination in combs {
              // Filter by `splitChunks.cacheGroups.{cacheGroup}.minChunks`
              if chunk_combination.len() < cache_group.min_chunks as usize {
                tracing::trace!(
                  "Module({:?}) is ignored by CacheGroup({:?}). Reason: chunk_combination.len({:?}) < cache_group.min_chunks({:?})",
                  module.identifier(),
                  cache_group.key,
                  chunk_combination.len(),
                  cache_group.min_chunks,
                );
                continue;
              }

              let selected_chunks = chunk_combination
                .iter()
                .map(|c| chunk_db.get(c).expect("This should never happen, please file an issue"))
                // Filter by `splitChunks.cacheGroups.{cacheGroup}.chunks`
                .filter(|c| (cache_group.chunk_filter)(c, chunk_group_db))
                .collect::<Box<[_]>>();

              // Filter by `splitChunks.cacheGroups.{cacheGroup}.minChunks`
              if selected_chunks.len() < cache_group.min_chunks as usize {
                tracing::trace!(
                  "Module({:?}) is ignored by CacheGroup({:?}). Reason: selected_chunks.len({:?}) < cache_group.min_chunks({:?})",
                  module.identifier(),
                  cache_group.key,
                  selected_chunks.len(),
                  cache_group.min_chunks,
                );
                continue;
              }
              let selected_chunks_key = Self::get_key(selected_chunks.iter().map(|chunk| &chunk.ukey), chunk_idx_map);

              merge_matched_item_into_module_group_map(
                MatchedItem {
                  module,
                  cache_group,
                  cache_group_index,
                  selected_chunks,
                  selected_chunks_key,
                },
                module_group_map,
              )
              .await;

              #[tracing::instrument(skip_all)]
              async fn merge_matched_item_into_module_group_map(
                matched_item: MatchedItem<'_>,
                module_group_map: &DashMap<String, ModuleGroup>,
              ) {
                let MatchedItem {
                  module,
                  cache_group_index,
                  cache_group,
                  selected_chunks,
                  selected_chunks_key,
                } = matched_item;

                // `Module`s with the same chunk_name would be merged togother.
                // `Module`s could be in different `ModuleGroup`s.
                let chunk_name: Option<String> = (cache_group.name)(module).await;

                let key: String = if let Some(cache_group_name) = &chunk_name {
                  [&cache_group.key, " name:", cache_group_name].join("")
                } else {
                  [&cache_group.key, " chunks:", selected_chunks_key.to_string().as_str()].join("")
                };

                let mut module_group = module_group_map.entry(key).or_insert_with(|| ModuleGroup {
                  modules: Default::default(),
                  cache_group_index,
                  cache_group_priority: cache_group.priority,
                  cache_group_reuse_existing_chunk: cache_group.reuse_existing_chunk,
                  sizes: Default::default(),
                  chunks: Default::default(),
                  chunk_name,
                });

                module_group.add_module(module);
                module_group
                  .chunks
                  .extend(selected_chunks.iter().map(|c| c.ukey))
              }
            }
          });
        }
      }
    });

    module_group_map.into_iter().collect()
  }

  #[tracing::instrument(skip_all)]
  pub(crate) fn remove_all_modules_from_other_module_groups(
    &self,
    current_module_group: &ModuleGroup,
    module_group_map: &mut ModuleGroupMap,
    used_chunks: &FxHashSet<ChunkUkey>,
    compilation: &mut Compilation,
  ) {
    // remove all modules from other entries and update size
    let keys_of_invalid_group = module_group_map
      .iter_mut()
      .par_bridge()
      .filter(|(_key, each_module_group)| {
        // Fast path: check whether has overlap on chunks
        each_module_group
          .chunks
          .intersection(used_chunks)
          .next()
          .is_some()
      })
      .filter_map(|(key, other_module_group)| {
        current_module_group.modules.iter().for_each(|module| {
          if other_module_group.modules.contains(module) {
            tracing::trace!("remove module({module}) from {key}");
            let module = compilation
              .module_graph
              .module_by_identifier(module)
              .unwrap_or_else(|| panic!("Module({module}) not found"));
            other_module_group.remove_module(&**module);
          }
        });

        if other_module_group.modules.is_empty() {
          tracing::trace!(
            "{key} is deleted for having empty modules",
          );
          return Some(key.clone());
        }

        tracing::trace!("other_module_group: {other_module_group:#?}");
        tracing::trace!("item.modules: {:#?}", current_module_group.modules);

        // Since there are modules removed, make sure the rest of chunks are all used.
        other_module_group.chunks.retain(|c| {
          let is_used_chunk = other_module_group
            .modules
            .iter()
            .any(|m| compilation.chunk_graph.is_module_in_chunk(m, *c));
          is_used_chunk
        });

        let cache_group = &self.cache_groups[other_module_group.cache_group_index];

        // Since we removed some modules and chunks from the `other_module_group`. There are chances
        // that the `min_chunks` and `min_size` validation is not satisfied anymore.

        // Validate `min_chunks` again
        if other_module_group.chunks.len() < cache_group.min_chunks as usize {
          tracing::trace!(
            "{key} is deleted for each_module_group.chunks.len()({:?}) < cache_group.min_chunks({:?})",
            other_module_group.chunks.len(),
            cache_group.min_chunks
          );
          return Some(key.clone());
        }

        // Validate `min_size` again
        if Self::remove_min_size_violating_modules(key, compilation, other_module_group, cache_group) {
          tracing::trace!(
            "{key} is deleted for violating min_size {:#?}",
            cache_group.min_size,
          );
          return Some(key.clone());
        }

        None
      })
      .collect::<Vec<_>>();

    keys_of_invalid_group.into_iter().for_each(|key| {
      module_group_map.remove(&key);
    });
  }

  fn get_key<'a, I: Iterator<Item = &'a ChunkUkey>>(
    chunks: I,
    chunk_idx_map: &FxHashMap<ChunkUkey, ChunksKey>,
  ) -> ChunksKey {
    let mut sorted = chunks.collect::<Vec<_>>();
    sorted.sort_unstable();

    let mut result: ChunksKey = 1usize.into();
    for chunk in sorted {
      let idx = chunk_idx_map
        .get(chunk)
        .expect("This should never happen, please file an issue");
      result |= idx;
    }
    result
  }

  #[allow(clippy::type_complexity)]
  fn prepare_combination_maps(
    module_graph: &ModuleGraph,
    chunk_graph: &ChunkGraph,
    chunk_idx_map: &FxHashMap<ChunkUkey, ChunksKey>,
  ) -> (
    FxHashMap<ChunksKey, FxHashSet<ChunkUkey>>,
    FxHashMap<usize, Vec<FxHashSet<ChunkUkey>>>,
  ) {
    let mut chunk_sets_in_graph = FxHashMap::default();

    for module in module_graph.modules().keys() {
      let chunks = chunk_graph.get_module_chunks(*module);
      let chunk_key = Self::get_key(chunks.iter(), chunk_idx_map);

      chunk_sets_in_graph.insert(chunk_key, chunks.clone());
    }

    let mut chunk_sets_by_count = FxHashMap::<usize, Vec<FxHashSet<ChunkUkey>>>::default();

    for chunks in chunk_sets_in_graph.values() {
      let count = chunks.len();
      chunk_sets_by_count
        .entry(count)
        .and_modify(|set| set.push(chunks.clone()))
        .or_insert(vec![chunks.clone()]);
    }

    (chunk_sets_in_graph, chunk_sets_by_count)
  }
}
