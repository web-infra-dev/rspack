use std::{
  cmp::Ordering,
  hash::{Hash, Hasher},
  ops::Deref,
  sync::Arc,
};

use futures::future::join_all;
use rayon::prelude::*;
use rspack_collections::IdentifierMap;
use rspack_core::{
  ChunkByUkey, ChunkUkey, Compilation, ExportsInfoArtifact, Module, ModuleIdentifier,
  RuntimeKeyMap, UsageKey, get_runtime_key,
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
  common::{ChunkFilter, ModuleChunks, ModuleSizes, ModuleTypeFilter},
  min_size::remove_min_size_violating_modules,
  module_group::{IndexedCacheGroup, ModuleGroup, ModuleGroupKey, compare_entries},
  options::{
    cache_group::CacheGroup,
    cache_group_test::{CacheGroupTest, CacheGroupTestFnCtx},
    chunk_name::{ChunkNameGetter, ChunkNameGetterFnCtx},
  },
};

type ChunksKey = u64;
type ChunkFilterCacheKey = (u32, ChunksKey);

struct TypeFilteredCacheGroups<'a> {
  all: Vec<IndexedCacheGroup<'a>>,
  regex: Vec<IndexedCacheGroup<'a>>,
  exact: FxHashMap<String, Vec<IndexedCacheGroup<'a>>>,
}

impl<'a> TypeFilteredCacheGroups<'a> {
  fn new(cache_groups: &[IndexedCacheGroup<'a>]) -> Self {
    let mut all = Vec::new();
    let mut regex = Vec::new();
    let mut exact = FxHashMap::default();

    for cache_group in cache_groups.iter().copied() {
      match &cache_group.cache_group.r#type {
        ModuleTypeFilter::All => all.push(cache_group),
        ModuleTypeFilter::Regex(_) => regex.push(cache_group),
        ModuleTypeFilter::String(module_type) => exact
          .entry(module_type.clone())
          .or_insert_with(Vec::new)
          .push(cache_group),
      }
    }

    Self { all, regex, exact }
  }

  fn candidates(&self, module_type: &str) -> Vec<IndexedCacheGroup<'a>> {
    let mut candidates = Vec::with_capacity(
      self.all.len() + self.regex.len() + self.exact.get(module_type).map_or(0, Vec::len),
    );

    candidates.extend(self.all.iter().copied());

    if let Some(exact) = self.exact.get(module_type) {
      candidates.extend(exact.iter().copied());
    }

    candidates.extend(
      self
        .regex
        .iter()
        .copied()
        .filter(|cache_group| cache_group.cache_group.r#type.test_internal(module_type)),
    );

    candidates.sort_unstable_by(|a, b| a.compare_by_index(b));
    candidates
  }
}

#[derive(Clone)]
struct ChunkCombination {
  key: ChunksKey,
  chunks: Arc<FxHashSet<ChunkUkey>>,
}

impl Deref for ChunkCombination {
  type Target = FxHashSet<ChunkUkey>;

  fn deref(&self) -> &Self::Target {
    &self.chunks
  }
}

/// If a module meets requirements of a `ModuleGroup`. We consider the `Module` and the `CacheGroup`
/// to be a `MatchedItem`, which are consumed later to calculate `ModuleGroup`.
struct MatchedItem<'a> {
  module: &'a dyn Module,
  cache_group_index: u32,
  cache_group: &'a CacheGroup,
  selected_chunks: Vec<ChunkUkey>,
  selected_chunks_key: Option<ChunksKey>,
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
  combinations: FxHashMap<ChunksKey, Vec<ChunkCombination>>,
  used_exports_combinations: FxHashMap<ChunksKey, Vec<ChunkCombination>>,
  grouped_by_exports: IdentifierMap<Vec<ChunksKey>>,
}

impl Combinator {
  fn get_non_used_exports_combs(
    &self,
    module: ModuleIdentifier,
    module_chunks: &ModuleChunks,
    chunk_index_map: &FxHashMap<ChunkUkey, u32>,
  ) -> &[ChunkCombination] {
    let chunks = module_chunks
      .get(&module)
      .expect("should have module chunks");
    let chunks_key = get_key(chunks.iter().copied(), chunk_index_map);
    self
      .combinations
      .get(&chunks_key)
      .expect("should have combinations")
  }

  fn get_used_exports_combs(&self, module: ModuleIdentifier) -> Vec<&ChunkCombination> {
    let mut result = vec![];
    let chunks_by_module_used = self
      .grouped_by_exports
      .get(&module)
      .expect("should have exports for module");

    for chunks_key in chunks_by_module_used.iter() {
      let combs = self
        .used_exports_combinations
        .get(chunks_key)
        .expect("should have combinations");
      result.extend(combs.iter());
    }

    result
  }

  fn group_chunks_by_exports(
    module_identifier: &ModuleIdentifier,
    module_chunks: impl Iterator<Item = ChunkUkey>,
    exports_info_artifact: &ExportsInfoArtifact,
    chunk_by_ukey: &ChunkByUkey,
    chunk_index_map: &FxHashMap<ChunkUkey, u32>,
  ) -> Vec<ChunkCombination> {
    let exports_info = exports_info_artifact.get_exports_info_data(module_identifier);
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

    grouped_by_used_exports
      .into_values()
      .map(|chunks| ChunkCombination {
        key: get_key(chunks.iter().copied(), chunk_index_map),
        chunks: Arc::new(chunks),
      })
      .collect()
  }

  fn get_combs(
    &self,
    module: ModuleIdentifier,
    used_exports: bool,
    module_chunks: &ModuleChunks,
    chunk_index_map: &FxHashMap<ChunkUkey, u32>,
  ) -> Vec<ChunkCombination> {
    if used_exports {
      self
        .get_used_exports_combs(module)
        .into_iter()
        .cloned()
        .collect()
    } else {
      self
        .get_non_used_exports_combs(module, module_chunks, chunk_index_map)
        .to_vec()
    }
  }

  fn get_combinations(
    chunk_sets_in_graph: FxHashMap<ChunksKey, ChunkCombination>,
    chunk_sets_by_count: FxIndexMap<u32, Vec<ChunkCombination>>,
  ) -> FxHashMap<ChunksKey, Vec<ChunkCombination>> {
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
        Some((
          chunk_key,
          ChunkCombination {
            key: chunk_key,
            chunks: Arc::new(chunks.clone()),
          },
        ))
      })
      .collect::<FxHashMap<_, _>>();

    let mut chunk_sets_by_count = FxIndexMap::<u32, Vec<ChunkCombination>>::default();
    for chunks in chunk_sets_in_graph.values() {
      let count = chunks.len();

      chunk_sets_by_count
        .entry(count as u32)
        .and_modify(|set| set.push(chunks.clone()))
        .or_insert_with(|| vec![chunks.clone()]);
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
          chunk_index_map,
        );
        let mut grouped_chunks_key = vec![];
        let mut used_exports_chunks = FxHashMap::default();
        for chunks in grouped_chunks {
          if chunks.is_empty() {
            continue;
          }
          used_exports_chunks.insert(chunks.key, chunks.clone());
          grouped_chunks_key.push(chunks.key);
        }
        Some(((*module, grouped_chunks_key), used_exports_chunks))
      })
      .unzip();

    self.grouped_by_exports = module_grouped_chunks.into_iter().collect();

    let mut used_exports_chunk_sets_in_graph = FxHashMap::default();
    let mut used_exports_chunk_sets_by_count = FxIndexMap::<u32, Vec<ChunkCombination>>::default();
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
  // #[tracing::instrument(skip_all)]
  pub(crate) fn find_best_module_group(
    &self,
    module_group_map: &mut ModuleGroupMap,
  ) -> (ModuleGroupKey, ModuleGroup) {
    debug_assert!(!module_group_map.is_empty());

    let best_entry_key = module_group_map
      .keys()
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
      .map(|(key, _)| key.clone())
      .expect("at least have one item");

    let best_module_group = module_group_map
      .swap_remove(&best_entry_key)
      .expect("This should never happen, please file an issue");
    (best_entry_key, best_module_group)
  }

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
    let module_group_map: FxDashMap<ModuleGroupKey, ModuleGroup> = FxDashMap::default();
    let type_filtered_cache_groups = TypeFilteredCacheGroups::new(&cache_groups);
    let sync_chunk_filter_result_cache: FxDashMap<ChunkFilterCacheKey, Arc<Vec<ChunkUkey>>> =
      FxDashMap::default();
    let module_group_results = rspack_parallel::scope::<_, Result<_>>(|token| {
      all_modules.iter().for_each(|mid| {
        let s = unsafe { token.used((&type_filtered_cache_groups, &sync_chunk_filter_result_cache, mid, &module_graph, compilation, &module_group_map, &combinator, module_chunks, removed_module_chunks, chunk_index_map)) };
        s.spawn(|(type_filtered_cache_groups, sync_chunk_filter_result_cache, mid, module_graph, compilation, module_group_map, combinator, module_chunks, removed_module_chunks, chunk_index_map)| async move {
          let belong_to_chunks = module_chunks.get(mid).expect("should have module chunks");
          if belong_to_chunks.is_empty() {
            return Ok(());
          }

          let removed_chunks = removed_module_chunks.get(mid);
          let available_chunks_upper_bound = removed_chunks.map_or(belong_to_chunks.len(), |removed| {
            belong_to_chunks.len() - belong_to_chunks.intersection(removed).count()
          });

          if available_chunks_upper_bound == 0 {
            return Ok(());
          }

          let module = module_graph.module_by_identifier(mid).expect("should have module").as_ref();
          let module_type = module.module_type().as_str();
          let layer = module.get_layer().map(|layer| layer.as_str());
          let mut name_for_condition: Option<Option<Box<str>>> = None;
          let mut sync_chunk_filter_upper_bounds: FxHashMap<u32, usize> = FxHashMap::default();
          let mut filtered = vec![];

          for cache_group in type_filtered_cache_groups.candidates(module_type) {
            if available_chunks_upper_bound < cache_group.cache_group.min_chunks as usize {
              continue;
            }

            if !cache_group.cache_group.chunk_filter.is_func() {
              let selected_chunks_upper_bound = *sync_chunk_filter_upper_bounds
                .entry(cache_group.cache_group_index)
                .or_insert_with(|| match &cache_group.cache_group.chunk_filter {
                  ChunkFilter::All => available_chunks_upper_bound,
                  chunk_filter => belong_to_chunks
                    .iter()
                    .filter(|chunk| {
                      !removed_chunks.is_some_and(|chunks| chunks.contains(chunk))
                        && chunk_filter.test_internal(chunk, compilation)
                    })
                    .count(),
                });

              if selected_chunks_upper_bound < cache_group.cache_group.min_chunks as usize {
                continue;
              }
            }

            // Filter by `splitChunks.cacheGroups.{cacheGroup}.layer`
            let is_match = if cache_group.cache_group.layer.is_func() {
              cache_group
                .cache_group
                .layer
                .test_func(layer.map(str::to_owned))
                .await?
            } else {
              cache_group
                .cache_group
                .layer
                .test_internal(layer)
            };
            if !is_match {
              continue;
            }

            // Filter by `splitChunks.cacheGroups.{cacheGroup}.test`
            let is_match = match &cache_group.cache_group.test {
              CacheGroupTest::String(str) => name_for_condition
                .get_or_insert_with(|| module.name_for_condition())
                .as_deref()
                .is_some_and(|name| name.starts_with(str)),
              CacheGroupTest::RegExp(regexp) => name_for_condition
                .get_or_insert_with(|| module.name_for_condition())
                .as_deref()
                .is_some_and(|name| regexp.test(name)),
              CacheGroupTest::Fn(f) => {
                let ctx = CacheGroupTestFnCtx { compilation, module };
                f(ctx).await?.unwrap_or_default()
              }
              CacheGroupTest::Enabled => true,
            };

            if !is_match {
              continue;
            }

            filtered.push(cache_group);
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
                let cache_key = (cache_group_index, chunk_combination.key);
                if let Some(selected_chunks) = sync_chunk_filter_result_cache.get(&cache_key) {
                  selected_chunks.value().as_ref().clone()
                } else {
                  let selected_chunks = chunk_combination.iter().filter(|c| {
                    cache_group.chunk_filter.test_internal(c, compilation)
                  }).copied().collect::<Vec<_>>();
                  sync_chunk_filter_result_cache.insert(
                    cache_key,
                    Arc::new(selected_chunks.clone()),
                  );
                  selected_chunks
                }
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

              if selected_chunks
                .iter()
                .any(|c| removed_chunks.is_some_and(|chunks| chunks.contains(c)))
              {
                continue;
              }
              let selected_chunks_key = matches!(&cache_group.chunk_filter, ChunkFilter::All)
                .then_some(chunk_combination.key);
              merge_matched_item_into_module_group_map(
                MatchedItem {
                  module,
                  cache_group,
                  cache_group_index,
                  selected_chunks,
                  selected_chunks_key,
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
    module_group_map: &mut ModuleGroupMap,
    used_chunks: &FxHashSet<ChunkUkey>,
    compilation: &Compilation,
    module_sizes: &ModuleSizes,
  ) {
    // remove all modules from other entries and update size
    let keys_of_invalid_group = module_group_map
      .par_iter_mut()
      .filter_map(|(key, other_module_group)| {
        other_module_group
          .chunks
          .intersection(used_chunks)
          .next()?;

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
          return Some(key.clone());
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
          return Some(key.clone());
        }

        // Validate `min_size` again
        if remove_min_size_violating_modules(key, other_module_group, cache_group, module_sizes)
          || !Self::check_min_size_reduction(&other_module_group.get_sizes(module_sizes), &cache_group.min_size_reduction, other_module_group.chunks.len()) {
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
      module_group_map.swap_remove(&key);
    });
  }
}

async fn merge_matched_item_into_module_group_map(
  matched_item: MatchedItem<'_>,
  module_group_map: &FxDashMap<ModuleGroupKey, ModuleGroup>,
  compilation: &Compilation,
  chunk_index_map: &FxHashMap<ChunkUkey, u32>,
) -> Result<()> {
  let MatchedItem {
    module,
    cache_group_index,
    cache_group,
    selected_chunks,
    selected_chunks_key,
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
  let key = if let Some(cache_group_name) = &chunk_name {
    ModuleGroupKey::Named {
      cache_group_index,
      chunk_name: cache_group_name.clone(),
    }
  } else {
    ModuleGroupKey::Anonymous {
      cache_group_index,
      chunks_key: selected_chunks_key
        .unwrap_or_else(|| get_key(selected_chunks.iter().copied(), chunk_index_map)),
    }
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
