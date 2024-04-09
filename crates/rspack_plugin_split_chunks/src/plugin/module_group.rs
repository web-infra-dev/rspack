use std::collections::{hash_map::Entry, HashMap};
use std::hash::{BuildHasherDefault, Hash, Hasher};

use dashmap::DashMap;
use rayon::prelude::*;
use rspack_core::{Chunk, ChunkGraph, ChunkUkey, Compilation, Module, ModuleGraph};
use rspack_error::Result;
use rustc_hash::{FxHashMap, FxHashSet, FxHasher};

use super::ModuleGroupMap;
use crate::module_group::{compare_entries, CacheGroupIdx, ModuleGroup};
use crate::options::cache_group::CacheGroup;
use crate::options::cache_group_test::{CacheGroupTest, CacheGroupTestFnCtx};
use crate::options::chunk_name::{ChunkNameGetter, ChunkNameGetterFnCtx};
use crate::SplitChunksPlugin;

type ChunksKey = u64;

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

  #[tracing::instrument(skip_all)]
  pub(crate) async fn prepare_module_group_map(
    &self,
    compilation: &mut Compilation,
  ) -> Result<ModuleGroupMap> {
    let chunk_db = &compilation.chunk_by_ukey;
    let chunk_group_db = &compilation.chunk_group_by_ukey;
    let module_graph = compilation.get_module_graph();

    /// If a module meets requirements of a `ModuleGroup`. We consider the `Module` and the `CacheGroup`
    /// to be a `MatchedItem`, which are consumed later to calculate `ModuleGroup`.
    struct MatchedItem<'a> {
      idx: CacheGroupIdx,
      module: &'a dyn Module,
      cache_group_index: usize,
      cache_group: &'a CacheGroup,
      selected_chunks: Box<[&'a Chunk]>,
      selected_chunks_key: ChunksKey,
    }

    let module_group_map: DashMap<String, ModuleGroup> = DashMap::default();

    // chunk_sets_in_graph: key: module, value: multiple chunks contains the module
    // single_chunk_sets: chunkset of module that belongs to only one chunk
    // chunk_sets_by_count: use chunkset len as key
    let (chunk_sets_in_graph, chunk_sets_by_count) =
      { Self::prepare_combination_maps(&module_graph, &compilation.chunk_graph) };

    let combinations_cache =
      DashMap::<ChunksKey, Vec<FxHashSet<ChunkUkey>>, ChunksKeyHashBuilder>::default();

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

      combinations_cache.insert(chunks_key, result.clone());
      result
    };

    module_graph.modules().values().par_bridge().map(|module| {
      let module = &***module;

      let belong_to_chunks = compilation
          .chunk_graph
          .get_module_chunks(module.identifier());

      let chunks_key = Self::get_key(belong_to_chunks.iter());

      let mut temp = vec![];

      for idx in 0..self.cache_groups.len() {
        let cache_group = &self.cache_groups[idx];
        // Filter by `splitChunks.cacheGroups.{cacheGroup}.test`
        let is_match_the_test = match &cache_group.test {
          CacheGroupTest::String(str) => module
            .name_for_condition()
            .map_or(false, |name| name.starts_with(str)),
          CacheGroupTest::RegExp(regexp) => module
            .name_for_condition()
            .map_or(false, |name| regexp.test(&name)),
          CacheGroupTest::Fn(f) => {
            let ctx = CacheGroupTestFnCtx { module };
            f(ctx).unwrap_or_default()
          }
          CacheGroupTest::Enabled => true,
        };
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

        temp.push((idx, is_match));
      }
      let mut chunk_key_to_string = HashMap::<ChunksKey, String, ChunksKeyHashBuilder>::default();
      temp.sort_by(|a, b| a.0.cmp(&b.0));

      let filtered = self
        .cache_groups
        .iter()
        .enumerate()
        .filter(|(index, _)| temp[*index].1);

      for (cache_group_index, (idx, cache_group)) in filtered.enumerate() {
        let combs = get_combination(chunks_key);

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
              .map(|c| {
                chunk_db.expect_get(c)
              })
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

          let selected_chunks_key =
            { Self::get_key(selected_chunks.iter().map(|chunk| &chunk.ukey)) };

          merge_matched_item_into_module_group_map(
            MatchedItem {
              idx: CacheGroupIdx::new(idx),
              module,
              cache_group,
              cache_group_index,
              selected_chunks,
              selected_chunks_key,
            },
            &module_group_map,
            &mut chunk_key_to_string
          )?;

          fn merge_matched_item_into_module_group_map(
            matched_item: MatchedItem<'_>,
            module_group_map: &DashMap<String, ModuleGroup>,
            chunk_key_to_string: &mut HashMap::<ChunksKey, String, ChunksKeyHashBuilder>
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
                let ctx = ChunkNameGetterFnCtx { module };
                f(ctx)?
              }
            };
            let key: String = if let Some(cache_group_name) = &chunk_name {
              let mut key =
                String::with_capacity(cache_group.key.len() + " name:".len() + cache_group_name.len());
              key.push_str(&cache_group.key);
              key.push_str(" name:");
              key.push_str(cache_group_name);
              key
            } else {
              let selected_chunks_key = match chunk_key_to_string.entry(selected_chunks_key) {
                Entry::Occupied(entry) => {
                  entry.get().to_string()
                },
                Entry::Vacant(entry) => {
                  let key = format!("{:x}", selected_chunks_key);
                  entry.insert(key.clone());
                  key
                },
              };
              let mut key =
                String::with_capacity(cache_group.key.len() + " chunks:".len() + selected_chunks_key.len());
              key.push_str(&cache_group.key);
              key.push_str(" chunks:");
              key.push_str(&selected_chunks_key);
              key
            };

            let mut module_group = {
              module_group_map.entry(key).or_insert_with(|| {
                ModuleGroup::new(idx, chunk_name, cache_group_index, cache_group)
              })
            };

            module_group.add_module(module);
            module_group
              .chunks
              .extend(selected_chunks.iter().map(|c| c.ukey));
            Ok(())
          }
        }
      }
      Ok(())
    }).collect::<Result<()>>()?;
    Ok(module_group_map.into_iter().collect())
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
    let module_graph = compilation.get_module_graph();
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
            let module = module_graph
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

  fn get_key<'a, I: Iterator<Item = &'a ChunkUkey>>(chunks: I) -> ChunksKey {
    let mut sorted_chunk_ukeys = chunks.map(|chunk| chunk.as_usize()).collect::<Vec<_>>();
    sorted_chunk_ukeys.sort_unstable();
    let mut hasher = FxHasher::default();
    for chunk_ukey in sorted_chunk_ukeys {
      chunk_ukey.hash(&mut hasher);
    }
    hasher.finish()
  }

  #[allow(clippy::type_complexity)]
  fn prepare_combination_maps(
    module_graph: &ModuleGraph,
    chunk_graph: &ChunkGraph,
  ) -> (
    HashMap<ChunksKey, FxHashSet<ChunkUkey>, ChunksKeyHashBuilder>,
    FxHashMap<usize, Vec<FxHashSet<ChunkUkey>>>,
  ) {
    let mut chunk_sets_in_graph =
      HashMap::<ChunksKey, FxHashSet<ChunkUkey>, ChunksKeyHashBuilder>::default();

    for module in module_graph.modules().keys() {
      let chunks = chunk_graph.get_module_chunks(*module);
      let chunk_key = Self::get_key(chunks.iter());
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
