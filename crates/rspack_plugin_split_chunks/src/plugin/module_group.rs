use std::{
  cmp::Ordering,
  collections::BinaryHeap,
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
  module_group::{IndexedCacheGroup, ModuleGroup},
  options::{
    cache_group::CacheGroup,
    cache_group_test::{CacheGroupTest, CacheGroupTestFnCtx},
    chunk_name::{ChunkNameGetter, ChunkNameGetterFnCtx},
  },
};

type ChunksKey = u64;

#[derive(Default)]
pub(crate) struct AffectedModuleGroupIndex {
  chunk_to_groups: FxHashMap<ChunkUkey, Vec<String>>,
  module_to_groups: IdentifierMap<Vec<String>>,
}

impl AffectedModuleGroupIndex {
  pub(crate) fn from_module_group_map(module_group_map: &ModuleGroupMap) -> Self {
    let mut index = Self::default();

    for (key, module_group) in module_group_map.iter() {
      for chunk in &module_group.chunks {
        index
          .chunk_to_groups
          .entry(*chunk)
          .or_default()
          .push(key.clone());
      }

      for module in &module_group.modules {
        index
          .module_to_groups
          .entry(*module)
          .or_default()
          .push(key.clone());
      }
    }

    index
  }

  pub(crate) fn collect_affected_keys<'a>(
    &'a self,
    current_module_group: &'a ModuleGroup,
    used_chunks: &FxHashSet<ChunkUkey>,
  ) -> Vec<&'a str> {
    let chunk_refs = used_chunks
      .iter()
      .filter_map(|chunk| self.chunk_to_groups.get(chunk))
      .map(Vec::len)
      .sum::<usize>();
    let module_refs = current_module_group
      .modules
      .iter()
      .filter_map(|module| self.module_to_groups.get(module))
      .map(Vec::len)
      .sum::<usize>();

    if chunk_refs == 0 || module_refs == 0 {
      return Vec::new();
    }

    let mut seed = FxHashSet::default();
    let mut affected = FxHashSet::default();

    if chunk_refs <= module_refs {
      for keys in used_chunks
        .iter()
        .filter_map(|chunk| self.chunk_to_groups.get(chunk))
      {
        seed.extend(keys.iter().map(String::as_str));
      }

      for keys in current_module_group
        .modules
        .iter()
        .filter_map(|module| self.module_to_groups.get(module))
      {
        for key in keys {
          let key = key.as_str();
          if seed.contains(key) {
            affected.insert(key);
          }
        }
      }
    } else {
      for keys in current_module_group
        .modules
        .iter()
        .filter_map(|module| self.module_to_groups.get(module))
      {
        seed.extend(keys.iter().map(String::as_str));
      }

      for keys in used_chunks
        .iter()
        .filter_map(|chunk| self.chunk_to_groups.get(chunk))
      {
        for key in keys {
          let key = key.as_str();
          if seed.contains(key) {
            affected.insert(key);
          }
        }
      }
    }

    affected.into_iter().collect()
  }
}

struct ModuleGroupHeapEntry {
  cache_group_index: u32,
  chunks_len: usize,
  key: String,
  modules: Vec<ModuleIdentifier>,
  size_reduce: f64,
  version: u32,
}

impl ModuleGroupHeapEntry {
  fn from_module_group(key: &str, version: u32, module_group: &ModuleGroup) -> Self {
    let chunks_len = module_group.chunks.len();
    let size_reduce = module_group.get_total_size() * chunks_len.saturating_sub(1) as f64;

    Self {
      cache_group_index: module_group.cache_group_index,
      chunks_len,
      key: key.to_string(),
      modules: module_group.ordered_module_identifiers(),
      size_reduce,
      version,
    }
  }
}

impl PartialEq for ModuleGroupHeapEntry {
  fn eq(&self, other: &Self) -> bool {
    self.version == other.version
      && self.cache_group_index == other.cache_group_index
      && self.chunks_len == other.chunks_len
      && self.key == other.key
      && self.modules == other.modules
      && self.size_reduce.to_bits() == other.size_reduce.to_bits()
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
    self
      .chunks_len
      .cmp(&other.chunks_len)
      .then_with(|| self.size_reduce.total_cmp(&other.size_reduce))
      .then_with(|| other.cache_group_index.cmp(&self.cache_group_index))
      .then_with(|| self.modules.len().cmp(&other.modules.len()))
      .then_with(|| self.modules.cmp(&other.modules))
      .then_with(|| self.key.cmp(&other.key))
      .then_with(|| self.version.cmp(&other.version))
  }
}

#[derive(Default)]
pub(crate) struct ModuleGroupQueue {
  heap: BinaryHeap<ModuleGroupHeapEntry>,
  versions: FxHashMap<String, u32>,
}

impl ModuleGroupQueue {
  pub(crate) fn from_module_group_map(module_group_map: &ModuleGroupMap) -> Self {
    let mut queue = Self::default();

    for (key, module_group) in module_group_map.iter() {
      queue.versions.insert(key.clone(), 0);
      queue.heap.push(ModuleGroupHeapEntry::from_module_group(
        key,
        0,
        module_group,
      ));
    }

    queue
  }

  pub(crate) fn pop_best(
    &mut self,
    module_group_map: &mut ModuleGroupMap,
  ) -> Option<(String, ModuleGroup)> {
    while let Some(entry) = self.heap.pop() {
      let Some(version) = self.versions.get(&entry.key) else {
        continue;
      };

      if *version != entry.version {
        continue;
      }

      self.versions.remove(&entry.key);
      if let Some(module_group) = module_group_map.swap_remove(&entry.key) {
        return Some((entry.key, module_group));
      }
    }

    None
  }

  pub(crate) fn refresh(&mut self, key: &str, module_group: &ModuleGroup) {
    let version = self
      .versions
      .entry(key.to_string())
      .and_modify(|version| *version += 1)
      .or_insert(0);
    self.heap.push(ModuleGroupHeapEntry::from_module_group(
      key,
      *version,
      module_group,
    ));
  }

  pub(crate) fn remove(&mut self, key: &str) {
    self.versions.remove(key);
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
    affected_module_group_index: &AffectedModuleGroupIndex,
    module_group_queue: &mut ModuleGroupQueue,
    module_group_map: &mut ModuleGroupMap,
    used_chunks: &FxHashSet<ChunkUkey>,
    compilation: &Compilation,
    module_sizes: &ModuleSizes,
  ) {
    // remove all modules from other entries and update size
    let keys_of_invalid_group = affected_module_group_index
      .collect_affected_keys(current_module_group, used_chunks)
      .into_iter()
      .filter_map(|key| {
        let other_module_group = module_group_map.get_mut(key)?;

        if !other_module_group
          .chunks
          .iter()
          .any(|chunk| used_chunks.contains(chunk))
        {
          return None;
        }

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
          // nothing is removed
          return None;
        }

        if other_module_group.modules.is_empty() {
          tracing::trace!(
            "{key} is deleted for having empty modules",
          );
          return Some(key.to_string());
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
          return Some(key.to_string());
        }

        // Validate `min_size` again
        if remove_min_size_violating_modules(&cache_group.key, other_module_group, cache_group, module_sizes)
          || !Self::check_min_size_reduction(&other_module_group.get_sizes(module_sizes), &cache_group.min_size_reduction, other_module_group.chunks.len()) {
          tracing::trace!(
            "{key} is deleted for violating min_size {:#?}",
            cache_group.min_size,
          );
          return Some(key.to_string());
        }

        module_group_queue.refresh(key, other_module_group);
        None
      })
      .collect::<Vec<_>>();

    keys_of_invalid_group.into_iter().for_each(|key| {
      module_group_queue.remove(&key);
      module_group_map.swap_remove(&key);
    });
  }
}

#[allow(clippy::too_many_arguments)]
fn process_chunk_combination_fast_path(
  module_identifier: ModuleIdentifier,
  cache_group_index: u32,
  cache_group: &CacheGroup,
  chunk_combination: &FxHashSet<ChunkUkey>,
  removed_chunks: Option<&FxHashSet<ChunkUkey>>,
  module_group_map: &mut ModuleGroupMap,
  compilation: &Compilation,
  chunk_index_map: &FxHashMap<ChunkUkey, u32>,
) {
  if chunk_combination.is_empty() {
    return;
  }

  if chunk_combination.len() < cache_group.min_chunks as usize {
    return;
  }

  if matches!(cache_group.chunk_filter, ChunkFilter::All) {
    if chunk_combination
      .iter()
      .any(|chunk| removed_chunks.is_some_and(|removed_chunks| removed_chunks.contains(chunk)))
    {
      return;
    }

    merge_matched_item_into_module_group_map_with_set(
      module_identifier,
      cache_group_index,
      cache_group,
      chunk_combination,
      module_group_map,
      chunk_index_map,
    );

    return;
  }

  let selected_chunks = chunk_combination
    .iter()
    .filter(|chunk| cache_group.chunk_filter.test_internal(chunk, compilation))
    .copied()
    .collect::<Vec<_>>();

  if selected_chunks.len() < cache_group.min_chunks as usize {
    return;
  }

  if selected_chunks
    .iter()
    .any(|chunk| removed_chunks.is_some_and(|removed_chunks| removed_chunks.contains(chunk)))
  {
    return;
  }

  merge_matched_item_into_module_group_map_sync(
    module_identifier,
    cache_group_index,
    cache_group,
    &selected_chunks,
    module_group_map,
    chunk_index_map,
  );
}

fn merge_module_group_maps(left: &mut ModuleGroupMap, right: ModuleGroupMap) {
  for (key, module_group) in right {
    if let Some(existing) = left.get_mut(&key) {
      for module in module_group.modules {
        existing.add_module(module);
      }
      existing.chunks.extend(module_group.chunks);
    } else {
      left.insert(key, module_group);
    }
  }
}

#[allow(clippy::too_many_arguments)]
async fn process_chunk_combination(
  module: &dyn Module,
  module_identifier: ModuleIdentifier,
  cache_group_index: u32,
  cache_group: &CacheGroup,
  chunk_combination: &FxHashSet<ChunkUkey>,
  removed_chunks: Option<&FxHashSet<ChunkUkey>>,
  module_group_map: &FxDashMap<String, ModuleGroup>,
  compilation: &Compilation,
  chunk_index_map: &FxHashMap<ChunkUkey, u32>,
) -> Result<()> {
  if chunk_combination.is_empty() {
    return Ok(());
  }

  if chunk_combination.len() < cache_group.min_chunks as usize {
    return Ok(());
  }

  if matches!(cache_group.chunk_filter, ChunkFilter::All) && !cache_group.name.is_fn() {
    if chunk_combination
      .iter()
      .any(|chunk| removed_chunks.is_some_and(|removed_chunks| removed_chunks.contains(chunk)))
    {
      return Ok(());
    }

    merge_matched_item_into_module_group_map_with_set_static_name(
      module_identifier,
      cache_group_index,
      cache_group,
      chunk_combination,
      module_group_map,
      chunk_index_map,
    );

    return Ok(());
  }

  let selected_chunks = if cache_group.chunk_filter.is_func() {
    join_all(chunk_combination.iter().map(|chunk| async move {
      cache_group
        .chunk_filter
        .test_func(chunk, compilation)
        .await
        .map(|filtered| (chunk, filtered))
    }))
    .await
    .into_iter()
    .collect::<Result<Vec<_>>>()?
    .into_iter()
    .filter_map(|(chunk, filtered)| if filtered { Some(chunk) } else { None })
    .copied()
    .collect::<Vec<_>>()
  } else {
    chunk_combination
      .iter()
      .filter(|chunk| cache_group.chunk_filter.test_internal(chunk, compilation))
      .copied()
      .collect::<Vec<_>>()
  };

  if selected_chunks.len() < cache_group.min_chunks as usize {
    return Ok(());
  }

  if selected_chunks
    .iter()
    .any(|chunk| removed_chunks.is_some_and(|removed_chunks| removed_chunks.contains(chunk)))
  {
    return Ok(());
  }

  merge_matched_item_into_module_group_map(
    MatchedItem {
      module,
      cache_group,
      cache_group_index,
      selected_chunks,
    },
    module_group_map,
    compilation,
    chunk_index_map,
  )
  .await
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

fn merge_matched_item_into_module_group_map_sync(
  module_identifier: ModuleIdentifier,
  cache_group_index: u32,
  cache_group: &CacheGroup,
  selected_chunks: &[ChunkUkey],
  module_group_map: &mut ModuleGroupMap,
  chunk_index_map: &FxHashMap<ChunkUkey, u32>,
) {
  let chunk_name = match &cache_group.name {
    ChunkNameGetter::String(name) => Some(name.clone()),
    ChunkNameGetter::Disabled => None,
    ChunkNameGetter::Fn(_) => unreachable!("parallel fast path should exclude dynamic names"),
  };

  let key = if let Some(cache_group_name) = &chunk_name {
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

  let module_group = module_group_map
    .entry(key)
    .or_insert_with(|| ModuleGroup::new(chunk_name, cache_group_index, cache_group));
  module_group.add_module(module_identifier);
  module_group.chunks.extend(selected_chunks.iter().copied());
}

fn merge_matched_item_into_module_group_map_with_set(
  module_identifier: ModuleIdentifier,
  cache_group_index: u32,
  cache_group: &CacheGroup,
  selected_chunks: &FxHashSet<ChunkUkey>,
  module_group_map: &mut ModuleGroupMap,
  chunk_index_map: &FxHashMap<ChunkUkey, u32>,
) {
  let chunk_name = match &cache_group.name {
    ChunkNameGetter::String(name) => Some(name.clone()),
    ChunkNameGetter::Disabled => None,
    ChunkNameGetter::Fn(_) => unreachable!("parallel fast path should exclude dynamic names"),
  };

  let key = if let Some(cache_group_name) = &chunk_name {
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

  let module_group = module_group_map
    .entry(key)
    .or_insert_with(|| ModuleGroup::new(chunk_name, cache_group_index, cache_group));
  module_group.add_module(module_identifier);
  module_group.chunks.extend(selected_chunks.iter().copied());
}

fn merge_matched_item_into_module_group_map_with_set_static_name(
  module_identifier: ModuleIdentifier,
  cache_group_index: u32,
  cache_group: &CacheGroup,
  selected_chunks: &FxHashSet<ChunkUkey>,
  module_group_map: &FxDashMap<String, ModuleGroup>,
  chunk_index_map: &FxHashMap<ChunkUkey, u32>,
) {
  let chunk_name = match &cache_group.name {
    ChunkNameGetter::String(name) => Some(name.clone()),
    ChunkNameGetter::Disabled => None,
    ChunkNameGetter::Fn(_) => unreachable!("static-name fast path should exclude dynamic names"),
  };

  let key = if let Some(cache_group_name) = &chunk_name {
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

  let mut module_group = module_group_map
    .entry(key)
    .or_insert_with(|| ModuleGroup::new(chunk_name, cache_group_index, cache_group));
  module_group.add_module(module_identifier);
  module_group.chunks.extend(selected_chunks.iter().copied());
}
