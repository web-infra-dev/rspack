use std::{
  cmp::Ordering,
  collections::{HashMap, hash_map},
  hash::{BuildHasherDefault, Hash, Hasher},
};

use dashmap::{DashMap, mapref::entry::Entry};
use futures::future::join_all;
use rayon::prelude::*;
use rspack_collections::{IdentifierMap, UkeyIndexMap, UkeySet};
use rspack_core::{
  ChunkByUkey, ChunkUkey, Compilation, Module, ModuleGraph, ModuleIdentifier,
  PrefetchExportsInfoMode, UsageKey,
};
use rspack_error::{Result, ToStringResultToRspackResultExt};
use rspack_util::{fx_hash::FxDashMap, tracing_preset::TRACING_BENCH_TARGET};
use rustc_hash::{FxHashMap, FxHashSet, FxHasher};
use tracing::instrument;

use super::ModuleGroupMap;
use crate::{
  SplitChunksPlugin,
  common::{ModuleChunks, ModuleSizes},
  module_group::{CacheGroupIdx, ModuleGroup, compare_entries},
  options::{
    cache_group::CacheGroup,
    cache_group_test::{CacheGroupTest, CacheGroupTestFnCtx},
    chunk_name::{ChunkNameGetter, ChunkNameGetterFnCtx},
  },
};

type ChunksKey = u64;

/// If a module meets requirements of a `ModuleGroup`. We consider the `Module` and the `CacheGroup`
/// to be a `MatchedItem`, which are consumed later to calculate `ModuleGroup`.
struct MatchedItem<'a> {
  idx: CacheGroupIdx,
  module: &'a dyn Module,
  cache_group_index: usize,
  cache_group: &'a CacheGroup,
  selected_chunks: Vec<ChunkUkey>,
  selected_chunks_key: ChunksKey,
}

#[derive(Default)]
struct IdentityHasher(u64);

impl Hasher for IdentityHasher {
  fn finish(&self) -> u64 {
    self.0
  }

  fn write(&mut self, _: &[u8]) {
    panic!("Invalid use of IdentityHasher");
  }

  fn write_u64(&mut self, i: u64) {
    self.0 = i;
  }
}

type ChunksKeyHashBuilder = BuildHasherDefault<IdentityHasher>;

fn get_key<I: Iterator<Item = ChunkUkey>>(chunks: I) -> ChunksKey {
  let mut sorted_chunk_ukeys = chunks
    .map(|chunk| {
      // Increment each usize by 1 to avoid hashing the value 0 with FxHasher, which would always return a hash of 0
      chunk.as_u32() + 1
    })
    .collect::<Vec<_>>();
  sorted_chunk_ukeys.sort_unstable();
  let mut hasher = FxHasher::default();
  for chunk_ukey in sorted_chunk_ukeys {
    chunk_ukey.hash(&mut hasher);
  }
  hasher.finish()
}

type ChunkSetsInGraph<'a> = (
  &'a FxHashMap<ChunksKey, UkeySet<ChunkUkey>>,
  &'a UkeyIndexMap<u32, Vec<UkeySet<ChunkUkey>>>,
);

#[derive(Default)]
struct Combinator {
  combinations_cache: FxDashMap<ChunksKey, Vec<UkeySet<ChunkUkey>>>,
  used_exports_combinations_cache: FxDashMap<ChunksKey, Vec<UkeySet<ChunkUkey>>>,

  chunk_sets_in_graph: FxHashMap<ChunksKey, UkeySet<ChunkUkey>>,
  chunk_sets_by_count: UkeyIndexMap<u32, Vec<UkeySet<ChunkUkey>>>,

  used_exports_chunk_sets_in_graph: FxHashMap<ChunksKey, UkeySet<ChunkUkey>>,
  used_exports_chunk_sets_by_count: UkeyIndexMap<u32, Vec<UkeySet<ChunkUkey>>>,

  grouped_by_exports: IdentifierMap<Vec<UkeySet<ChunkUkey>>>,
}

impl Combinator {
  fn group_chunks_by_exports(
    module_identifier: &ModuleIdentifier,
    module_chunks: impl Iterator<Item = ChunkUkey>,
    module_graph: &ModuleGraph,
    chunk_by_ukey: &ChunkByUkey,
  ) -> Vec<UkeySet<ChunkUkey>> {
    let exports_info =
      module_graph.get_prefetched_exports_info(module_identifier, PrefetchExportsInfoMode::Default);
    let mut grouped_by_used_exports: FxHashMap<UsageKey, UkeySet<ChunkUkey>> = Default::default();
    for chunk_ukey in module_chunks {
      let chunk = chunk_by_ukey.expect_get(&chunk_ukey);
      let usage_key = exports_info.get_usage_key(Some(chunk.runtime()));

      grouped_by_used_exports
        .entry(usage_key)
        .or_default()
        .insert(chunk_ukey);
    }

    grouped_by_used_exports.values().cloned().collect()
  }

  fn get_combination(
    &self,
    chunks_key: ChunksKey,
    combinations_cache: &FxDashMap<ChunksKey, Vec<UkeySet<ChunkUkey>>>,
    chunk_sets_in_graph: &FxHashMap<ChunksKey, UkeySet<ChunkUkey>>,
    chunk_sets_by_count: &UkeyIndexMap<u32, Vec<UkeySet<ChunkUkey>>>,
  ) -> Vec<UkeySet<ChunkUkey>> {
    match combinations_cache.entry(chunks_key) {
      Entry::Occupied(entry) => entry.get().clone(),
      Entry::Vacant(entry) => {
        let chunks_set = chunk_sets_in_graph
          .get(&chunks_key)
          .expect("This should never happen, please file an issue");

        let mut result = vec![chunks_set.clone()];

        for (count, array_of_set) in chunk_sets_by_count.iter() {
          if *count < chunks_set.len() as u32 {
            for set in array_of_set {
              if set.is_subset(chunks_set) {
                result.push(set.clone());
              }
            }
          } else {
            break;
          }
        }

        entry.insert(result.clone());
        result
      }
    }
  }

  fn get_combs(
    &self,
    module: ModuleIdentifier,
    used_exports: bool,
    module_chunks: &ModuleChunks,
  ) -> Vec<UkeySet<ChunkUkey>> {
    if used_exports {
      let (chunk_sets_in_graph, chunk_sets_by_count) = self.group_by_used_exports();

      let mut result = vec![];
      let chunks_by_module_used = self
        .grouped_by_exports
        .get(&module)
        .expect("should have exports for module");

      for chunks in chunks_by_module_used.iter() {
        let chunks_key = get_key(chunks.iter().copied());
        let combs = self.get_combination(
          chunks_key,
          &self.used_exports_combinations_cache,
          chunk_sets_in_graph,
          chunk_sets_by_count,
        );
        result.extend(combs.into_iter());
      }

      result
    } else {
      let (chunk_sets_in_graph, chunk_sets_by_count) = self.group_by_chunks();
      let chunks = module_chunks
        .get(&module)
        .expect("should have module chunks");
      self.get_combination(
        get_key(chunks.iter().copied()),
        &self.combinations_cache,
        chunk_sets_in_graph,
        chunk_sets_by_count,
      )
    }
  }

  fn prepare_group_by_chunks(
    &mut self,
    all_modules: &[ModuleIdentifier],
    module_chunks: &ModuleChunks,
  ) {
    self.chunk_sets_in_graph = all_modules
      .par_iter()
      .filter_map(|module| {
        let chunks = module_chunks
          .get(module)
          .expect("should have module chunks");
        if chunks.is_empty() {
          return None;
        }
        let chunk_key = get_key(chunks.iter().copied());
        Some((chunk_key, chunks.clone()))
      })
      .collect::<FxHashMap<_, _>>();

    for chunks in self.chunk_sets_in_graph.values() {
      let count = chunks.len();

      self
        .chunk_sets_by_count
        .entry(count as u32)
        .and_modify(|set| set.push(chunks.clone()))
        .or_insert(vec![chunks.clone()]);
    }

    self.chunk_sets_by_count.sort_keys();
  }

  fn group_by_chunks(&self) -> ChunkSetsInGraph<'_> {
    (&self.chunk_sets_in_graph, &self.chunk_sets_by_count)
  }

  fn prepare_group_by_used_exports(
    &mut self,
    all_modules: &[ModuleIdentifier],
    module_graph: &ModuleGraph,
    chunk_by_ukey: &ChunkByUkey,
    module_chunks: &ModuleChunks,
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
            .cloned(),
          module_graph,
          chunk_by_ukey,
        );
        let mut used_exports_chunks = FxHashMap::default();
        for chunks in &grouped_chunks {
          if chunks.is_empty() {
            continue;
          }
          let chunk_key = get_key(chunks.iter().copied());
          used_exports_chunks.insert(chunk_key, chunks.clone());
        }
        Some(((*module, grouped_chunks), used_exports_chunks))
      })
      .unzip();

    self.grouped_by_exports = module_grouped_chunks.into_iter().collect();

    for used_exports_chunks in used_exports_chunks {
      for (chunk_key, chunks) in used_exports_chunks {
        if self
          .used_exports_chunk_sets_in_graph
          .insert(chunk_key, chunks.clone())
          .is_some()
        {
          continue;
        }
        let count = chunks.len();
        self
          .used_exports_chunk_sets_by_count
          .entry(count as u32)
          .or_default()
          .push(chunks);
      }
    }

    self.used_exports_chunk_sets_by_count.sort_keys();
  }

  fn group_by_used_exports(&self) -> ChunkSetsInGraph<'_> {
    (
      &self.used_exports_chunk_sets_in_graph,
      &self.used_exports_chunk_sets_by_count,
    )
  }
}

impl SplitChunksPlugin {
  // #[tracing::instrument(skip_all)]
  pub(crate) fn find_best_module_group(
    &self,
    module_group_map: &mut ModuleGroupMap,
  ) -> (String, ModuleGroup) {
    debug_assert!(!module_group_map.is_empty());

    let best_entry_key = module_group_map
      .iter()
      .min_by(|a, b| {
        if compare_entries(a.1, b.1) < 0f64 {
          Ordering::Greater
        } else {
          Ordering::Less
        }
      })
      .map(|(key, _)| key.clone())
      .expect("at least have one item");

    let best_module_group = module_group_map
      .remove(&best_entry_key)
      .expect("This should never happen, please file an issue");
    (best_entry_key, best_module_group)
  }

  #[instrument(name = "Compilation:SplitChunks:prepare_module_group_map",target=TRACING_BENCH_TARGET, skip_all)]
  pub(crate) async fn prepare_module_group_map(
    &self,
    all_modules: &[ModuleIdentifier],
    compilation: &Compilation,
    module_sizes: &ModuleSizes,
    module_chunks: &ModuleChunks,
  ) -> Result<ModuleGroupMap> {
    let module_graph = compilation.get_module_graph();

    let module_group_map: DashMap<String, ModuleGroup> = DashMap::default();

    let mut combinator = Combinator::default();

    combinator.prepare_group_by_chunks(all_modules, module_chunks);

    if self
      .cache_groups
      .iter()
      .any(|cache_group| cache_group.used_exports)
    {
      combinator.prepare_group_by_used_exports(
        all_modules,
        &module_graph,
        &compilation.chunk_by_ukey,
        module_chunks,
      );
    }

    let module_group_results = rspack_futures::scope::<_, Result<_>>(|token| {
      all_modules.iter().for_each(|module| {
        let s = unsafe { token.used((&self, module, &module_graph, compilation, &module_group_map, &combinator, &module_sizes, module_chunks)) };
        s.spawn(|(plugin, module, module_graph, compilation, module_group_map, combinator, module_sizes, module_chunks)| async move {
          let belong_to_chunks = module_chunks.get(module).expect("should have module chunks");
          if belong_to_chunks.is_empty() {
            return Ok(());
          }

          let module = module_graph.module_by_identifier(module).expect("should have module").as_ref();
          let mut temp = Vec::with_capacity(plugin.cache_groups.len());

          for idx in 0..plugin.cache_groups.len() {
            let cache_group = &plugin.cache_groups[idx];

            let mut is_match = true;
            // Filter by `splitChunks.cacheGroups.{cacheGroup}.type`
            is_match &= (cache_group.r#type)(module);
            // Filter by `splitChunks.cacheGroups.{cacheGroup}.layer`
            is_match &= (cache_group.layer)(module.get_layer().map(ToString::to_string)).await?;

            // Filter by `splitChunks.cacheGroups.{cacheGroup}.test`
            is_match &= match &cache_group.test {
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

            temp.push(is_match);
          }
          let mut chunk_key_to_string = HashMap::<ChunksKey, String, ChunksKeyHashBuilder>::default();

          let filtered = plugin
            .cache_groups
            .iter()
            .enumerate()
            .filter(|(index, _)| temp[*index]);

          let mut used_exports_combs = None;
          let mut non_used_exports_combs = None;
          let mut added_keys = FxHashSet::default();

          for (cache_group_index, (idx, cache_group)) in filtered.enumerate() {
            let combs = if cache_group.used_exports {
              if used_exports_combs.is_none() {
                used_exports_combs = Some(combinator.get_combs(
                  module.identifier(),
                  true,
                  module_chunks,
                ));
              }
              used_exports_combs.as_ref().expect("should have used_exports_combs")
            } else {
              if non_used_exports_combs.is_none() {
                non_used_exports_combs = Some(combinator.get_combs(
                  module.identifier(),
                  false,
                  module_chunks,
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
                  module.identifier(),
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
                  module.identifier(),
                  cache_group.key,
                  selected_chunks.len(),
                  cache_group.min_chunks,
                );
                continue;
              }

              let selected_chunks_key =
                get_key(selected_chunks.iter().copied());

              merge_matched_item_into_module_group_map(
                MatchedItem {
                  idx: CacheGroupIdx::new(idx),
                  module,
                  cache_group,
                  cache_group_index,
                  selected_chunks,
                  selected_chunks_key,
                },
                module_group_map,
                &mut chunk_key_to_string,
                compilation,
                module_sizes,
                &mut added_keys,
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

    Ok(module_group_map.into_iter().collect())
  }

  // #[tracing::instrument(skip_all)]
  pub(crate) fn remove_all_modules_from_other_module_groups(
    &self,
    current_module_group: &ModuleGroup,
    module_group_map: &mut ModuleGroupMap,
    used_chunks: &UkeySet<ChunkUkey>,
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
          other_module_group.remove_module(module, module_sizes);
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
          compilation
            .chunk_graph
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
        if Self::remove_min_size_violating_modules(key, other_module_group, cache_group, module_sizes)
          || !Self::check_min_size_reduction(&other_module_group.sizes, &cache_group.min_size_reduction, other_module_group.chunks.len()) {
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
}

async fn merge_matched_item_into_module_group_map(
  matched_item: MatchedItem<'_>,
  module_group_map: &DashMap<String, ModuleGroup>,
  chunk_key_to_string: &mut HashMap<ChunksKey, String, ChunksKeyHashBuilder>,
  compilation: &Compilation,
  module_sizes: &ModuleSizes,
  added_keys: &mut FxHashSet<String>,
) -> Result<()> {
  let MatchedItem {
    idx,
    module,
    cache_group_index,
    cache_group,
    selected_chunks,
    selected_chunks_key,
  } = matched_item;

  // `Module`s with the same chunk_name would be merged togother.
  // `Module`s could be in different `ModuleGroup`s.
  let chunk_name = match &cache_group.name {
    ChunkNameGetter::String(name) => Some(name.to_string()),
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
    let selected_chunks_key = match chunk_key_to_string.entry(selected_chunks_key) {
      hash_map::Entry::Occupied(entry) => entry.get().to_string(),
      hash_map::Entry::Vacant(entry) => {
        let key = format!("{selected_chunks_key:x}");
        entry.insert(key.clone());
        key
      }
    };
    let mut key = String::with_capacity(cache_group.key.len() + selected_chunks_key.len());
    key.push_str(&cache_group.key);
    key.push_str(&selected_chunks_key);
    key
  };

  let mut module_group = {
    module_group_map
      .entry(key.clone())
      .or_insert_with(|| ModuleGroup::new(idx, chunk_name.clone(), cache_group_index, cache_group))
  };
  if chunk_name.is_none() || added_keys.insert(key) {
    module_group.add_module(module.identifier(), module_sizes);
  }
  module_group.chunks.extend(selected_chunks.iter().copied());

  Ok(())
}
