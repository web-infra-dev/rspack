use std::{
  collections::VecDeque,
  hash::{Hash, Hasher},
  ops::Deref,
  sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
  },
};

use futures::{
  StreamExt,
  channel::{mpsc, oneshot},
  future::join_all,
};
use rayon::prelude::*;
use rspack_core::{
  ChunkByUkey, ChunkUkey, Compilation, ExportsInfoArtifact, Module, ModuleIdentifier,
  RuntimeKeyMap, UsageKey, get_runtime_key,
};
use rspack_error::{Result, ToStringResultToRspackResultExt};
use rspack_util::{fx_hash::FxDashMap, tracing_preset::TRACING_BENCH_TARGET};
use rustc_hash::{FxHashMap, FxHashSet, FxHasher};
use tracing::instrument;

use super::ModuleGroupMap;
use crate::{
  SplitChunksPlugin,
  common::{ChunkFilter, ModuleChunkMap, ModuleChunks, ModuleLayerFilter, ModuleSizes},
  min_size::remove_min_size_violating_modules,
  module_group::{IndexedCacheGroup, ModuleGroup, ModuleGroupKey, compare_entries},
  options::{
    cache_group::CacheGroup,
    cache_group_test::{CacheGroupTest, CacheGroupTestFnCtx},
    chunk_name::{ChunkNameGetter, ChunkNameGetterFnCtx},
  },
};

type ChunksKey = u64;

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

enum SelectedChunks<'a> {
  All(&'a ChunkCombination),
  Owned(ChunkCombination),
  Filtered(Vec<ChunkUkey>),
}

enum SelectedChunksIter<'a> {
  All(std::collections::hash_set::Iter<'a, ChunkUkey>),
  Filtered(std::slice::Iter<'a, ChunkUkey>),
}

impl<'a> Iterator for SelectedChunksIter<'a> {
  type Item = &'a ChunkUkey;

  fn next(&mut self) -> Option<Self::Item> {
    match self {
      Self::All(iter) => iter.next(),
      Self::Filtered(iter) => iter.next(),
    }
  }

  fn size_hint(&self) -> (usize, Option<usize>) {
    match self {
      Self::All(iter) => iter.size_hint(),
      Self::Filtered(iter) => iter.size_hint(),
    }
  }
}

impl SelectedChunks<'_> {
  fn len(&self) -> usize {
    match self {
      Self::All(chunks) => chunks.len(),
      Self::Owned(chunks) => chunks.len(),
      Self::Filtered(chunks) => chunks.len(),
    }
  }

  fn iter(&self) -> SelectedChunksIter<'_> {
    match self {
      Self::All(chunks) => SelectedChunksIter::All(chunks.iter()),
      Self::Owned(chunks) => SelectedChunksIter::All(chunks.iter()),
      Self::Filtered(chunks) => SelectedChunksIter::Filtered(chunks.iter()),
    }
  }

  fn key(&self) -> Option<ChunksKey> {
    match self {
      Self::All(chunks) => Some(chunks.key),
      Self::Owned(chunks) => Some(chunks.key),
      Self::Filtered(_) => None,
    }
  }
}

/// If a module meets requirements of a `ModuleGroup`. We consider the `Module` and the `CacheGroup`
/// to be a `MatchedItem`, which are consumed later to calculate `ModuleGroup`.
struct MatchedItem<'a> {
  module: &'a dyn Module,
  cache_group_index: u32,
  cache_group: &'a CacheGroup,
  selected_chunks: SelectedChunks<'a>,
}

struct PendingMatchedItem {
  module: ModuleIdentifier,
  cache_group_position: usize,
  chunk_combination: ChunkCombination,
}

struct PendingModuleItems {
  order: usize,
  items: VecDeque<PendingMatchedItem>,
}

struct ProcessedMatchedItem {
  module: ModuleIdentifier,
  cache_group_position: usize,
  selected_chunks: SelectedChunks<'static>,
  chunk_name: Option<String>,
}

struct PendingNameRequest {
  module: ModuleIdentifier,
  chunks: Vec<ChunkUkey>,
  cache_group_position: usize,
  response: oneshot::Sender<Option<String>>,
}

// Keep each N-API payload bounded while amortizing the fixed cost of crossing into JavaScript.
// Module filters carry fewer live wrappers than chunk/name callbacks, so they can use larger batches.
// Queue a bounded wave of batches together to avoid a Rust/JS round trip between adjacent batches.
const JS_MODULE_FILTER_BATCH_SIZE: usize = 512;
const JS_CHUNK_FILTER_BATCH_SIZE: usize = 128;
const JS_CHUNK_NAME_BATCH_SIZE: usize = 128;
const JS_MAX_CONCURRENT_BATCHES: usize = 32;

fn ensure_batch_result_len<T>(kind: &str, expected: usize, results: Vec<T>) -> Result<Vec<T>> {
  if results.len() != expected {
    return Err(rspack_error::error!(
      "splitChunks {kind} callback returned {} results for a batch of {expected} items",
      results.len()
    ));
  }
  Ok(results)
}

async fn process_name_requests(
  mut receiver: mpsc::UnboundedReceiver<PendingNameRequest>,
  cache_groups: &[IndexedCacheGroup<'_>],
  compilation: &Compilation,
) -> Result<()> {
  let module_graph = compilation.get_module_graph();

  while let Some(first_request) = receiver.next().await {
    // Bound the queue drain to one payload. Completing this batch immediately applies
    // backpressure at the same callback suspension point as the pre-batch implementation instead
    // of waiting for every module task to reach the callback first.
    let mut requests = Vec::with_capacity(JS_CHUNK_NAME_BATCH_SIZE);
    requests.push(first_request);
    while requests.len() < JS_CHUNK_NAME_BATCH_SIZE
      && let Ok(request) = receiver.try_recv()
    {
      requests.push(request);
    }

    let mut names = std::iter::repeat_with(|| None)
      .take(requests.len())
      .collect::<Vec<Option<Option<String>>>>();
    let mut run_start = 0;
    while run_start < requests.len() {
      let cache_group_position = requests[run_start].cache_group_position;
      let mut run_end = run_start + 1;
      while run_end < requests.len()
        && requests[run_end].cache_group_position == cache_group_position
      {
        run_end += 1;
      }

      let cache_group = cache_groups
        .get(cache_group_position)
        .expect("should have cache group")
        .cache_group;
      let ChunkNameGetter::Fn(get_name) = &cache_group.name else {
        unreachable!("pending name request should use a name callback")
      };

      for wave_start in
        (run_start..run_end).step_by(JS_CHUNK_NAME_BATCH_SIZE * JS_MAX_CONCURRENT_BATCHES)
      {
        let wave_end =
          (wave_start + JS_CHUNK_NAME_BATCH_SIZE * JS_MAX_CONCURRENT_BATCHES).min(run_end);
        let wave = &requests[wave_start..wave_end];
        let batch_results = join_all(wave.chunks(JS_CHUNK_NAME_BATCH_SIZE).map(|batch| {
          let contexts = batch
            .iter()
            .map(|request| ChunkNameGetterFnCtx {
              module: module_graph
                .module_by_identifier(&request.module)
                .expect("should have module")
                .as_ref(),
              compilation,
              chunks: &request.chunks,
              cache_group_key: &cache_group.key,
            })
            .collect();
          async move { ensure_batch_result_len("name", batch.len(), get_name(contexts).await?) }
        }))
        .await;

        let mut result_index = wave_start;
        for batch_result in batch_results {
          for name in batch_result? {
            names[result_index] = Some(name);
            result_index += 1;
          }
        }
        debug_assert_eq!(result_index, wave_end);
      }

      run_start = run_end;
    }

    for (request, name) in requests.into_iter().zip(names) {
      // A dropped receiver means its module task already failed; the coordinator can continue
      // serving the remaining requests and report the original task error from the scope.
      let _ = request
        .response
        .send(name.expect("name request should have a result"));
    }
  }

  Ok(())
}

async fn filter_chunk_batches(
  chunk_filter: &ChunkFilter,
  batches: Vec<Vec<(usize, ChunkUkey)>>,
  compilation: &Compilation,
) -> Result<Vec<(usize, ChunkUkey)>> {
  let batch_results = join_all(batches.into_iter().map(|batch| async move {
    let chunks = batch
      .iter()
      .map(|(_, chunk_ukey)| *chunk_ukey)
      .collect::<Vec<_>>();
    let results = ensure_batch_result_len(
      "chunks",
      chunks.len(),
      chunk_filter.test_func_batch(chunks, compilation).await?,
    )?;
    Ok::<_, rspack_error::Error>(
      batch
        .into_iter()
        .zip(results)
        .filter_map(|(item, matched)| matched.then_some(item))
        .collect::<Vec<_>>(),
    )
  }))
  .await;

  let mut matched_chunks = Vec::new();
  for batch_result in batch_results {
    matched_chunks.extend(batch_result?);
  }
  Ok(matched_chunks)
}

fn extend_selected_chunks(
  selected: &mut [Vec<ChunkUkey>],
  matched_chunks: Vec<(usize, ChunkUkey)>,
) {
  for (item_index, chunk_ukey) in matched_chunks {
    selected
      .get_mut(item_index)
      .expect("should have selected chunks item")
      .push(chunk_ukey);
  }
}

async fn process_pending_items(
  item_count: usize,
  pending_by_cache_group: Vec<Vec<(usize, PendingMatchedItem)>>,
  cache_groups: &[IndexedCacheGroup<'_>],
  compilation: &Compilation,
) -> Result<Vec<Option<ProcessedMatchedItem>>> {
  let module_graph = compilation.get_module_graph();
  let mut processed_items = std::iter::repeat_with(|| None)
    .take(item_count)
    .collect::<Vec<Option<ProcessedMatchedItem>>>();

  for (cache_group_position, pending_items) in pending_by_cache_group.into_iter().enumerate() {
    if pending_items.is_empty() {
      continue;
    }
    let IndexedCacheGroup { cache_group, .. } = cache_groups
      .get(cache_group_position)
      .expect("should have cache group");

    let selected_chunks = match &cache_group.chunk_filter {
      ChunkFilter::All => pending_items
        .iter()
        .map(|(_, pending)| SelectedChunks::Owned(pending.chunk_combination.clone()))
        .collect::<Vec<_>>(),
      ChunkFilter::Func(_) => {
        let mut selected = std::iter::repeat_with(Vec::new)
          .take(pending_items.len())
          .collect::<Vec<Vec<ChunkUkey>>>();
        let mut batches = Vec::with_capacity(JS_MAX_CONCURRENT_BATCHES);
        let mut batch = Vec::with_capacity(JS_CHUNK_FILTER_BATCH_SIZE);
        for (item_index, (_, pending)) in pending_items.iter().enumerate() {
          for chunk_ukey in pending.chunk_combination.iter().copied() {
            batch.push((item_index, chunk_ukey));
            if batch.len() == JS_CHUNK_FILTER_BATCH_SIZE {
              batches.push(std::mem::replace(
                &mut batch,
                Vec::with_capacity(JS_CHUNK_FILTER_BATCH_SIZE),
              ));
              if batches.len() == JS_MAX_CONCURRENT_BATCHES {
                extend_selected_chunks(
                  &mut selected,
                  filter_chunk_batches(
                    &cache_group.chunk_filter,
                    std::mem::replace(&mut batches, Vec::with_capacity(JS_MAX_CONCURRENT_BATCHES)),
                    compilation,
                  )
                  .await?,
                );
              }
            }
          }
        }
        if !batch.is_empty() {
          batches.push(batch);
        }
        if !batches.is_empty() {
          extend_selected_chunks(
            &mut selected,
            filter_chunk_batches(&cache_group.chunk_filter, batches, compilation).await?,
          );
        }
        selected.into_iter().map(SelectedChunks::Filtered).collect()
      }
      _ => pending_items
        .iter()
        .map(|(_, pending)| {
          SelectedChunks::Filtered(
            pending
              .chunk_combination
              .iter()
              .filter(|chunk| cache_group.chunk_filter.test_internal(chunk, compilation))
              .copied()
              .collect(),
          )
        })
        .collect(),
    };

    let matched_items = pending_items
      .into_iter()
      .zip(selected_chunks)
      .filter_map(|((replay_index, pending), selected_chunks)| {
        if selected_chunks.len() < cache_group.min_chunks as usize {
          return None;
        }
        Some((replay_index, pending.module, selected_chunks))
      })
      .collect::<Vec<_>>();

    let chunk_names = match &cache_group.name {
      ChunkNameGetter::String(name) => vec![Some(name.clone()); matched_items.len()],
      ChunkNameGetter::Disabled => vec![None; matched_items.len()],
      ChunkNameGetter::Fn(get_name) => {
        let mut names = Vec::with_capacity(matched_items.len());
        for matched_items_wave in
          matched_items.chunks(JS_CHUNK_NAME_BATCH_SIZE * JS_MAX_CONCURRENT_BATCHES)
        {
          let batch_results = join_all(matched_items_wave.chunks(JS_CHUNK_NAME_BATCH_SIZE).map(
            |batch| async move {
              let chunks = batch
                .iter()
                .map(|(_, _, selected_chunks)| selected_chunks.iter().copied().collect())
                .collect::<Vec<Vec<ChunkUkey>>>();
              let contexts = batch
                .iter()
                .zip(&chunks)
                .map(|((_, module_identifier, _), chunks)| ChunkNameGetterFnCtx {
                  module: module_graph
                    .module_by_identifier(module_identifier)
                    .expect("should have module")
                    .as_ref(),
                  compilation,
                  chunks,
                  cache_group_key: &cache_group.key,
                })
                .collect();
              ensure_batch_result_len("name", batch.len(), get_name(contexts).await?)
            },
          ))
          .await;
          for batch_result in batch_results {
            names.extend(batch_result?);
          }
        }
        names
      }
    };

    for ((replay_index, module_identifier, selected_chunks), chunk_name) in
      matched_items.into_iter().zip(chunk_names)
    {
      processed_items[replay_index] = Some(ProcessedMatchedItem {
        module: module_identifier,
        cache_group_position,
        selected_chunks,
        chunk_name,
      });
    }
  }

  Ok(processed_items)
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
  non_used_exports_chunks_keys: Vec<Option<ChunksKey>>,
  grouped_by_exports: Vec<Vec<ChunksKey>>,
}

enum ChunkCombinations<'a> {
  Slice(&'a [ChunkCombination]),
  UsedExports(Vec<&'a ChunkCombination>),
}

enum ChunkCombinationsIter<'a> {
  Slice(std::slice::Iter<'a, ChunkCombination>),
  UsedExports(std::iter::Copied<std::slice::Iter<'a, &'a ChunkCombination>>),
}

impl<'a> Iterator for ChunkCombinationsIter<'a> {
  type Item = &'a ChunkCombination;

  fn next(&mut self) -> Option<Self::Item> {
    match self {
      Self::Slice(iter) => iter.next(),
      Self::UsedExports(iter) => iter.next(),
    }
  }

  fn size_hint(&self) -> (usize, Option<usize>) {
    match self {
      Self::Slice(iter) => iter.size_hint(),
      Self::UsedExports(iter) => iter.size_hint(),
    }
  }
}

impl<'a> IntoIterator for &'a ChunkCombinations<'a> {
  type Item = &'a ChunkCombination;
  type IntoIter = ChunkCombinationsIter<'a>;

  fn into_iter(self) -> Self::IntoIter {
    match self {
      ChunkCombinations::Slice(combs) => ChunkCombinationsIter::Slice(combs.iter()),
      ChunkCombinations::UsedExports(combs) => {
        ChunkCombinationsIter::UsedExports(combs.iter().copied())
      }
    }
  }
}

impl Combinator {
  fn get_non_used_exports_combs(
    &self,
    module_index: usize,
    module_chunks: &ModuleChunks,
    chunk_index_map: &FxHashMap<ChunkUkey, u32>,
  ) -> &[ChunkCombination] {
    let chunks = module_chunks
      .get(module_index)
      .expect("should have module chunks");
    let chunks_key = self
      .non_used_exports_chunks_keys
      .get(module_index)
      .and_then(|key| *key)
      .unwrap_or_else(|| get_key(chunks.iter().copied(), chunk_index_map));
    self
      .combinations
      .get(&chunks_key)
      .expect("should have combinations")
  }

  fn get_used_exports_combs(&self, module_index: usize) -> Vec<&ChunkCombination> {
    let mut result = vec![];
    let chunks_by_module_used = self
      .grouped_by_exports
      .get(module_index)
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
    module_index: usize,
    used_exports: bool,
    module_chunks: &ModuleChunks,
    chunk_index_map: &FxHashMap<ChunkUkey, u32>,
  ) -> ChunkCombinations<'_> {
    if used_exports {
      ChunkCombinations::UsedExports(self.get_used_exports_combs(module_index))
    } else {
      ChunkCombinations::Slice(self.get_non_used_exports_combs(
        module_index,
        module_chunks,
        chunk_index_map,
      ))
    }
  }

  fn get_combinations(
    chunk_sets_in_graph: FxHashMap<ChunksKey, ChunkCombination>,
    chunk_sets_by_count: Vec<ChunkCombination>,
  ) -> FxHashMap<ChunksKey, Vec<ChunkCombination>> {
    chunk_sets_in_graph
      .into_par_iter()
      .map(|(chunks_key, chunks_set)| {
        let mut result = vec![];
        let chunks_set_len = chunks_set.len();
        for set in &chunk_sets_by_count {
          if set.len() >= chunks_set_len {
            break;
          }
          if set.is_subset(&chunks_set) {
            result.push(set.clone());
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
    min_chunks: usize,
  ) {
    self.non_used_exports_chunks_keys = all_modules
      .par_iter()
      .enumerate()
      .map(|(module_index, _)| {
        let chunks = module_chunks
          .get(module_index)
          .expect("should have module chunks");
        if chunks.is_empty() || chunks.len() < min_chunks {
          None
        } else {
          Some(get_key(chunks.iter().copied(), chunk_index_map))
        }
      })
      .collect::<Vec<_>>();

    let mut chunk_sets_in_graph = FxHashMap::with_capacity_and_hasher(
      self.non_used_exports_chunks_keys.len(),
      Default::default(),
    );
    for (module_index, chunk_key) in self
      .non_used_exports_chunks_keys
      .iter()
      .enumerate()
      .filter_map(|(module_index, chunk_key)| chunk_key.map(|chunk_key| (module_index, chunk_key)))
    {
      chunk_sets_in_graph
        .entry(chunk_key)
        .or_insert_with(|| ChunkCombination {
          key: chunk_key,
          chunks: Arc::new(
            module_chunks
              .get(module_index)
              .expect("should have module chunks")
              .clone(),
          ),
        });
    }

    let mut chunk_sets_by_count = Vec::<ChunkCombination>::with_capacity(chunk_sets_in_graph.len());
    for chunks in chunk_sets_in_graph.values() {
      chunk_sets_by_count.push(chunks.clone());
    }

    chunk_sets_by_count.sort_unstable_by_key(|chunks| chunks.len());

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
    let (grouped_by_exports, used_exports_chunks): (Vec<_>, Vec<_>) = all_modules
      .par_iter()
      .enumerate()
      .map(|(module_index, module)| {
        let grouped_chunks = Self::group_chunks_by_exports(
          module,
          module_chunks
            .get(module_index)
            .expect("should have module chunks")
            .iter()
            .copied(),
          exports_info_artifact,
          chunk_by_ukey,
          chunk_index_map,
        );
        let mut grouped_chunks_key = Vec::with_capacity(grouped_chunks.len());
        let mut used_exports_chunks = Vec::with_capacity(grouped_chunks.len());
        for chunks in grouped_chunks {
          if chunks.is_empty() {
            continue;
          }
          grouped_chunks_key.push(chunks.key);
          used_exports_chunks.push(chunks);
        }
        (grouped_chunks_key, used_exports_chunks)
      })
      .unzip();

    self.grouped_by_exports = grouped_by_exports;

    let mut used_exports_chunk_sets_in_graph = FxHashMap::default();
    let mut used_exports_chunk_sets_by_count = Vec::<ChunkCombination>::default();
    for used_exports_chunks in used_exports_chunks {
      for chunks in used_exports_chunks {
        let chunk_key = chunks.key;
        if let std::collections::hash_map::Entry::Vacant(entry) =
          used_exports_chunk_sets_in_graph.entry(chunk_key)
        {
          used_exports_chunk_sets_by_count.push(chunks.clone());
          entry.insert(chunks);
        }
      }
    }

    used_exports_chunk_sets_by_count.sort_unstable_by_key(|chunks| chunks.len());

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

    let mut best_entry_index = 0;
    for entry_index in 1..module_group_map.len() {
      let [(entry_key, entry), (best_entry_key, best_entry)] = module_group_map
        .get_disjoint_indices_mut([entry_index, best_entry_index])
        .expect("entry indices should be valid and unique");
      let result = compare_entries((entry_key, entry), (best_entry_key, best_entry));
      if result > 0f64 {
        best_entry_index = entry_index;
      }
    }

    module_group_map
      .swap_remove_index(best_entry_index)
      .expect("This should never happen, please file an issue")
  }

  #[allow(clippy::too_many_arguments)]
  #[instrument(name = "Compilation:SplitChunks:prepare_module_group_map",target=TRACING_BENCH_TARGET, skip_all)]
  pub(crate) async fn prepare_module_group_map(
    &self,
    combinator: &Combinator,
    all_modules: &[ModuleIdentifier],
    cache_groups: Vec<IndexedCacheGroup<'_>>,
    compilation: &Compilation,
    module_chunks: &ModuleChunks,
    chunk_index_map: &FxHashMap<ChunkUkey, u32>,
  ) -> Result<ModuleGroupMap> {
    if cache_groups
      .iter()
      .all(|cache_group| !cache_group.cache_group.has_js_callback())
    {
      self
        .prepare_module_group_map_native(
          combinator,
          all_modules,
          cache_groups,
          compilation,
          module_chunks,
          chunk_index_map,
        )
        .await
    } else if cache_groups.iter().all(|indexed_cache_group| {
      let cache_group = indexed_cache_group.cache_group;
      !cache_group.layer.is_func()
        && !matches!(cache_group.test, CacheGroupTest::Fn(_))
        && !cache_group.chunk_filter.is_func()
    }) {
      self
        .prepare_module_group_map_name_batched(
          combinator,
          all_modules,
          cache_groups,
          compilation,
          module_chunks,
          chunk_index_map,
        )
        .await
    } else {
      self
        .prepare_module_group_map_batched(
          combinator,
          all_modules,
          cache_groups,
          compilation,
          module_chunks,
          chunk_index_map,
        )
        .await
    }
  }

  #[allow(clippy::too_many_arguments)]
  async fn prepare_module_group_map_name_batched(
    &self,
    combinator: &Combinator,
    all_modules: &[ModuleIdentifier],
    cache_groups: Vec<IndexedCacheGroup<'_>>,
    compilation: &Compilation,
    module_chunks: &ModuleChunks,
    chunk_index_map: &FxHashMap<ChunkUkey, u32>,
  ) -> Result<ModuleGroupMap> {
    debug_assert!(cache_groups.iter().any(|indexed_cache_group| matches!(
      indexed_cache_group.cache_group.name,
      ChunkNameGetter::Fn(_)
    )));
    debug_assert!(cache_groups.iter().all(|indexed_cache_group| {
      let cache_group = indexed_cache_group.cache_group;
      !cache_group.layer.is_func()
        && !matches!(cache_group.test, CacheGroupTest::Fn(_))
        && !cache_group.chunk_filter.is_func()
    }));

    let module_graph = compilation.get_module_graph();
    let module_group_map: FxDashMap<ModuleGroupKey, ModuleGroup> = FxDashMap::default();
    let (name_sender, name_receiver) = mpsc::unbounded();

    let module_group_results = rspack_parallel::scope::<_, Result<_>>(|token| {
      let coordinator = unsafe { token.used((name_receiver, &cache_groups, compilation)) };
      coordinator.spawn(|(name_receiver, cache_groups, compilation)| async move {
        process_name_requests(name_receiver, cache_groups, compilation).await
      });

      all_modules
        .iter()
        .enumerate()
        .for_each(|(module_index, module_identifier)| {
          let name_sender = name_sender.clone();
          let s = unsafe {
            token.used((
              &cache_groups,
              module_index,
              module_identifier,
              &module_graph,
              compilation,
              &module_group_map,
              combinator,
              module_chunks,
              chunk_index_map,
              name_sender,
            ))
          };
          s.spawn(
            |(
              cache_groups,
              module_index,
              module_identifier,
              module_graph,
              compilation,
              module_group_map,
              combinator,
              module_chunks,
              chunk_index_map,
              name_sender,
            )| async move {
              let belong_to_chunks = module_chunks
                .get(module_index)
                .expect("should have module chunks");
              if belong_to_chunks.is_empty() {
                return Ok(());
              }

              let module = module_graph
                .module_by_identifier(module_identifier)
                .expect("should have module")
                .as_ref();
              let mut used_exports_combinations = None;
              let mut non_used_exports_combinations = None;

              for (cache_group_position, indexed_cache_group) in cache_groups.iter().enumerate() {
                let cache_group = indexed_cache_group.cache_group;
                if !(cache_group.r#type)(module)
                  || !cache_group
                    .layer
                    .test_internal(module.get_layer().map(String::as_str))
                {
                  continue;
                }

                let is_match = match &cache_group.test {
                  CacheGroupTest::String(test) => module
                    .name_for_condition()
                    .is_some_and(|name| name.starts_with(test)),
                  CacheGroupTest::RegExp(test) => module
                    .name_for_condition()
                    .is_some_and(|name| test.test(&name)),
                  CacheGroupTest::Enabled => true,
                  CacheGroupTest::Fn(_) => {
                    unreachable!("name-only batching should not contain a test callback")
                  }
                };
                if !is_match || belong_to_chunks.len() < cache_group.min_chunks as usize {
                  continue;
                }

                let combinations = if cache_group.used_exports {
                  if used_exports_combinations.is_none() {
                    used_exports_combinations = Some(combinator.get_combs(
                      module_index,
                      true,
                      module_chunks,
                      chunk_index_map,
                    ));
                  }
                  used_exports_combinations
                    .as_ref()
                    .expect("should have used exports combinations")
                } else {
                  if non_used_exports_combinations.is_none() {
                    non_used_exports_combinations = Some(combinator.get_combs(
                      module_index,
                      false,
                      module_chunks,
                      chunk_index_map,
                    ));
                  }
                  non_used_exports_combinations
                    .as_ref()
                    .expect("should have non-used exports combinations")
                };

                for chunk_combination in combinations {
                  if chunk_combination.is_empty()
                    || chunk_combination.len() < cache_group.min_chunks as usize
                  {
                    continue;
                  }

                  if matches!(&cache_group.chunk_filter, ChunkFilter::All)
                    && matches!(&cache_group.name, ChunkNameGetter::Disabled)
                  {
                    let mut module_group = module_group_map
                      .entry(ModuleGroupKey::Anonymous {
                        cache_group_index: indexed_cache_group.cache_group_index,
                        chunks_key: chunk_combination.key,
                      })
                      .or_insert_with(|| {
                        ModuleGroup::new(None, indexed_cache_group.cache_group_index, cache_group)
                      });
                    module_group.add_module_with_shared_chunks(
                      module.identifier(),
                      chunk_combination.iter().copied(),
                    );
                    continue;
                  }

                  let selected_chunks = match &cache_group.chunk_filter {
                    ChunkFilter::All => SelectedChunks::All(chunk_combination),
                    ChunkFilter::Func(_) => {
                      unreachable!("name-only batching should not contain a chunks callback")
                    }
                    _ => SelectedChunks::Filtered(
                      chunk_combination
                        .iter()
                        .filter(|chunk| cache_group.chunk_filter.test_internal(chunk, compilation))
                        .copied()
                        .collect(),
                    ),
                  };

                  if selected_chunks.len() < cache_group.min_chunks as usize {
                    continue;
                  }

                  let chunk_name = match &cache_group.name {
                    ChunkNameGetter::String(name) => Some(name.clone()),
                    ChunkNameGetter::Disabled => None,
                    ChunkNameGetter::Fn(_) => {
                      let (response, response_receiver) = oneshot::channel();
                      name_sender
                        .unbounded_send(PendingNameRequest {
                          module: module.identifier(),
                          chunks: selected_chunks.iter().copied().collect(),
                          cache_group_position,
                          response,
                        })
                        .map_err(|_| {
                          rspack_error::error!("splitChunks name batch coordinator stopped")
                        })?;
                      response_receiver.await.map_err(|_| {
                        rspack_error::error!("splitChunks name batch response was canceled")
                      })?
                    }
                  };

                  merge_matched_item_into_module_group_map(
                    MatchedItem {
                      module,
                      cache_group,
                      cache_group_index: indexed_cache_group.cache_group_index,
                      selected_chunks,
                    },
                    chunk_name,
                    module_group_map,
                    chunk_index_map,
                  );
                }
              }
              Ok(())
            },
          );
        });

      drop(name_sender);
    })
    .await
    .into_iter()
    .map(|result| result.to_rspack_result())
    .collect::<Result<Vec<_>>>()?;

    for result in module_group_results {
      result?;
    }

    let module_group_count = module_group_map.len();
    let mut result = Vec::with_capacity(module_group_count);
    result.extend(module_group_map);
    result.sort_by(|a, b| a.0.cmp(&b.0));
    let mut ordered_result =
      ModuleGroupMap::with_capacity_and_hasher(module_group_count, Default::default());
    ordered_result.extend(result);
    Ok(ordered_result)
  }

  #[allow(clippy::too_many_arguments)]
  async fn prepare_module_group_map_batched(
    &self,
    combinator: &Combinator,
    all_modules: &[ModuleIdentifier],
    cache_groups: Vec<IndexedCacheGroup<'_>>,
    compilation: &Compilation,
    module_chunks: &ModuleChunks,
    chunk_index_map: &FxHashMap<ChunkUkey, u32>,
  ) -> Result<ModuleGroupMap> {
    let module_graph = compilation.get_module_graph();
    let mut module_group_map = ModuleGroupMap::default();
    let mut matched_cache_groups_by_module = vec![Vec::<usize>::new(); all_modules.len()];
    let has_module_filter_callbacks = cache_groups.iter().any(|indexed_cache_group| {
      indexed_cache_group.cache_group.layer.is_func()
        || matches!(indexed_cache_group.cache_group.test, CacheGroupTest::Fn(_))
    });

    if has_module_filter_callbacks {
      for (cache_group_position, indexed_cache_group) in cache_groups.iter().enumerate() {
        let cache_group = indexed_cache_group.cache_group;
        let typed_module_indices = all_modules
          .par_iter()
          .enumerate()
          .filter_map(|(module_index, module_identifier)| {
            let belong_to_chunks = module_chunks
              .get(module_index)
              .expect("should have module chunks");
            if belong_to_chunks.is_empty() {
              return None;
            }
            let module = module_graph
              .module_by_identifier(module_identifier)
              .expect("should have module")
              .as_ref();
            (cache_group.r#type)(module).then_some(module_index)
          })
          .collect::<Vec<_>>();

        let layered_module_indices = match &cache_group.layer {
          ModuleLayerFilter::Func(_) => {
            let mut layered_module_indices = Vec::with_capacity(typed_module_indices.len());
            for module_wave in
              typed_module_indices.chunks(JS_MODULE_FILTER_BATCH_SIZE * JS_MAX_CONCURRENT_BATCHES)
            {
              let batch_results = join_all(module_wave.chunks(JS_MODULE_FILTER_BATCH_SIZE).map(
                |module_indices| async move {
                  let layers = module_indices
                    .iter()
                    .map(|module_index| {
                      let module_identifier = all_modules
                        .get(*module_index)
                        .expect("should have module identifier");
                      module_graph
                        .module_by_identifier(module_identifier)
                        .expect("should have module")
                        .get_layer()
                        .map(ToString::to_string)
                    })
                    .collect();
                  let layer_results = ensure_batch_result_len(
                    "layer",
                    module_indices.len(),
                    cache_group.layer.test_func_batch(layers).await?,
                  )?;
                  Ok::<_, rspack_error::Error>(
                    module_indices
                      .iter()
                      .zip(layer_results)
                      .filter_map(|(module_index, matched)| matched.then_some(*module_index))
                      .collect::<Vec<_>>(),
                  )
                },
              ))
              .await;
              for batch_result in batch_results {
                layered_module_indices.extend(batch_result?);
              }
            }
            layered_module_indices
          }
          _ => typed_module_indices
            .into_par_iter()
            .filter(|module_index| {
              let module_identifier = all_modules
                .get(*module_index)
                .expect("should have module identifier");
              let module = module_graph
                .module_by_identifier(module_identifier)
                .expect("should have module");
              cache_group
                .layer
                .test_internal(module.get_layer().map(String::as_str))
            })
            .collect(),
        };

        let mut tested_module_indices = Vec::with_capacity(layered_module_indices.len());
        match &cache_group.test {
          CacheGroupTest::String(test) => {
            tested_module_indices.extend(
              layered_module_indices
                .par_iter()
                .copied()
                .filter(|module_index| {
                  let module_identifier = all_modules
                    .get(*module_index)
                    .expect("should have module identifier");
                  module_graph
                    .module_by_identifier(module_identifier)
                    .expect("should have module")
                    .name_for_condition()
                    .is_some_and(|name| name.starts_with(test))
                })
                .collect::<Vec<_>>(),
            );
          }
          CacheGroupTest::RegExp(test) => {
            tested_module_indices.extend(
              layered_module_indices
                .par_iter()
                .copied()
                .filter(|module_index| {
                  let module_identifier = all_modules
                    .get(*module_index)
                    .expect("should have module identifier");
                  module_graph
                    .module_by_identifier(module_identifier)
                    .expect("should have module")
                    .name_for_condition()
                    .is_some_and(|name| test.test(&name))
                })
                .collect::<Vec<_>>(),
            );
          }
          CacheGroupTest::Fn(test) => {
            for module_wave in
              layered_module_indices.chunks(JS_MODULE_FILTER_BATCH_SIZE * JS_MAX_CONCURRENT_BATCHES)
            {
              let batch_results = join_all(module_wave.chunks(JS_MODULE_FILTER_BATCH_SIZE).map(
                |module_indices| async move {
                  let contexts = module_indices
                    .iter()
                    .map(|module_index| {
                      let module_identifier = all_modules
                        .get(*module_index)
                        .expect("should have module identifier");
                      let module = module_graph
                        .module_by_identifier(module_identifier)
                        .expect("should have module")
                        .as_ref();
                      CacheGroupTestFnCtx {
                        compilation,
                        module,
                      }
                    })
                    .collect();
                  let test_results =
                    ensure_batch_result_len("test", module_indices.len(), test(contexts).await?)?;
                  Ok::<_, rspack_error::Error>(
                    module_indices
                      .iter()
                      .zip(test_results)
                      .filter_map(|(module_index, matched)| {
                        matched.unwrap_or_default().then_some(*module_index)
                      })
                      .collect::<Vec<_>>(),
                  )
                },
              ))
              .await;
              for batch_result in batch_results {
                tested_module_indices.extend(batch_result?);
              }
            }
          }
          CacheGroupTest::Enabled => tested_module_indices.extend(layered_module_indices),
        }

        for module_index in tested_module_indices {
          let belong_to_chunks = module_chunks
            .get(module_index)
            .expect("should have module chunks");
          if belong_to_chunks.len() >= cache_group.min_chunks as usize {
            matched_cache_groups_by_module
              .get_mut(module_index)
              .expect("should have matched cache groups")
              .push(cache_group_position);
          }
        }
      }
    }

    // The pre-batch implementation let each module task advance independently. Capture the order
    // in which tasks first reach an eligible chunk combination so replaying the batched results
    // retains that interleaving instead of grouping every mutation by cache group.
    let pending_order = AtomicUsize::new(0);
    let pending_results = rspack_parallel::scope::<_, Result<_>>(|token| {
      all_modules
        .iter()
        .enumerate()
        .for_each(|(module_index, module_identifier)| {
          let s = unsafe {
            token.used((
              &cache_groups,
              &matched_cache_groups_by_module,
              has_module_filter_callbacks,
              module_index,
              module_identifier,
              &module_graph,
              &pending_order,
              combinator,
              module_chunks,
              chunk_index_map,
            ))
          };
          s.spawn(
            |(
              cache_groups,
              matched_cache_groups_by_module,
              has_module_filter_callbacks,
              module_index,
              module_identifier,
              module_graph,
              pending_order,
              combinator,
              module_chunks,
              chunk_index_map,
            )| async move {
              let belong_to_chunks = module_chunks
                .get(module_index)
                .expect("should have module chunks");
              if belong_to_chunks.is_empty() {
                return Ok(None);
              }

              let matched_cache_groups = matched_cache_groups_by_module
                .get(module_index)
                .expect("should have matched cache groups");
              if has_module_filter_callbacks && matched_cache_groups.is_empty() {
                return Ok(None);
              }

              let module = module_graph
                .module_by_identifier(module_identifier)
                .expect("should have module")
                .as_ref();
              let mut used_exports_combs = None;
              let mut non_used_exports_combs = None;
              let mut pending = Vec::new();
              let mut order = None;

              let mut matched_cache_group_cursor = 0;
              for (cache_group_position, indexed_cache_group) in cache_groups.iter().enumerate() {
                if has_module_filter_callbacks {
                  if matched_cache_groups
                    .get(matched_cache_group_cursor)
                    .copied()
                    != Some(cache_group_position)
                  {
                    continue;
                  }
                  matched_cache_group_cursor += 1;
                }

                let IndexedCacheGroup { cache_group, .. } = indexed_cache_group;
                if !has_module_filter_callbacks {
                  if !(cache_group.r#type)(module)
                    || !cache_group
                      .layer
                      .test_internal(module.get_layer().map(String::as_str))
                  {
                    continue;
                  }

                  let is_match = match &cache_group.test {
                    CacheGroupTest::String(test) => module
                      .name_for_condition()
                      .is_some_and(|name| name.starts_with(test)),
                    CacheGroupTest::RegExp(test) => module
                      .name_for_condition()
                      .is_some_and(|name| test.test(&name)),
                    CacheGroupTest::Enabled => true,
                    CacheGroupTest::Fn(_) => {
                      unreachable!("module filter callback should have been precomputed")
                    }
                  };
                  if !is_match || belong_to_chunks.len() < cache_group.min_chunks as usize {
                    continue;
                  }
                }

                let combinations = if cache_group.used_exports {
                  if used_exports_combs.is_none() {
                    used_exports_combs = Some(combinator.get_combs(
                      module_index,
                      true,
                      module_chunks,
                      chunk_index_map,
                    ));
                  }
                  used_exports_combs
                    .as_ref()
                    .expect("should have used exports combinations")
                } else {
                  if non_used_exports_combs.is_none() {
                    non_used_exports_combs = Some(combinator.get_combs(
                      module_index,
                      false,
                      module_chunks,
                      chunk_index_map,
                    ));
                  }
                  non_used_exports_combs
                    .as_ref()
                    .expect("should have non-used exports combinations")
                };

                for chunk_combination in combinations {
                  if chunk_combination.is_empty()
                    || chunk_combination.len() < cache_group.min_chunks as usize
                  {
                    continue;
                  }

                  order.get_or_insert_with(|| pending_order.fetch_add(1, Ordering::Relaxed));
                  pending.push(PendingMatchedItem {
                    module: module.identifier(),
                    cache_group_position,
                    chunk_combination: chunk_combination.clone(),
                  });
                }
              }
              Ok(order.map(|order| PendingModuleItems {
                order,
                items: pending.into(),
              }))
            },
          );
        });
    })
    .await
    .into_iter()
    .map(|result| result.to_rspack_result())
    .collect::<Result<Vec<_>>>()?;

    let mut pending_by_module = Vec::new();
    for result in pending_results {
      if let Some(pending) = result? {
        pending_by_module.push(pending);
      }
    }
    pending_by_module.sort_unstable_by_key(|pending| pending.order);

    // Assign a replay position by advancing every module to at most one chunks/name callback per
    // round. Callback work can then be fully grouped by cache group without changing the order in
    // which completed items mutate ModuleGroups.
    let mut pending_by_cache_group = std::iter::repeat_with(Vec::new)
      .take(cache_groups.len())
      .collect::<Vec<Vec<(usize, PendingMatchedItem)>>>();
    let mut item_count = 0;
    while !pending_by_module.is_empty() {
      let mut next_pending_by_module = Vec::with_capacity(pending_by_module.len());
      for mut pending_module in pending_by_module {
        loop {
          let pending = pending_module
            .items
            .pop_front()
            .expect("pending module should contain an item");
          let cache_group = cache_groups
            .get(pending.cache_group_position)
            .expect("should have cache group")
            .cache_group;
          let yields_to_callback = matches!(&cache_group.chunk_filter, ChunkFilter::Func(_))
            || matches!(&cache_group.name, ChunkNameGetter::Fn(_));
          pending_by_cache_group
            .get_mut(pending.cache_group_position)
            .expect("should have pending cache group")
            .push((item_count, pending));
          item_count += 1;
          if yields_to_callback || pending_module.items.is_empty() {
            break;
          }
        }
        if !pending_module.items.is_empty() {
          next_pending_by_module.push(pending_module);
        }
      }
      pending_by_module = next_pending_by_module;
    }

    let processed_items = process_pending_items(
      item_count,
      pending_by_cache_group,
      &cache_groups,
      compilation,
    )
    .await?;

    for processed in processed_items.into_iter().flatten() {
      let IndexedCacheGroup {
        cache_group_index,
        cache_group,
      } = cache_groups
        .get(processed.cache_group_position)
        .expect("should have cache group");
      merge_matched_item_into_ordered_module_group_map(
        MatchedItem {
          module: module_graph
            .module_by_identifier(&processed.module)
            .expect("should have module")
            .as_ref(),
          cache_group,
          cache_group_index: *cache_group_index,
          selected_chunks: processed.selected_chunks,
        },
        processed.chunk_name,
        &mut module_group_map,
        chunk_index_map,
      );
    }

    // Sort the module_group_map by key to ensure deterministic iteration order
    let module_group_count = module_group_map.len();
    let mut result = Vec::with_capacity(module_group_count);
    result.extend(module_group_map);
    result.sort_by(|a, b| a.0.cmp(&b.0));
    let mut ordered_result =
      ModuleGroupMap::with_capacity_and_hasher(module_group_count, Default::default());
    ordered_result.extend(result);
    Ok(ordered_result)
  }

  #[allow(clippy::too_many_arguments)]
  async fn prepare_module_group_map_native(
    &self,
    combinator: &Combinator,
    all_modules: &[ModuleIdentifier],
    cache_groups: Vec<IndexedCacheGroup<'_>>,
    compilation: &Compilation,
    module_chunks: &ModuleChunks,
    chunk_index_map: &FxHashMap<ChunkUkey, u32>,
  ) -> Result<ModuleGroupMap> {
    debug_assert!(
      cache_groups
        .iter()
        .all(|cache_group| !cache_group.cache_group.has_js_callback())
    );

    let module_graph = compilation.get_module_graph();
    let module_group_map: FxDashMap<ModuleGroupKey, ModuleGroup> = FxDashMap::default();
    let module_group_results = rspack_parallel::scope::<_, Result<_>>(|token| {
      all_modules
        .iter()
        .enumerate()
        .for_each(|(module_index, module_identifier)| {
          let s = unsafe {
            token.used((
              &cache_groups,
              module_index,
              module_identifier,
              &module_graph,
              compilation,
              &module_group_map,
              combinator,
              module_chunks,
              chunk_index_map,
            ))
          };
          s.spawn(
            |(
              cache_groups,
              module_index,
              module_identifier,
              module_graph,
              compilation,
              module_group_map,
              combinator,
              module_chunks,
              chunk_index_map,
            )| async move {
              let belong_to_chunks = module_chunks
                .get(module_index)
                .expect("should have module chunks");
              if belong_to_chunks.is_empty() {
                return Ok(());
              }

              let module = module_graph
                .module_by_identifier(module_identifier)
                .expect("should have module")
                .as_ref();
              let mut used_exports_combinations = None;
              let mut non_used_exports_combinations = None;

              for indexed_cache_group in cache_groups {
                let cache_group = indexed_cache_group.cache_group;
                if !(cache_group.r#type)(module)
                  || !cache_group
                    .layer
                    .test_internal(module.get_layer().map(String::as_str))
                {
                  continue;
                }

                let is_match = match &cache_group.test {
                  CacheGroupTest::String(test) => module
                    .name_for_condition()
                    .is_some_and(|name| name.starts_with(test)),
                  CacheGroupTest::RegExp(test) => module
                    .name_for_condition()
                    .is_some_and(|name| test.test(&name)),
                  CacheGroupTest::Enabled => true,
                  CacheGroupTest::Fn(_) => {
                    unreachable!("native cache group should not contain a test function")
                  }
                };
                if !is_match || belong_to_chunks.len() < cache_group.min_chunks as usize {
                  continue;
                }

                let combinations = if cache_group.used_exports {
                  if used_exports_combinations.is_none() {
                    used_exports_combinations = Some(combinator.get_combs(
                      module_index,
                      true,
                      module_chunks,
                      chunk_index_map,
                    ));
                  }
                  used_exports_combinations
                    .as_ref()
                    .expect("should have used exports combinations")
                } else {
                  if non_used_exports_combinations.is_none() {
                    non_used_exports_combinations = Some(combinator.get_combs(
                      module_index,
                      false,
                      module_chunks,
                      chunk_index_map,
                    ));
                  }
                  non_used_exports_combinations
                    .as_ref()
                    .expect("should have non-used exports combinations")
                };

                for chunk_combination in combinations {
                  if chunk_combination.is_empty()
                    || chunk_combination.len() < cache_group.min_chunks as usize
                  {
                    continue;
                  }

                  if matches!(&cache_group.chunk_filter, ChunkFilter::All)
                    && matches!(&cache_group.name, ChunkNameGetter::Disabled)
                  {
                    let mut module_group = module_group_map
                      .entry(ModuleGroupKey::Anonymous {
                        cache_group_index: indexed_cache_group.cache_group_index,
                        chunks_key: chunk_combination.key,
                      })
                      .or_insert_with(|| {
                        ModuleGroup::new(None, indexed_cache_group.cache_group_index, cache_group)
                      });
                    module_group.add_module_with_shared_chunks(
                      module.identifier(),
                      chunk_combination.iter().copied(),
                    );
                    continue;
                  }

                  let selected_chunks = match &cache_group.chunk_filter {
                    ChunkFilter::All => SelectedChunks::All(chunk_combination),
                    ChunkFilter::Func(_) => {
                      unreachable!("native cache group should not contain a chunks function")
                    }
                    _ => SelectedChunks::Filtered(
                      chunk_combination
                        .iter()
                        .filter(|chunk| cache_group.chunk_filter.test_internal(chunk, compilation))
                        .copied()
                        .collect(),
                    ),
                  };

                  if selected_chunks.len() < cache_group.min_chunks as usize {
                    continue;
                  }

                  let chunk_name = match &cache_group.name {
                    ChunkNameGetter::String(name) => Some(name.clone()),
                    ChunkNameGetter::Disabled => None,
                    ChunkNameGetter::Fn(_) => {
                      unreachable!("native cache group should not contain a name function")
                    }
                  };
                  merge_matched_item_into_module_group_map(
                    MatchedItem {
                      module,
                      cache_group,
                      cache_group_index: indexed_cache_group.cache_group_index,
                      selected_chunks,
                    },
                    chunk_name,
                    module_group_map,
                    chunk_index_map,
                  );
                }
              }
              Ok(())
            },
          );
        });
    })
    .await
    .into_iter()
    .map(|result| result.to_rspack_result())
    .collect::<Result<Vec<_>>>()?;

    for result in module_group_results {
      result?;
    }

    let module_group_count = module_group_map.len();
    let mut result = Vec::with_capacity(module_group_count);
    result.extend(module_group_map);
    result.sort_by(|a, b| a.0.cmp(&b.0));
    let mut ordered_result =
      ModuleGroupMap::with_capacity_and_hasher(module_group_count, Default::default());
    ordered_result.extend(result);
    Ok(ordered_result)
  }

  // #[tracing::instrument(skip_all)]
  pub(crate) fn remove_all_modules_from_other_module_groups(
    &self,
    placed_module_chunks: &ModuleChunkMap,
    module_group_map: &mut ModuleGroupMap,
    module_sizes: &ModuleSizes,
  ) {
    // remove all modules from other entries and update size
    let keys_of_invalid_group = module_group_map
      .par_iter_mut()
      .filter_map(|(key, other_module_group)| {
        let duplicated_modules = match (
          placed_module_chunks,
          other_module_group.shared_module_chunks(),
        ) {
          (
            ModuleChunkMap::Shared {
              modules,
              chunks: placed_chunks,
            },
            Some(other_chunks),
          ) => {
            other_chunks.intersection(placed_chunks).next()?;
            if other_module_group.modules.len() > modules.len() {
              modules
                .intersection(&other_module_group.modules)
                .copied()
                .collect::<Vec<_>>()
            } else {
              other_module_group
                .modules
                .intersection(modules)
                .copied()
                .collect::<Vec<_>>()
            }
          }
          _ => other_module_group
            .modules
            .iter()
            .filter(|module| {
              let Some(placed_chunks) = placed_module_chunks.get(module) else {
                return false;
              };
              let Some(other_chunks) = other_module_group.get_module_chunks(module) else {
                return false;
              };
              placed_chunks.intersection(other_chunks).next().is_some()
            })
            .copied()
            .collect::<Vec<_>>(),
        };

        if duplicated_modules.is_empty() {
          return None;
        }

        other_module_group.remove_modules(duplicated_modules);

        if other_module_group.modules.is_empty() {
          tracing::trace!(
            "{key} is deleted for having empty modules",
          );
          return Some(key.clone());
        }

        tracing::trace!("other_module_group: {other_module_group:#?}");
        tracing::trace!("placed_module_chunks: {placed_module_chunks:#?}");

        let cache_group = other_module_group.get_cache_group(&self.cache_groups);

        // Since we removed some modules and chunks from the `other_module_group`. There are chances
        // that the `min_chunks` and `min_size` validation is not satisfied anymore.

        // Validate `min_size` again
        if remove_min_size_violating_modules(key, other_module_group, cache_group, module_sizes) {
          tracing::trace!(
            "{key} is deleted for violating min_size {:#?}",
            cache_group.min_size,
          );
          return Some(key.clone());
        }

        other_module_group.rebuild_chunks();

        // Validate `min_chunks` again
        if other_module_group.chunks.len() < cache_group.min_chunks as usize {
          tracing::trace!(
            "{key} is deleted for each_module_group.chunks.len()({:?}) < cache_group.min_chunks({:?})",
            other_module_group.chunks.len(),
            cache_group.min_chunks
          );
          return Some(key.clone());
        }

        let chunks_len = other_module_group.chunks.len();
        if !Self::check_min_size_reduction(
          other_module_group.get_sizes(module_sizes),
          &cache_group.min_size_reduction,
          chunks_len,
        ) {
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

fn merge_matched_item_into_module_group_map(
  matched_item: MatchedItem<'_>,
  chunk_name: Option<String>,
  module_group_map: &FxDashMap<ModuleGroupKey, ModuleGroup>,
  chunk_index_map: &FxHashMap<ChunkUkey, u32>,
) {
  let MatchedItem {
    module,
    cache_group_index,
    cache_group,
    selected_chunks,
  } = matched_item;

  // `Module`s with the same chunk_name would be merged together.
  // `Module`s could be in different `ModuleGroup`s.
  let is_named = chunk_name.is_some();
  let key = if let Some(cache_group_name) = &chunk_name {
    ModuleGroupKey::Named {
      cache_group_index,
      chunk_name: cache_group_name.clone(),
    }
  } else {
    ModuleGroupKey::Anonymous {
      cache_group_index,
      chunks_key: selected_chunks
        .key()
        .unwrap_or_else(|| get_key(selected_chunks.iter().copied(), chunk_index_map)),
    }
  };

  let mut module_group = {
    module_group_map
      .entry(key)
      .or_insert_with(|| ModuleGroup::new(chunk_name, cache_group_index, cache_group))
  };
  merge_matched_item_into_module_group(module, selected_chunks, is_named, &mut module_group);
}

fn merge_matched_item_into_ordered_module_group_map(
  matched_item: MatchedItem<'_>,
  chunk_name: Option<String>,
  module_group_map: &mut ModuleGroupMap,
  chunk_index_map: &FxHashMap<ChunkUkey, u32>,
) {
  let MatchedItem {
    module,
    cache_group_index,
    cache_group,
    selected_chunks,
  } = matched_item;

  let is_named = chunk_name.is_some();
  let key = if let Some(cache_group_name) = &chunk_name {
    ModuleGroupKey::Named {
      cache_group_index,
      chunk_name: cache_group_name.clone(),
    }
  } else {
    ModuleGroupKey::Anonymous {
      cache_group_index,
      chunks_key: selected_chunks
        .key()
        .unwrap_or_else(|| get_key(selected_chunks.iter().copied(), chunk_index_map)),
    }
  };

  let module_group = module_group_map
    .entry(key)
    .or_insert_with(|| ModuleGroup::new(chunk_name, cache_group_index, cache_group));
  merge_matched_item_into_module_group(module, selected_chunks, is_named, module_group);
}

fn merge_matched_item_into_module_group(
  module: &dyn Module,
  selected_chunks: SelectedChunks<'_>,
  is_named: bool,
  module_group: &mut ModuleGroup,
) {
  if is_named {
    module_group.add_module(module.identifier(), selected_chunks.iter().copied());
  } else {
    module_group
      .add_module_with_shared_chunks(module.identifier(), selected_chunks.iter().copied());
  }
}
