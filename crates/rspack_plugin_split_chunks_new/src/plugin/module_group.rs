use async_scoped::TokioScope;
use dashmap::DashMap;
use rayon::prelude::*;
use rspack_core::{Chunk, ChunkUkey, Compilation, Module};
use rustc_hash::FxHashSet;

use super::ModuleGroupMap;
use crate::SplitChunksPlugin;
use crate::{
  cache_group::CacheGroup,
  module_group::{compare_entries, ModuleGroup},
};

impl SplitChunksPlugin {
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
      .expect("item should exist");
    (best_entry_key, best_module_group)
  }

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
    }

    let module_group_map: DashMap<String, ModuleGroup> = DashMap::default();

    async_scoped::Scope::scope_and_block(|scope: &mut TokioScope<'_, _>| {
      for module in compilation.module_graph.modules().values() {
        let module = &**module;

        let belong_to_chunks = compilation
          .chunk_graph
          .get_module_chunks((*module).identifier());

        let module_group_map = &module_group_map;

        for (cache_group_index, cache_group) in self.cache_groups.iter().enumerate() {
          scope.spawn(async move {
            // Filter by `splitChunks.cacheGroups.{cacheGroup}.test`
            let is_match_the_test: bool = (cache_group.test)(module);

            if !is_match_the_test {
              tracing::trace!(
                "Module({:?}) is ignored by CacheGroup({:?}). Reason: is_match_the_test({:?})",
                module.identifier(),
                cache_group.key,
                is_match_the_test,
              );
              return;
            }

            let selected_chunks = belong_to_chunks
              .iter()
              .map(|c| chunk_db.get(c).expect("Should have a chunk here"))
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
              return;
            }

            merge_matched_item_into_module_group_map(
              MatchedItem {
                module,
                cache_group,
                cache_group_index,
                selected_chunks,
              },
              module_group_map,
            )
            .await;

            async fn merge_matched_item_into_module_group_map(
              matched_item: MatchedItem<'_>,
              module_group_map: &DashMap<String, ModuleGroup>,
            ) {
              let MatchedItem {
                module,
                cache_group_index,
                cache_group,
                selected_chunks,
              } = matched_item;

              // `Module`s with the same chunk_name would be merged togother.
              // `Module`s could be in different `ModuleGroup`s.
              let chunk_name: Option<String> = (cache_group.name)(module).await;

              let key: String = if let Some(cache_group_name) = &chunk_name {
                [&cache_group.key, " name:", cache_group_name].join("")
              } else {
                [&cache_group.key, " index:", &cache_group_index.to_string()].join("")
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
          });
        }
      }
    });

    module_group_map.into_iter().collect()
  }

  pub(crate) fn remove_all_modules_from_other_module_groups(
    &self,
    item: &ModuleGroup,
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
      .filter_map(|(key, each_module_group)| {
        item.modules.iter().for_each(|module| {
          if each_module_group.modules.contains(module) {
            tracing::trace!("remove module({module}) from {key}");
            let module = compilation
              .module_graph
              .module_by_identifier(module)
              .unwrap_or_else(|| panic!("Module({module}) not found"));
            each_module_group.remove_module(&**module);
          }
        });

        if each_module_group.modules.is_empty() {
          tracing::trace!(
            "{key} is deleted for having empty modules",
          );
          return Some(key.clone());
        }

        tracing::trace!("each_module_group: {each_module_group:#?}");
        tracing::trace!("item.modules: {:#?}", item.modules);

        // Since there are modules removed, make sure the rest of chunks are all used.
        each_module_group.chunks.retain(|c| {
          let is_used_chunk = each_module_group
            .modules
            .iter()
            .any(|m| compilation.chunk_graph.is_module_in_chunk(m, *c));
          is_used_chunk
        });

        let cache_group: &CacheGroup = &self.cache_groups[each_module_group.cache_group_index];

        // There are some unused chunks, which is removed
        if each_module_group.chunks.len() < cache_group.min_chunks as usize {
          // `min_size` is not satisfied, remove this invalid `ModuleGroup`
          tracing::trace!(
            "{key} is deleted for each_module_group.chunks.len()({:?}) < cache_group.min_chunks({:?})",
            each_module_group.chunks.len(),
            cache_group.min_chunks
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
}
