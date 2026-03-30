use std::{
  cmp::Ordering,
  hash::{Hash, Hasher},
};

use futures::future::join_all;
use rayon::prelude::*;
use rspack_collections::IdentifierMap;
use rspack_core::{
  ChunkByUkey, ChunkUkey, Compilation, ExportsInfoArtifact, Module, ModuleIdentifier,
  PrefetchExportsInfoMode, RuntimeKeyMap, UsageKey, get_runtime_key,
};
use rspack_error::{Result, ToStringResultToRspackResultExt};
use rspack_util::{
  fx_hash::{FxDashMap, FxIndexMap},
  tracing_preset::TRACING_BENCH_TARGET,
};
use rustc_hash::{FxHashMap, FxHashSet, FxHasher};
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

type ChunksKey = u64;

#[derive(Clone, Copy)]
struct ModuleGroupBucketPosition {
  chunk_len: usize,
  index: usize,
}

#[derive(Default)]
pub(crate) struct ModuleGroupChunkLenBuckets {
  buckets: Vec<Vec<String>>,
  positions: FxHashMap<String, ModuleGroupBucketPosition>,
  max_chunk_len: usize,
}

impl ModuleGroupChunkLenBuckets {
  pub(crate) fn from_module_group_map(module_group_map: &ModuleGroupMap) -> Self {
    let mut buckets = Self::default();

    for (key, module_group) in module_group_map.iter() {
      buckets.insert(key.clone(), module_group.chunks.len());
    }

    buckets
  }

  pub(crate) fn pop_best(
    &mut self,
    module_group_map: &mut ModuleGroupMap,
  ) -> Option<(String, ModuleGroup)> {
    if self.positions.is_empty() {
      return None;
    }

    self.trim_max_chunk_len();

    let bucket = self.buckets.get(self.max_chunk_len)?;
    let best_entry_key = bucket
      .iter()
      .map(|key| (key, module_group_map.get(key).expect("should have item")))
      .min_by(|a, b| {
        let result = compare_entries((a.0, a.1), (b.0, b.1));
        if result < 0f64 {
          Ordering::Greater
        } else if result > 0f64 {
          Ordering::Less
        } else {
          Ordering::Equal
        }
      })
      .map(|(key, _)| key.clone())?;

    self.remove(&best_entry_key);
    let best_module_group = module_group_map
      .swap_remove(&best_entry_key)
      .expect("module group buckets and map should stay in sync");
    Some((best_entry_key, best_module_group))
  }

  pub(crate) fn refresh(&mut self, key: &str, chunk_len: usize) {
    let Some(position) = self.positions.get(key).copied() else {
      return;
    };

    if position.chunk_len == chunk_len {
      return;
    }

    self.remove(key);
    self.insert(key.to_string(), chunk_len);
  }

  pub(crate) fn remove(&mut self, key: &str) {
    let Some(position) = self.positions.remove(key) else {
      return;
    };

    let bucket = self
      .buckets
      .get_mut(position.chunk_len)
      .expect("bucket should exist");
    let removed_key = bucket.swap_remove(position.index);
    debug_assert_eq!(removed_key, key);

    if let Some(swapped_key) = bucket.get(position.index) {
      let swapped_position = self
        .positions
        .get_mut(swapped_key)
        .expect("swapped key should exist");
      swapped_position.index = position.index;
    }

    if position.chunk_len == self.max_chunk_len {
      self.trim_max_chunk_len();
    }
  }

  fn insert(&mut self, key: String, chunk_len: usize) {
    if self.buckets.len() <= chunk_len {
      self.buckets.resize_with(chunk_len + 1, Vec::new);
    }

    let bucket = &mut self.buckets[chunk_len];
    let index = bucket.len();
    bucket.push(key.clone());
    self
      .positions
      .insert(key, ModuleGroupBucketPosition { chunk_len, index });
    self.max_chunk_len = self.max_chunk_len.max(chunk_len);
  }

  fn trim_max_chunk_len(&mut self) {
    while self.max_chunk_len > 0
      && self
        .buckets
        .get(self.max_chunk_len)
        .is_some_and(Vec::is_empty)
    {
      self.max_chunk_len -= 1;
    }
  }
}

/// If a module meets requirements of a `ModuleGroup`. We consider the `Module` and the `CacheGroup`
/// to be a `MatchedItem`, which are consumed later to calculate `ModuleGroup`.
struct MatchedItem<'a> {
  module: &'a dyn Module,
  cache_group_index: u32,
  cache_group: &'a CacheGroup,
  selected_chunks: Vec<ChunkUkey>,
}

fn get_key<I: Iterator<Item = ChunkUkey>>(
  chunks: I,
  chunk_index_map: &FxHashMap<ChunkUkey, u32>,
) -> ChunksKey {
  let mut sorted_chunk_ukeys = chunks
    .map(|chunk| {
      // Increment each chunk index by 1 to avoid hashing the value 0 with FxHasher, which would always return a hash of 0
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
  combinations: FxHashMap<ChunksKey, Vec<FxHashSet<ChunkUkey>>>,
  used_exports_combinations: FxHashMap<ChunksKey, Vec<FxHashSet<ChunkUkey>>>,
  grouped_by_exports: IdentifierMap<Vec<ChunksKey>>,
}

impl Combinator {
  fn group_chunks_by_exports(
    module_identifier: &ModuleIdentifier,
    module_chunks: impl Iterator<Item = ChunkUkey>,
    exports_info_artifact: &ExportsInfoArtifact,
    chunk_by_ukey: &ChunkByUkey,
  ) -> Vec<FxHashSet<ChunkUkey>> {
    let exports_info = exports_info_artifact
      .get_prefetched_exports_info(module_identifier, PrefetchExportsInfoMode::Default);
    let mut grouped_by_used_exports: FxHashMap<UsageKey, FxHashSet<ChunkUkey>> = Default::default();
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
    chunk_index_map: &FxHashMap<ChunkUkey, u32>,
  ) -> Vec<FxHashSet<ChunkUkey>> {
    if used_exports {
      let mut result = vec![];
      let chunks_by_module_used = self
        .grouped_by_exports
        .get(&module)
        .expect("should have exports for module");

      for chunks_key in chunks_by_module_used.iter() {
        let combs = self
          .used_exports_combinations
          .get(chunks_key)
          .expect("should have combinations")
          .clone();
        result.extend(combs.into_iter());
      }

      result
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

  fn get_combinations(
    chunk_sets_in_graph: FxHashMap<ChunksKey, FxHashSet<ChunkUkey>>,
    chunk_sets_by_count: FxIndexMap<u32, Vec<FxHashSet<ChunkUkey>>>,
  ) -> FxHashMap<ChunksKey, Vec<FxHashSet<ChunkUkey>>> {
    chunk_sets_in_graph
      .into_par_iter()
      .map(|(chunks_key, chunks_set)| {
        let mut result = vec![];
        for (count, array_of_set) in chunk_sets_by_count.iter() {
          if *count < chunks_set.len() as u32 {
            for set in array_of_set {
              if set.is_subset(&chunks_set) {
                result.push(set.clone());
              }
            }
          } else {
            break;
          }
        }
        result.push(chunks_set);
        (chunks_key, result)
      })
      .collect::<FxHashMap<_, _>>()
  }

  pub(crate) fn prepare_group_by_chunks(
    &mut self,
    all_modules: &[ModuleIdentifier],
    module_chunks: &ModuleChunks,
    chunk_index_map: &FxHashMap<ChunkUkey, u32>,
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

    let mut chunk_sets_by_count = FxIndexMap::<u32, Vec<FxHashSet<ChunkUkey>>>::default();
    for chunks in chunk_sets_in_graph.values() {
      let count = chunks.len();

      chunk_sets_by_count
        .entry(count as u32)
        .and_modify(|set| set.push(chunks.clone()))
        .or_insert(vec![chunks.clone()]);
    }

    chunk_sets_by_count.sort_keys();

    self.combinations = Self::get_combinations(chunk_sets_in_graph, chunk_sets_by_count);
  }

  pub(crate) fn prepare_group_by_used_exports(
    &mut self,
    all_modules: &[ModuleIdentifier],
    exports_info_artifact: &ExportsInfoArtifact,
    chunk_by_ukey: &ChunkByUkey,
    module_chunks: &ModuleChunks,
    chunk_index_map: &FxHashMap<ChunkUkey, u32>,
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
    let mut used_exports_chunk_sets_by_count =
      FxIndexMap::<u32, Vec<FxHashSet<ChunkUkey>>>::default();
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
          .push(chunks);
      }
    }

    used_exports_chunk_sets_by_count.sort_keys();

    self.used_exports_combinations = Self::get_combinations(
      used_exports_chunk_sets_in_graph,
      used_exports_chunk_sets_by_count,
    );
  }
}

impl SplitChunksPlugin {
  #[allow(clippy::too_many_arguments)]
  #[instrument(name = "Compilation:SplitChunks:prepare_module_group_map",target=TRACING_BENCH_TARGET, skip_all)]
  pub(crate) async fn prepare_module_group_map(
    &self,
    combinator: &Combinator,
    all_modules: &[ModuleIdentifier],
    cache_groups: Vec<IndexedCacheGroup<'_>>,
    removed_module_chunks: &IdentifierMap<FxHashSet<ChunkUkey>>,
    compilation: &Compilation,
    module_chunks: &ModuleChunks,
    chunk_index_map: &FxHashMap<ChunkUkey, u32>,
  ) -> Result<ModuleGroupMap> {
    let module_graph = compilation.get_module_graph();
    let module_group_map: FxDashMap<String, ModuleGroup> = FxDashMap::default();
    let module_group_results = rspack_futures::scope::<_, Result<_>>(|token| {
      all_modules.iter().for_each(|mid| {
        let s = unsafe { token.used((&cache_groups, mid, &module_graph, compilation, &module_group_map, &combinator, module_chunks, removed_module_chunks, chunk_index_map)) };
        s.spawn(|(cache_groups, mid, module_graph, compilation, module_group_map, combinator, module_chunks, removed_module_chunks, chunk_index_map)| async move {
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

            for chunk_combination in combs {
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
                module_group_map,
                compilation,
                chunk_index_map,
              ).await?;
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

    // Sort the module_group_map by key to ensure deterministic iteration order
    let mut result: Vec<_> = module_group_map.into_iter().collect();
    result.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(result.into_iter().collect())
  }

  // #[tracing::instrument(skip_all)]
  pub(crate) fn remove_all_modules_from_other_module_groups(
    &self,
    current_module_group: &ModuleGroup,
    module_group_chunk_len_buckets: &mut ModuleGroupChunkLenBuckets,
    module_group_map: &mut ModuleGroupMap,
    used_chunks: &FxHashSet<ChunkUkey>,
    compilation: &Compilation,
    module_sizes: &ModuleSizes,
  ) {
    // remove all modules from other entries and update size
    let bucket_updates = module_group_map
      .par_iter_mut()
      .filter_map(|(key, other_module_group)| {
        other_module_group
          .chunks
          .intersection(used_chunks)
          .next()?;

        let old_chunk_len = other_module_group.chunks.len();
        let module_count = other_module_group.modules.len();

        let duplicated_modules = if other_module_group.modules.len() > current_module_group.modules.len() {
          current_module_group.modules.intersection(&other_module_group.modules).copied().collect::<Vec<_>>()
        } else {
          other_module_group.modules.intersection(&current_module_group.modules).copied().collect::<Vec<_>>()
        };

        for module in duplicated_modules {
          other_module_group.remove_module(module);
        }

        if module_count == other_module_group.modules.len() {
          // nothing is removed
          return None;
        }

        if other_module_group.modules.is_empty() {
          tracing::trace!(
            "{key} is deleted for having empty modules",
          );
          return Some(ModuleGroupBucketUpdate::Remove(key.clone()));
        }

        tracing::trace!("other_module_group: {other_module_group:#?}");
        tracing::trace!("item.modules: {:#?}", current_module_group.modules);

        // Since there are modules removed, make sure the rest of chunks are all used.
        other_module_group.chunks.retain(|c| {
          compilation.build_chunk_graph_artifact.chunk_graph
            .is_any_module_in_chunk(other_module_group.modules.iter(), *c)
        });

        let cache_group = other_module_group.get_cache_group(&self.cache_groups);

        // Since we removed some modules and chunks from the `other_module_group`. There are chances
        // that the `min_chunks` and `min_size` validation is not satisfied anymore.

        // Validate `min_chunks` again
        if other_module_group.chunks.len() < cache_group.min_chunks as usize {
          tracing::trace!(
            "{key} is deleted for each_module_group.chunks.len()({:?}) < cache_group.min_chunks({:?})",
            other_module_group.chunks.len(),
            cache_group.min_chunks
          );
          return Some(ModuleGroupBucketUpdate::Remove(key.clone()));
        }

        // Validate `min_size` again
        if remove_min_size_violating_modules(&cache_group.key, other_module_group, cache_group, module_sizes)
          || !Self::check_min_size_reduction(&other_module_group.get_sizes(module_sizes), &cache_group.min_size_reduction, other_module_group.chunks.len()) {
          tracing::trace!(
            "{key} is deleted for violating min_size {:#?}",
            cache_group.min_size,
          );
          return Some(ModuleGroupBucketUpdate::Remove(key.clone()));
        }

        let new_chunk_len = other_module_group.chunks.len();
        if old_chunk_len != new_chunk_len {
          return Some(ModuleGroupBucketUpdate::Refresh {
            key: key.clone(),
            chunk_len: new_chunk_len,
          });
        }

        None
      })
      .collect::<Vec<_>>();

    bucket_updates.into_iter().for_each(|update| match update {
      ModuleGroupBucketUpdate::Remove(key) => {
        module_group_chunk_len_buckets.remove(&key);
        module_group_map.swap_remove(&key);
      }
      ModuleGroupBucketUpdate::Refresh { key, chunk_len } => {
        module_group_chunk_len_buckets.refresh(&key, chunk_len);
      }
    });
  }
}

enum ModuleGroupBucketUpdate {
  Remove(String),
  Refresh { key: String, chunk_len: usize },
}

async fn merge_matched_item_into_module_group_map(
  matched_item: MatchedItem<'_>,
  module_group_map: &FxDashMap<String, ModuleGroup>,
  compilation: &Compilation,
  chunk_index_map: &FxHashMap<ChunkUkey, u32>,
) -> Result<()> {
  let MatchedItem {
    module,
    cache_group_index,
    cache_group,
    selected_chunks,
  } = matched_item;

  // `Module`s with the same chunk_name would be merged togother.
  // `Module`s could be in different `ModuleGroup`s.
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

  let mut module_group = {
    module_group_map
      .entry(key)
      .or_insert_with(|| ModuleGroup::new(chunk_name, cache_group_index, cache_group))
  };
  module_group.add_module(module.identifier());
  module_group.chunks.extend(selected_chunks.iter().copied());

  Ok(())
}

#[cfg(test)]
mod tests {
  use rspack_core::{ChunkUkey, ModuleIdentifier, SourceType};
  use rustc_hash::FxHashMap;

  use super::*;
  use crate::{
    CacheGroup, CacheGroupTest, ChunkNameGetter, SplitChunkSizes, create_all_chunk_filter,
    create_default_module_layer_filter, create_default_module_type_filter,
  };

  fn test_cache_group() -> CacheGroup {
    CacheGroup {
      key: "test".to_string(),
      chunk_filter: create_all_chunk_filter(),
      test: CacheGroupTest::Enabled,
      r#type: create_default_module_type_filter(),
      layer: create_default_module_layer_filter(),
      name: ChunkNameGetter::Disabled,
      priority: 0.0,
      min_size: SplitChunkSizes::default(),
      min_size_reduction: SplitChunkSizes::default(),
      reuse_existing_chunk: false,
      min_chunks: 1,
      id_hint: String::new(),
      max_initial_requests: f64::INFINITY,
      max_async_requests: f64::INFINITY,
      max_async_size: SplitChunkSizes::default(),
      max_initial_size: SplitChunkSizes::default(),
      filename: None,
      automatic_name_delimiter: "~".to_string(),
      used_exports: false,
    }
  }

  fn test_module_sizes(entries: &[(&str, f64)]) -> ModuleSizes {
    let mut module_sizes = ModuleSizes::default();

    for (identifier, size) in entries {
      module_sizes.insert(
        ModuleIdentifier::from(*identifier),
        FxHashMap::from_iter([(SourceType::JavaScript, *size)]),
      );
    }

    module_sizes
  }

  fn test_module_group(
    cache_group: &CacheGroup,
    module_sizes: &ModuleSizes,
    key: &str,
    chunk_lens: &[u32],
    module_ids: &[&str],
  ) -> (String, ModuleGroup) {
    let mut module_group = ModuleGroup::new(None, 0, cache_group);
    for module_id in module_ids {
      module_group.add_module(ModuleIdentifier::from(*module_id));
    }
    module_group
      .chunks
      .extend(chunk_lens.iter().copied().map(ChunkUkey::from));
    module_group.get_sizes(module_sizes);
    (key.to_string(), module_group)
  }

  #[test]
  fn picks_best_from_highest_chunk_len_bucket() {
    let cache_group = test_cache_group();
    let module_sizes = test_module_sizes(&[("a", 10.0), ("b", 100.0)]);
    let mut module_group_map = ModuleGroupMap::default();

    let (a_key, a_group) = test_module_group(&cache_group, &module_sizes, "a", &[1, 2, 3], &["a"]);
    let (b_key, b_group) = test_module_group(&cache_group, &module_sizes, "b", &[1, 2], &["b"]);
    module_group_map.insert(a_key, a_group);
    module_group_map.insert(b_key, b_group);

    let mut buckets = ModuleGroupChunkLenBuckets::from_module_group_map(&module_group_map);
    let (best_key, _) = buckets.pop_best(&mut module_group_map).unwrap();

    assert_eq!(best_key, "a");
    assert_eq!(buckets.max_chunk_len, 2);
  }

  #[test]
  fn keeps_compare_entries_behavior_within_bucket() {
    let cache_group = test_cache_group();
    let module_sizes = test_module_sizes(&[("a", 10.0), ("b", 20.0), ("c", 100.0)]);
    let mut module_group_map = ModuleGroupMap::default();

    let (a_key, a_group) = test_module_group(&cache_group, &module_sizes, "a", &[1, 2, 3], &["a"]);
    let (b_key, b_group) = test_module_group(&cache_group, &module_sizes, "b", &[4, 5, 6], &["b"]);
    let (c_key, c_group) = test_module_group(&cache_group, &module_sizes, "c", &[7, 8], &["c"]);
    module_group_map.insert(a_key, a_group);
    module_group_map.insert(b_key, b_group);
    module_group_map.insert(c_key, c_group);

    let mut buckets = ModuleGroupChunkLenBuckets::from_module_group_map(&module_group_map);
    let (best_key, _) = buckets.pop_best(&mut module_group_map).unwrap();

    assert_eq!(best_key, "b");
  }

  #[test]
  fn refresh_moves_group_between_buckets() {
    let cache_group = test_cache_group();
    let module_sizes = test_module_sizes(&[("a", 10.0), ("b", 20.0)]);
    let mut module_group_map = ModuleGroupMap::default();

    let (a_key, a_group) = test_module_group(&cache_group, &module_sizes, "a", &[1, 2, 3], &["a"]);
    let (b_key, b_group) = test_module_group(&cache_group, &module_sizes, "b", &[4, 5], &["b"]);
    module_group_map.insert(a_key, a_group);
    module_group_map.insert(b_key, b_group);

    let mut buckets = ModuleGroupChunkLenBuckets::from_module_group_map(&module_group_map);

    module_group_map.get_mut("a").unwrap().chunks = [ChunkUkey::from(1)].into_iter().collect();
    buckets.refresh("a", 1);

    let (best_key, _) = buckets.pop_best(&mut module_group_map).unwrap();

    assert_eq!(best_key, "b");
    assert_eq!(buckets.max_chunk_len, 1);
  }
}
