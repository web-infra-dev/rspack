use std::{
  cmp::Ordering,
  collections::BinaryHeap,
  hash::{Hash, Hasher},
  sync::{Arc, RwLock},
};

const SHARD_COUNT: usize = 256;

fn shard_id(key: &str) -> usize {
  let mut hasher = FxHasher::default();
  key.hash(&mut hasher);
  (hasher.finish() as usize) % SHARD_COUNT
}
use futures::future::join_all;
use rayon::prelude::*;
use rspack_collections::{IdentifierMap, UkeyIndexMap, UkeyMap, UkeySet};
use rspack_core::{
  ChunkByUkey, ChunkUkey, Compilation, ExportsInfoArtifact, Module, ModuleIdentifier,
  PrefetchExportsInfoMode, RuntimeKeyMap, UsageKey, get_runtime_key,
};
use rspack_error::{Result, ToStringResultToRspackResultExt};
use rspack_util::tracing_preset::TRACING_BENCH_TARGET;
use rustc_hash::{FxHashMap, FxHasher};
use tracing::instrument;

use super::ModuleGroupMap;
use crate::{
  SplitChunksPlugin,
  common::{ModuleChunks, ModuleSizes},
  min_size::remove_min_size_violating_modules,
  module_group::{IndexedCacheGroup, ModuleGroup, compare_entries},
  options::{
    cache_group::CacheGroup,
    cache_group_test::{CacheGroupTest, CacheGroupTestFnCtx},
    chunk_name::{ChunkNameGetter, ChunkNameGetterFnCtx},
  },
};

/// Heap entry for selecting the best ModuleGroup. Ordered so that the "best" entry
/// (by compare_entries) is the max in the BinaryHeap.
pub(crate) struct ModuleGroupHeapEntry {
  pub key: String,
  pub group: ModuleGroup,
}

impl PartialEq for ModuleGroupHeapEntry {
  fn eq(&self, other: &Self) -> bool {
    self.key == other.key
  }
}

impl Eq for ModuleGroupHeapEntry {}

impl PartialOrd for ModuleGroupHeapEntry {
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    Some(self.cmp(other))
  }
}

impl Ord for ModuleGroupHeapEntry {
  fn cmp(&self, other: &Self) -> Ordering {
    let result = compare_entries((&self.key, &self.group), (&other.key, &other.group));
    if result < 0f64 {
      Ordering::Less
    } else if result > 0f64 {
      Ordering::Greater
    } else {
      Ordering::Equal
    }
  }
}

type ChunksKey = u64;

/// If a module meets requirements of a `ModuleGroup`. We consider the `Module` and the `CacheGroup`
/// to be a `MatchedItem`, which are consumed later to calculate `ModuleGroup`.
struct MatchedItem<'a> {
  module: &'a dyn Module,
  cache_group_index: usize,
  cache_group: &'a CacheGroup,
  selected_chunks: Vec<ChunkUkey>,
}

fn get_key<I: Iterator<Item = ChunkUkey>>(
  chunks: I,
  chunk_index_map: &UkeyMap<ChunkUkey, u64>,
) -> ChunksKey {
  let mut sorted_chunk_ukeys = chunks
    .map(|chunk| {
      // Increment each usize by 1 to avoid hashing the value 0 with FxHasher, which would always return a hash of 0
      *chunk_index_map
        .get(&chunk)
        .expect("should already have index for chunk ukey")
    })
    .collect::<Vec<_>>();
  sorted_chunk_ukeys.sort_unstable();
  let mut hasher = FxHasher::default();
  for chunk_ukey in sorted_chunk_ukeys {
    chunk_ukey.hash(&mut hasher);
  }
  hasher.finish()
}

#[derive(Default)]
pub(crate) struct Combinator {
  combinations: FxHashMap<ChunksKey, Arc<[UkeySet<ChunkUkey>]>>,
  used_exports_combinations: FxHashMap<ChunksKey, Arc<[UkeySet<ChunkUkey>]>>,
  grouped_by_exports: IdentifierMap<Vec<ChunksKey>>,
}

impl Combinator {
  fn group_chunks_by_exports(
    module_identifier: &ModuleIdentifier,
    module_chunks: impl Iterator<Item = ChunkUkey>,
    exports_info_artifact: &ExportsInfoArtifact,
    chunk_by_ukey: &ChunkByUkey,
  ) -> Vec<UkeySet<ChunkUkey>> {
    let exports_info = exports_info_artifact
      .get_prefetched_exports_info(module_identifier, PrefetchExportsInfoMode::Default);
    let mut grouped_by_used_exports: FxHashMap<UsageKey, UkeySet<ChunkUkey>> = Default::default();
    let mut runtime_key_map = RuntimeKeyMap::default();
    for chunk_ukey in module_chunks {
      let chunk = chunk_by_ukey.expect_get(&chunk_ukey);
      let runtime = chunk.runtime();
      let usage_key = runtime_key_map
        .entry(get_runtime_key(runtime).clone())
        .or_insert_with(|| exports_info.get_usage_key(Some(runtime)))
        .clone();

      grouped_by_used_exports
        .entry(usage_key)
        .or_default()
        .insert(chunk_ukey);
    }

    grouped_by_used_exports.into_values().collect()
  }

  fn get_combs(
    &self,
    module: ModuleIdentifier,
    used_exports: bool,
    module_chunks: &ModuleChunks,
    chunk_index_map: &UkeyMap<ChunkUkey, u64>,
  ) -> Arc<[UkeySet<ChunkUkey>]> {
    if used_exports {
      let chunks_by_module_used = self
        .grouped_by_exports
        .get(&module)
        .expect("should have exports for module");
      let mut result = Vec::new();
      for chunks_key in chunks_by_module_used.iter() {
        let combs = self
          .used_exports_combinations
          .get(chunks_key)
          .expect("should have combinations");
        result.extend(combs.iter().cloned());
      }
      Arc::from(result.into_boxed_slice())
    } else {
      let chunks = module_chunks
        .get(&module)
        .expect("should have module chunks");
      let chunks_key = get_key(chunks.iter().copied(), chunk_index_map);
      self
        .combinations
        .get(&chunks_key)
        .expect("should have combinations")
        .clone()
    }
  }

  /// Build combinations: for each chunk set S in the graph, collect all T in the graph
  /// with T.len() < S.len() and T.is_subset(S), then add S. Uses chunk_sets_by_count
  /// (count -> keys) to only iterate unique sets in the graph per count.
  fn get_combinations(
    chunk_sets_in_graph: &FxHashMap<ChunksKey, UkeySet<ChunkUkey>>,
    chunk_sets_by_count: UkeyIndexMap<u32, Vec<ChunksKey>>,
  ) -> FxHashMap<ChunksKey, Arc<[UkeySet<ChunkUkey>]>> {
    chunk_sets_in_graph
      .par_iter()
      .map(|(chunks_key, chunks_set)| {
        let mut result = vec![];
        for (count, keys) in chunk_sets_by_count.iter() {
          if *count >= chunks_set.len() as u32 {
            break;
          }
          for key in keys {
            let set = chunk_sets_in_graph
              .get(key)
              .expect("key from chunk_sets_by_count");
            if set.is_subset(chunks_set) {
              result.push(set.clone());
            }
          }
        }
        result.push(chunks_set.clone());
        (*chunks_key, Arc::from(result.into_boxed_slice()))
      })
      .collect::<FxHashMap<_, _>>()
  }

  pub(crate) fn prepare_group_by_chunks(
    &mut self,
    all_modules: &[ModuleIdentifier],
    module_chunks: &ModuleChunks,
    chunk_index_map: &UkeyMap<ChunkUkey, u64>,
  ) {
    let chunk_sets_in_graph = all_modules
      .par_iter()
      .filter_map(|module| {
        let chunks = module_chunks
          .get(module)
          .expect("should have module chunks");
        if chunks.is_empty() {
          return None;
        }
        let chunk_key = get_key(chunks.iter().copied(), chunk_index_map);
        Some((chunk_key, chunks.clone()))
      })
      .collect::<FxHashMap<_, _>>();

    let mut chunk_sets_by_count = UkeyIndexMap::<u32, Vec<ChunksKey>>::default();
    for (chunk_key, chunks) in &chunk_sets_in_graph {
      let count = chunks.len();
      chunk_sets_by_count
        .entry(count as u32)
        .or_default()
        .push(*chunk_key);
    }

    chunk_sets_by_count.sort_keys();

    self.combinations = Self::get_combinations(&chunk_sets_in_graph, chunk_sets_by_count);
  }

  pub(crate) fn prepare_group_by_used_exports(
    &mut self,
    all_modules: &[ModuleIdentifier],
    exports_info_artifact: &ExportsInfoArtifact,
    chunk_by_ukey: &ChunkByUkey,
    module_chunks: &ModuleChunks,
    chunk_index_map: &UkeyMap<ChunkUkey, u64>,
  ) {
    let (module_grouped_chunks, used_exports_chunks): (Vec<_>, Vec<_>) = all_modules
      .par_iter()
      .filter_map(|module| {
        let grouped_chunks = Self::group_chunks_by_exports(
          module,
          module_chunks
            .get(module)
            .expect("should have module chunks")
            .iter()
            .copied(),
          exports_info_artifact,
          chunk_by_ukey,
        );
        let mut grouped_chunks_key = vec![];
        let mut used_exports_chunks = FxHashMap::default();
        for chunks in grouped_chunks {
          if chunks.is_empty() {
            continue;
          }
          let chunk_key = get_key(chunks.iter().copied(), chunk_index_map);
          used_exports_chunks.insert(chunk_key, chunks);
          grouped_chunks_key.push(chunk_key);
        }
        Some(((*module, grouped_chunks_key), used_exports_chunks))
      })
      .unzip();

    self.grouped_by_exports = module_grouped_chunks.into_iter().collect();

    let mut used_exports_chunk_sets_in_graph = FxHashMap::default();
    let mut used_exports_chunk_sets_by_count = UkeyIndexMap::<u32, Vec<ChunksKey>>::default();
    for used_exports_chunks in used_exports_chunks {
      for (chunk_key, chunks) in used_exports_chunks {
        if used_exports_chunk_sets_in_graph
          .insert(chunk_key, chunks.clone())
          .is_some()
        {
          continue;
        }
        let count = chunks.len();
        used_exports_chunk_sets_by_count
          .entry(count as u32)
          .or_default()
          .push(chunk_key);
      }
    }

    used_exports_chunk_sets_by_count.sort_keys();

    self.used_exports_combinations = Self::get_combinations(
      &used_exports_chunk_sets_in_graph,
      used_exports_chunk_sets_by_count,
    );
  }
}

/// Result of removing modules from other groups: (key, chunks) to remove from map and reverse_index,
/// and (key, updated group, old chunks) to re-insert and push to heap and update reverse_index.
pub(crate) type RemoveAllModulesResult = (
  Vec<(String, UkeySet<ChunkUkey>)>,
  Vec<(String, ModuleGroup, UkeySet<ChunkUkey>)>,
);

impl SplitChunksPlugin {
  /// Pops from heap until we find a key still present in the map, then removes
  /// that key from the map and returns (key, group). Uses the map's value (current state).
  pub(crate) fn find_best_module_group_from_heap(
    &self,
    module_group_map: &mut ModuleGroupMap,
    heap: &mut BinaryHeap<ModuleGroupHeapEntry>,
  ) -> (String, ModuleGroup) {
    loop {
      let entry = heap.pop().expect("heap empty but map non-empty");
      if module_group_map.contains_key(&entry.key) {
        let group = module_group_map
          .swap_remove(&entry.key)
          .expect("key just checked");
        return (entry.key, group);
      }
    }
  }

  #[allow(clippy::too_many_arguments)]
  #[instrument(name = "Compilation:SplitChunks:prepare_module_group_map",target=TRACING_BENCH_TARGET, skip_all)]
  pub(crate) async fn prepare_module_group_map(
    &self,
    combinator: &Combinator,
    all_modules: &[ModuleIdentifier],
    cache_groups: Vec<IndexedCacheGroup<'_>>,
    removed_module_chunks: &IdentifierMap<UkeySet<ChunkUkey>>,
    compilation: &Compilation,
    module_chunks: &ModuleChunks,
    chunk_index_map: &UkeyMap<ChunkUkey, u64>,
  ) -> Result<ModuleGroupMap> {
    let module_graph = compilation.get_module_graph();
    let shards: Vec<RwLock<FxHashMap<String, ModuleGroup>>> = (0..SHARD_COUNT)
      .map(|_| RwLock::new(FxHashMap::default()))
      .collect();
    let module_group_results = rspack_futures::scope::<_, Result<_>>(|token| {
      all_modules.iter().for_each(|mid| {
        let s = unsafe {
          token.used((
            &cache_groups,
            mid,
            &module_graph,
            compilation,
            &shards,
            &combinator,
            module_chunks,
            removed_module_chunks,
            chunk_index_map,
          ))
        };
        s.spawn(
          |(cache_groups, mid, module_graph, compilation, shards, combinator, module_chunks, removed_module_chunks, chunk_index_map)| async move {
          let belong_to_chunks = module_chunks.get(mid).expect("should have module chunks");
          if belong_to_chunks.is_empty() {
            return Ok(());
          }

          if let Some(removed_chunks) = removed_module_chunks.get(mid) && belong_to_chunks.iter().all(|c| removed_chunks.contains(c)) {
            return Ok(());
          }
          let module = module_graph.module_by_identifier(mid).expect("should have module").as_ref();
          let mut filtered = vec![];

          for cache_group in cache_groups.iter() {
            let mut is_match = true;
            // Filter by `splitChunks.cacheGroups.{cacheGroup}.type`
            is_match &= (cache_group.cache_group.r#type)(module);
            // Filter by `splitChunks.cacheGroups.{cacheGroup}.layer`
            is_match &= (cache_group.cache_group.layer)(module.get_layer().map(ToString::to_string)).await?;

            // Filter by `splitChunks.cacheGroups.{cacheGroup}.test`
            is_match &= match &cache_group.cache_group.test {
              CacheGroupTest::String(str) => module
                .name_for_condition().is_some_and(|name| name.starts_with(str)),
              CacheGroupTest::RegExp(regexp) => module
                .name_for_condition().is_some_and(|name| regexp.test(&name)),
              CacheGroupTest::Fn(f) => {
                let ctx = CacheGroupTestFnCtx { compilation, module };
                f(ctx).await?.unwrap_or_default()
              }
              CacheGroupTest::Enabled => true,
            };

            if is_match {
              filtered.push(cache_group);
            }
          }
          let mut used_exports_combs = None;
          let mut non_used_exports_combs = None;

          for cache_group in filtered {
            let IndexedCacheGroup {
              cache_group_index,
              cache_group,
            } = cache_group;
            let combs = if cache_group.used_exports {
              if used_exports_combs.is_none() {
                used_exports_combs = Some(combinator.get_combs(
                  *mid,
                  true,
                  module_chunks,
                  chunk_index_map,
                ));
              }
              used_exports_combs.as_ref().expect("should have used_exports_combs")
            } else {
              if non_used_exports_combs.is_none() {
                non_used_exports_combs = Some(combinator.get_combs(
                  *mid,
                  false,
                  module_chunks,
                  chunk_index_map,
                ));
              }
              non_used_exports_combs.as_ref().expect("should have non_used_exports_combs")
            };

            for chunk_combination in combs.iter() {
              if chunk_combination.is_empty() {
                continue;
              }

              // Filter by `splitChunks.cacheGroups.{cacheGroup}.minChunks`
              if chunk_combination.len() < cache_group.min_chunks as usize {
                tracing::trace!(
                  "Module({:?}) is ignored by CacheGroup({:?}). Reason: chunk_combination.len({:?}) < cache_group.min_chunks({:?})",
                  mid,
                  cache_group.key,
                  chunk_combination.len(),
                  cache_group.min_chunks,
                );
                continue;
              }


              let selected_chunks = if cache_group.chunk_filter.is_func() {
                join_all(chunk_combination.iter().map(|c| async move {
                  // Filter by `splitChunks.cacheGroups.{cacheGroup}.chunks`
                  cache_group.chunk_filter.test_func(c, compilation).await.map(|filtered|  (c, filtered))
                }))
                  .await
                  .into_iter()
                  .collect::<Result<Vec<_>>>()?
                  .into_iter()
                  .filter_map(
                    |(chunk, filtered)| {
                      if filtered {
                        Some(chunk)
                      } else {
                        None
                      }
                    }
                  ).copied().collect::<Vec<_>>()
              } else {
                chunk_combination.iter().filter(|c| {
                  cache_group.chunk_filter.test_internal(c, compilation)
                }).copied().collect::<Vec<_>>()
              };

              // Filter by `splitChunks.cacheGroups.{cacheGroup}.minChunks`
              if selected_chunks.len() < cache_group.min_chunks as usize {
                tracing::trace!(
                  "Module({:?}) is ignored by CacheGroup({:?}). Reason: selected_chunks.len({:?}) < cache_group.min_chunks({:?})",
                  mid,
                  cache_group.key,
                  selected_chunks.len(),
                  cache_group.min_chunks,
                );
                continue;
              }

              if selected_chunks.iter().any(|c| removed_module_chunks.get(mid).is_some_and(|chunks| chunks.contains(c))) {
                continue;
              }
              merge_matched_item_into_module_group_map(
                MatchedItem {
                  module,
                  cache_group,
                  cache_group_index: *cache_group_index,
                  selected_chunks,
                },
                shards,
                compilation,
                chunk_index_map,
              )
              .await?;
            }
          }
          Ok(())
        });
      })
    })
    .await
    .into_iter().map(|r| r.to_rspack_result())
    .collect::<Result<Vec<_>>>()?;

    for result in module_group_results {
      result?;
    }

    let mut result: Vec<_> = shards
      .into_iter()
      .flat_map(|s| s.into_inner().expect("shard lock poisoned").into_iter())
      .collect();
    result.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(result.into_iter().collect())
  }

  // #[tracing::instrument(skip_all)]
  pub(crate) fn remove_all_modules_from_other_module_groups(
    &self,
    current_module_group: &ModuleGroup,
    module_group_map: &mut ModuleGroupMap,
    _used_chunks: &UkeySet<ChunkUkey>,
    affected_keys: &rustc_hash::FxHashSet<String>,
    compilation: &Compilation,
    module_sizes: &ModuleSizes,
  ) -> RemoveAllModulesResult {
    let mut to_remove = vec![];
    let mut to_update = vec![];

    for key in affected_keys {
      let Some(mut other_module_group) = module_group_map.swap_remove(key) else {
        continue;
      };

      let old_chunks = other_module_group.chunks.clone();
      let module_count = other_module_group.modules.len();

      let duplicated_modules =
        if other_module_group.modules.len() > current_module_group.modules.len() {
          current_module_group
            .modules
            .intersection(&other_module_group.modules)
            .copied()
            .collect::<Vec<_>>()
        } else {
          other_module_group
            .modules
            .intersection(&current_module_group.modules)
            .copied()
            .collect::<Vec<_>>()
        };

      for module in duplicated_modules {
        other_module_group.remove_module(module);
      }

      if module_count == other_module_group.modules.len() {
        module_group_map.insert(key.clone(), other_module_group);
        continue;
      }

      if other_module_group.modules.is_empty() {
        tracing::trace!("{key} is deleted for having empty modules");
        to_remove.push((key.clone(), old_chunks));
        continue;
      }

      tracing::trace!("other_module_group: {other_module_group:#?}");
      tracing::trace!("item.modules: {:#?}", current_module_group.modules);

      other_module_group.chunks.retain(|c| {
        compilation
          .build_chunk_graph_artifact
          .chunk_graph
          .is_any_module_in_chunk(other_module_group.modules.iter(), *c)
      });

      let cache_group = other_module_group.get_cache_group(&self.cache_groups);

      if other_module_group.chunks.len() < cache_group.min_chunks as usize {
        tracing::trace!(
          "{key} is deleted for each_module_group.chunks.len()({:?}) < cache_group.min_chunks({:?})",
          other_module_group.chunks.len(),
          cache_group.min_chunks
        );
        to_remove.push((key.clone(), old_chunks));
        continue;
      }

      if remove_min_size_violating_modules(
        &cache_group.key,
        &mut other_module_group,
        cache_group,
        module_sizes,
      ) || !Self::check_min_size_reduction(
        &other_module_group.get_sizes(module_sizes),
        &cache_group.min_size_reduction,
        other_module_group.chunks.len(),
      ) {
        tracing::trace!(
          "{key} is deleted for violating min_size {:#?}",
          cache_group.min_size,
        );
        to_remove.push((key.clone(), old_chunks));
        continue;
      }

      to_update.push((key.clone(), other_module_group, old_chunks));
    }

    (to_remove, to_update)
  }
}

async fn merge_matched_item_into_module_group_map(
  matched_item: MatchedItem<'_>,
  shards: &[RwLock<FxHashMap<String, ModuleGroup>>],
  compilation: &Compilation,
  chunk_index_map: &UkeyMap<ChunkUkey, u64>,
) -> Result<()> {
  let MatchedItem {
    module,
    cache_group_index,
    cache_group,
    selected_chunks,
  } = matched_item;

  let chunk_name = match &cache_group.name {
    ChunkNameGetter::String(name) => Some(name.clone()),
    ChunkNameGetter::Disabled => None,
    ChunkNameGetter::Fn(f) => {
      let ctx = ChunkNameGetterFnCtx {
        module,
        chunks: &selected_chunks,
        cache_group_key: &cache_group.key,
        compilation,
      };
      f(ctx).await?
    }
  };
  let key: String = if let Some(cache_group_name) = &chunk_name {
    let mut key = String::with_capacity(cache_group.key.len() + cache_group_name.len());
    key.push_str(&cache_group.key);
    key.push_str(cache_group_name);
    key
  } else {
    format!(
      "\0\0{}{:x}",
      &cache_group.key,
      get_key(selected_chunks.iter().copied(), chunk_index_map)
    )
  };

  let shard_id = shard_id(&key);
  let mut guard = shards[shard_id]
    .write()
    .expect("RwLock poisoned in split chunks shard");
  let module_group = guard
    .entry(key)
    .or_insert_with(|| ModuleGroup::new(chunk_name, cache_group_index, cache_group));
  module_group.add_module(module.identifier());
  module_group.chunks.extend(selected_chunks.iter().copied());

  Ok(())
}
