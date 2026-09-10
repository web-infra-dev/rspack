use std::{
  hash::{Hash, Hasher},
  ops::Deref,
  sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
  },
};

use futures::{
  FutureExt, StreamExt,
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

use super::{ModuleGroupMap, intersections::collect_intersections};
use crate::{
  SplitChunksNameBatchFn, SplitChunksPlugin,
  common::{
    ChunkFilter, ModuleChunkMap, ModuleChunks, ModuleSizes, is_default_module_layer_filter,
    is_default_module_type_filter,
  },
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
  data: Arc<ChunkCombinationData>,
}

struct ChunkCombinationData {
  chunks: FxHashSet<ChunkUkey>,
  bitmap: super::bitmap::ChunkBitmap,
}

impl ChunkCombination {
  fn new(
    key: ChunksKey,
    chunks: FxHashSet<ChunkUkey>,
    indices: &FxHashMap<ChunkUkey, u32>,
  ) -> Self {
    let mut bitmap = super::bitmap::ChunkBitmap::new(indices.len() + 1);
    for chunk in &chunks {
      bitmap.insert(indices[chunk] as usize);
    }
    Self {
      key,
      data: Arc::new(ChunkCombinationData { chunks, bitmap }),
    }
  }

  fn is_subset(&self, other: &Self) -> bool {
    self.data.bitmap.is_subset(&other.data.bitmap)
  }
}

struct PreparedCombinations {
  values: Vec<ChunkCombination>,
  original_len: usize,
}

struct CandidateRelations {
  chunks: Vec<Vec<u32>>,
  support: Vec<(usize, Vec<usize>)>,
  originals: Vec<ChunksKey>,
}

pub(super) struct IntersectionPreparation<'a> {
  pub settings: Option<(usize, Vec<&'a CacheGroup>)>,
  pub module_sizes: &'a ModuleSizes,
  pub compilation: &'a Compilation,
}

impl IntersectionPreparation<'_> {
  fn use_direct_candidates(&self) -> bool {
    self.settings.as_ref().is_some_and(|(_, groups)| {
      groups
        .iter()
        .all(|group| matches!(group.chunk_filter, ChunkFilter::All))
    })
  }

  fn min_size_bound(&self) -> u64 {
    // Filtering can map different intersections to the same selected chunk
    // set. Their modules may pass minSize only after those candidates merge.
    // A bound on one unfiltered intersection cannot reject that final group.
    if !self.use_direct_candidates() {
      return 0;
    }
    self
      .settings
      .as_ref()
      .into_iter()
      .flat_map(|(_, groups)| groups)
      .flat_map(|group| group.min_size.values())
      .filter(|size| **size > 0.0)
      .map(|size| size.floor() as u64)
      .min()
      .unwrap_or(0)
  }

  fn module_size_bound(&self, module: &ModuleIdentifier) -> u64 {
    let Some((_, groups)) = &self.settings else {
      return u64::MAX;
    };
    let graph = self.compilation.get_module_graph();
    let item = graph
      .module_by_identifier(module)
      .expect("module should exist");
    // Native string/regexp tests are exact and independent of enumeration.
    // Callback tests remain possible matches, so no callback is called twice.
    if !groups.iter().any(|group| match &group.test {
      CacheGroupTest::String(test) => item
        .name_for_condition()
        .is_some_and(|name| name.starts_with(test)),
      CacheGroupTest::RegExp(test) => item
        .name_for_condition()
        .is_some_and(|name| test.test(&name)),
      CacheGroupTest::Fn(_) | CacheGroupTest::Enabled => true,
    }) {
      return 0;
    }
    let sizes = self
      .module_sizes
      .get(module)
      .expect("module should have sizes");
    // A source without a positive threshold, or a module without source types,
    // can survive minSize with arbitrarily few bytes. Do not prune its branch.
    if sizes.is_empty()
      || sizes.iter().any(|(ty, size)| {
        !size.is_finite()
          || *size < 0.0
          || groups.iter().any(|group| {
            let minimum = group.min_size.get(ty).copied().unwrap_or_default();
            minimum.is_nan() || minimum <= 0.0
          })
      })
    {
      return u64::MAX;
    }
    // Round every contribution upward and the threshold downward. Summing all
    // source types and duplicate used-export occurrences is a conservative
    // upper bound for an unfiltered candidate; group tests can only remove
    // contributions. min_size_bound disables pruning when chunks are filtered.
    sizes
      .values()
      .fold(0u64, |total, size| total.saturating_add(size.ceil() as u64))
  }
}

impl Deref for ChunkCombination {
  type Target = FxHashSet<ChunkUkey>;

  fn deref(&self) -> &Self::Target {
    &self.data.chunks
  }
}

enum SelectedChunks<'a> {
  All(&'a ChunkCombination),
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
      Self::Filtered(chunks) => chunks.len(),
    }
  }

  fn iter(&self) -> SelectedChunksIter<'_> {
    match self {
      Self::All(chunks) => SelectedChunksIter::All(chunks.iter()),
      Self::Filtered(chunks) => SelectedChunksIter::Filtered(chunks.iter()),
    }
  }

  fn key(&self) -> Option<ChunksKey> {
    match self {
      Self::All(chunks) => Some(chunks.key),
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

struct PendingNameRequest {
  module: ModuleIdentifier,
  chunks: Vec<ChunkUkey>,
  cache_group_position: usize,
  response: Option<oneshot::Sender<Option<String>>>,
}

// Keep each N-API payload bounded while amortizing the fixed cost of crossing into JavaScript.
const JS_CHUNK_NAME_BATCH_SIZE: usize = 128;

pub(super) fn cache_group_uses_intersections(
  cache_group: &CacheGroup,
  has_name_batch_getter: bool,
) -> bool {
  matches!(&cache_group.name, ChunkNameGetter::Disabled)
    && !has_name_batch_getter
    && (cache_group.min_size.iter().any(|(_, size)| *size > 0.0)
      || cache_group
        .min_size_reduction
        .iter()
        .any(|(_, size)| *size > 0.0))
}

async fn process_name_requests(
  mut receiver: mpsc::UnboundedReceiver<PendingNameRequest>,
  cache_groups: &[IndexedCacheGroup<'_>],
  name_batch_getters: &[Option<SplitChunksNameBatchFn>],
  compilation: &Compilation,
) -> Result<()> {
  let module_graph = compilation.get_module_graph();

  while let Some(first_request) = receiver.next().await {
    let mut requests = Vec::with_capacity(JS_CHUNK_NAME_BATCH_SIZE);
    requests.push(first_request);
    while requests.len() < JS_CHUNK_NAME_BATCH_SIZE
      && let Ok(request) = receiver.try_recv()
    {
      requests.push(request);
    }

    let mut run_start = 0;
    while run_start < requests.len() {
      let cache_group_position = requests[run_start].cache_group_position;
      let mut run_end = run_start + 1;
      while run_end < requests.len()
        && requests[run_end].cache_group_position == cache_group_position
      {
        run_end += 1;
      }

      let indexed_cache_group = &cache_groups[cache_group_position];
      let cache_group = indexed_cache_group.cache_group;
      let get_name = name_batch_getters
        .get(indexed_cache_group.cache_group_index as usize)
        .and_then(Option::as_ref)
        .expect("pending name request should use a batch name callback");
      let contexts = requests[run_start..run_end]
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
      let names = get_name(contexts).await?;
      debug_assert_eq!(names.len(), run_end - run_start);

      for (request, name) in requests[run_start..run_end].iter_mut().zip(names) {
        if let Some(response) = request.response.take() {
          let _ = response.send(name);
        }
      }
      run_start = run_end;
    }
  }

  Ok(())
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
  combinations: FxHashMap<ChunksKey, PreparedCombinations>,
  used_exports_combinations: FxHashMap<ChunksKey, PreparedCombinations>,
  non_used_exports_chunks_keys: Vec<Option<ChunksKey>>,
  grouped_by_exports: Vec<Vec<ChunksKey>>,
  non_used_exports_candidates: Option<CandidateRelations>,
  used_exports_candidates: Option<CandidateRelations>,
  non_used_exports_originals: FxHashSet<ChunksKey>,
  used_exports_originals: FxHashSet<ChunksKey>,
}

enum ChunkCombinations<'a> {
  Slice {
    prepared: &'a PreparedCombinations,
    originals: &'a FxHashSet<ChunksKey>,
    include_intersections: bool,
  },
  UsedExports {
    keys: &'a [ChunksKey],
    combinations: &'a FxHashMap<ChunksKey, PreparedCombinations>,
    originals: &'a FxHashSet<ChunksKey>,
    include_intersections: bool,
  },
}
struct PreparedCombinationsIter<'a> {
  values: std::slice::Iter<'a, ChunkCombination>,
  original_remaining: usize,
  originals: &'a FxHashSet<ChunksKey>,
}
impl<'a> PreparedCombinationsIter<'a> {
  fn new(
    prepared: &'a PreparedCombinations,
    originals: &'a FxHashSet<ChunksKey>,
    include_intersections: bool,
  ) -> Self {
    let end = if include_intersections {
      prepared.values.len()
    } else {
      prepared.original_len
    };
    Self {
      values: prepared.values[..end].iter(),
      original_remaining: prepared.original_len,
      originals,
    }
  }
}
impl<'a> Iterator for PreparedCombinationsIter<'a> {
  type Item = &'a ChunkCombination;
  fn next(&mut self) -> Option<Self::Item> {
    for candidate in self.values.by_ref() {
      let original = self.original_remaining != 0;
      self.original_remaining = self.original_remaining.saturating_sub(1);
      // Original entries are always retained. Avoid publishing a synthesized
      // entry under a key already occupied by an original combination.
      if original || !self.originals.contains(&candidate.key) {
        return Some(candidate);
      }
    }
    None
  }
  fn size_hint(&self) -> (usize, Option<usize>) {
    (0, Some(self.values.len()))
  }
}
enum ChunkCombinationsIter<'a> {
  Slice(PreparedCombinationsIter<'a>),
  UsedExports {
    keys: std::slice::Iter<'a, ChunksKey>,
    combinations: &'a FxHashMap<ChunksKey, PreparedCombinations>,
    current: PreparedCombinationsIter<'a>,
    include_intersections: bool,
  },
}
impl<'a> Iterator for ChunkCombinationsIter<'a> {
  type Item = &'a ChunkCombination;
  fn next(&mut self) -> Option<Self::Item> {
    match self {
      Self::Slice(iter) => iter.next(),
      Self::UsedExports {
        keys,
        combinations,
        current,
        include_intersections,
      } => loop {
        if let Some(value) = current.next() {
          return Some(value);
        }
        let prepared = combinations.get(keys.next()?).expect("prepared chunk set");
        *current =
          PreparedCombinationsIter::new(prepared, current.originals, *include_intersections);
      },
    }
  }
}
impl<'a> IntoIterator for &'a ChunkCombinations<'a> {
  type Item = &'a ChunkCombination;
  type IntoIter = ChunkCombinationsIter<'a>;
  fn into_iter(self) -> Self::IntoIter {
    match self {
      ChunkCombinations::Slice {
        prepared,
        originals,
        include_intersections,
      } => ChunkCombinationsIter::Slice(PreparedCombinationsIter::new(
        prepared,
        originals,
        *include_intersections,
      )),
      ChunkCombinations::UsedExports {
        keys,
        combinations,
        originals,
        include_intersections,
      } => ChunkCombinationsIter::UsedExports {
        keys: keys.iter(),
        combinations,
        current: PreparedCombinationsIter {
          values: [].iter(),
          original_remaining: 0,
          originals,
        },
        include_intersections: *include_intersections,
      },
    }
  }
}

impl Combinator {
  fn candidates(&self, used_exports: bool) -> Option<&CandidateRelations> {
    if used_exports {
      self.used_exports_candidates.as_ref()
    } else {
      self.non_used_exports_candidates.as_ref()
    }
  }

  fn prepare_direct_groups(
    &self,
    indexed: &IndexedCacheGroup<'_>,
    matches: &[AtomicBool],
    all_modules: &[ModuleIdentifier],
    chunks_by_index: &[ChunkUkey],
  ) -> Vec<(ModuleGroupKey, ModuleGroup)> {
    let cache_group = indexed.cache_group;
    let candidates = self
      .candidates(cache_group.used_exports)
      .expect("direct candidate relations");
    let rows: FxHashMap<_, _> = candidates
      .originals
      .iter()
      .enumerate()
      .map(|(i, key)| (*key, i))
      .collect();
    let mut modules_by_row = vec![vec![]; rows.len()];
    for (module, matched) in matches.iter().enumerate() {
      if !matched.load(Ordering::Relaxed) {
        continue;
      }
      if cache_group.used_exports {
        for key in &self.grouped_by_exports[module] {
          if let Some(row) = rows.get(key) {
            modules_by_row[*row].push(module);
          }
        }
      } else if let Some(key) = self.non_used_exports_chunks_keys[module]
        && let Some(row) = rows.get(&key)
      {
        modules_by_row[*row].push(module);
      }
    }
    candidates
      .support
      .par_iter()
      .filter_map(|(index, support)| {
        let chunks = &candidates.chunks[*index];
        if chunks.len() < cache_group.min_chunks as usize {
          return None;
        }
        let mut group = ModuleGroup::new(None, indexed.cache_group_index, cache_group);
        // These rows and module keys belong to the current priority snapshot.
        // Candidate support already proves containment; no per-module chunk scan
        // is needed, and synthesized candidates never need an expanded cache row.
        for row in support {
          group.modules.extend(
            modules_by_row[*row]
              .iter()
              .copied()
              .map(|module| all_modules[module]),
          );
        }
        if group.modules.is_empty() {
          return None;
        }
        group.chunks.extend(
          chunks
            .iter()
            .map(|index| chunks_by_index[*index as usize - 1]),
        );
        // Candidate indices are already sorted. Hash them in the same order
        // as get_key without mapping back and allocating another sorted copy.
        let mut hasher = FxHasher::default();
        for chunk in chunks {
          chunk.hash(&mut hasher);
        }
        let key = ModuleGroupKey::Anonymous {
          cache_group_index: indexed.cache_group_index,
          chunks_key: hasher.finish(),
        };
        Some((key, group))
      })
      .collect()
  }

  // Querying a prepared set does not enumerate, hash its chunks or allocate a result Vec.
  fn get_non_used_exports_combs(&self, module_index: usize) -> &PreparedCombinations {
    let key = self.non_used_exports_chunks_keys[module_index].expect("prepared module chunk key");
    self.combinations.get(&key).expect("prepared combinations")
  }

  fn group_chunks_by_exports(
    module_identifier: &ModuleIdentifier,
    module_chunks: &FxHashSet<ChunkUkey>,
    exports_info_artifact: &ExportsInfoArtifact,
    chunk_by_ukey: &ChunkByUkey,
    chunk_index_map: &FxHashMap<ChunkUkey, u32>,
  ) -> Vec<ChunkCombination> {
    // A single chunk cannot produce runtime-dependent usage groups.
    if module_chunks.len() == 1 {
      return vec![ChunkCombination::new(
        get_key(module_chunks.iter().copied(), chunk_index_map),
        module_chunks.clone(),
        chunk_index_map,
      )];
    }

    let exports_info = exports_info_artifact.get_exports_info_data(module_identifier);
    let mut grouped_by_used_exports: FxHashMap<UsageKey, FxHashSet<ChunkUkey>> = Default::default();
    let mut runtime_key_map = RuntimeKeyMap::default();
    for chunk_ukey in module_chunks.iter().copied() {
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
      .map(|chunks| {
        ChunkCombination::new(
          get_key(chunks.iter().copied(), chunk_index_map),
          chunks,
          chunk_index_map,
        )
      })
      .collect()
  }

  fn get_combinations(
    &self,
    module_index: usize,
    used_exports: bool,
    include_intersections: bool,
  ) -> ChunkCombinations<'_> {
    if used_exports {
      ChunkCombinations::UsedExports {
        keys: &self.grouped_by_exports[module_index],
        combinations: &self.used_exports_combinations,
        originals: &self.used_exports_originals,
        include_intersections,
      }
    } else {
      ChunkCombinations::Slice {
        prepared: self.get_non_used_exports_combs(module_index),
        originals: &self.non_used_exports_originals,
        include_intersections,
      }
    }
  }

  // Build the original-subset index once for this priority snapshot.
  fn index_original_sets(
    combinations: &mut FxHashMap<ChunksKey, PreparedCombinations>,
    chunk_sets_by_count: &[ChunkCombination],
  ) {
    debug_assert!(combinations.is_empty());
    if chunk_sets_by_count.len() >= 128 {
      Self::index_original_sets_from_postings(combinations, chunk_sets_by_count);
      return;
    }
    let rows = chunk_sets_by_count
      .par_iter()
      .map(|set| {
        let mut values = chunk_sets_by_count
          .iter()
          .take_while(|candidate| candidate.len() < set.len())
          .filter(|candidate| candidate.is_subset(set))
          .cloned()
          .collect::<Vec<_>>();
        values.push(set.clone());
        (
          set.key,
          PreparedCombinations {
            original_len: values.len(),
            values,
          },
        )
      })
      .collect::<Vec<_>>();
    combinations.extend(rows);
  }

  fn index_original_sets_from_postings(
    combinations: &mut FxHashMap<ChunksKey, PreparedCombinations>,
    originals: &[ChunkCombination],
  ) {
    let mut ordered = originals.iter().collect::<Vec<_>>();
    ordered.sort_unstable_by_key(|set| (set.len(), set.key));
    let mut postings: FxHashMap<ChunkUkey, Vec<usize>> = FxHashMap::default();
    for (index, set) in ordered.iter().enumerate() {
      for chunk in set.iter() {
        postings.entry(*chunk).or_default().push(index);
      }
    }
    // Every superset must contain the rarest chunk of this original. Check
    // only those rows instead of testing every smaller/larger pair in the graph.
    let supersets = ordered
      .par_iter()
      .map(|set| {
        let Some(anchor) = set.iter().min_by_key(|chunk| postings[chunk].len()) else {
          return ordered
            .iter()
            .enumerate()
            .filter_map(|(index, superset)| (!superset.is_empty()).then_some(index))
            .collect::<Vec<_>>();
        };
        postings[anchor]
          .iter()
          .copied()
          .filter(|index| {
            let superset = ordered[*index];
            set.len() < superset.len() && set.is_subset(superset)
          })
          .collect::<Vec<_>>()
      })
      .collect::<Vec<_>>();
    let mut subsets = vec![vec![]; ordered.len()];
    // Transpose in original order so each cached row keeps the same iteration
    // order as the full scan, independently of parallel discovery scheduling.
    for (subset, supersets) in supersets.into_iter().enumerate() {
      for superset in supersets {
        subsets[superset].push(subset);
      }
    }
    combinations.extend(
      subsets
        .into_par_iter()
        .enumerate()
        .map(|(index, subsets)| {
          let mut values = Vec::with_capacity(subsets.len() + 1);
          values.extend(subsets.into_iter().map(|subset| ordered[subset].clone()));
          values.push(ordered[index].clone());
          (
            ordered[index].key,
            PreparedCombinations {
              original_len: values.len(),
              values,
            },
          )
        })
        .collect::<Vec<_>>(),
    );
  }

  fn prepare_combinations(
    combinations: &mut FxHashMap<ChunksKey, PreparedCombinations>,
    chunk_sets_by_count: Vec<ChunkCombination>,
    chunk_set_size_bounds: FxHashMap<ChunksKey, u64>,
    min_size_bound: u64,
    chunk_index_map: &FxHashMap<ChunkUkey, u32>,
    intersection_min_chunks: Option<usize>,
    direct_candidates: bool,
  ) -> Option<CandidateRelations> {
    let mut candidates = None;
    let mut intersections_by_chunks_key: FxHashMap<ChunksKey, Vec<ChunkCombination>> =
      FxHashMap::default();
    if let Some(min_chunks) = intersection_min_chunks {
      let original_sets = chunk_sets_by_count
        .iter()
        .map(|set| {
          let mut chunks = set
            .iter()
            .map(|chunk| *chunk_index_map.get(chunk).expect("chunk should have index"))
            .collect::<Vec<_>>();
          chunks.sort_unstable();
          chunks
        })
        .collect::<Vec<_>>();
      let (intersections, propagation) = collect_intersections(
        &original_sets,
        &chunk_sets_by_count
          .iter()
          .map(|set| chunk_set_size_bounds[&set.key])
          .collect::<Vec<_>>(),
        min_size_bound,
        min_chunks,
      );
      if direct_candidates {
        candidates = Some(CandidateRelations {
          chunks: intersections,
          support: propagation,
          originals: chunk_sets_by_count.iter().map(|set| set.key).collect(),
        });
      } else {
        let chunks_by_index: FxHashMap<_, _> = chunk_index_map
          .iter()
          .map(|(chunk, index)| (*index, *chunk))
          .collect();
        for (intersection_index, supersets) in propagation {
          let indices = &intersections[intersection_index];
          let chunks: FxHashSet<_> = indices
            .iter()
            .map(|index| *chunks_by_index.get(index).expect("index should have chunk"))
            .collect();
          let intersection = ChunkCombination::new(
            get_key(chunks.iter().copied(), chunk_index_map),
            chunks,
            chunk_index_map,
          );
          for superset_index in supersets {
            intersections_by_chunks_key
              .entry(chunk_sets_by_count[superset_index].key)
              .or_default()
              .push(intersection.clone());
          }
        }
      }
    }
    Self::index_original_sets(combinations, &chunk_sets_by_count);
    for (key, intersections) in intersections_by_chunks_key {
      combinations
        .get_mut(&key)
        .expect("prepared original")
        .values
        .extend(intersections);
    }
    candidates
  }

  pub(super) fn prepare_group_by_chunks(
    &mut self,
    all_modules: &[ModuleIdentifier],
    module_chunks: &ModuleChunks,
    chunk_index_map: &FxHashMap<ChunkUkey, u32>,
    min_chunks: usize,
    intersection_preparation: IntersectionPreparation<'_>,
  ) {
    let direct_candidates = intersection_preparation.use_direct_candidates();
    let intersection_min_chunks = intersection_preparation
      .settings
      .as_ref()
      .map(|(min_chunks, _)| *min_chunks);
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
    let mut chunk_set_size_bounds = FxHashMap::<ChunksKey, u64>::default();
    for (module_index, chunk_key) in self
      .non_used_exports_chunks_keys
      .iter()
      .enumerate()
      .filter_map(|(module_index, chunk_key)| chunk_key.map(|chunk_key| (module_index, chunk_key)))
    {
      if intersection_min_chunks.is_some() {
        let size = intersection_preparation.module_size_bound(&all_modules[module_index]);
        let total = chunk_set_size_bounds.entry(chunk_key).or_default();
        *total = total.saturating_add(size);
      }
      chunk_sets_in_graph.entry(chunk_key).or_insert_with(|| {
        ChunkCombination::new(
          chunk_key,
          module_chunks
            .get(module_index)
            .expect("module chunks")
            .clone(),
          chunk_index_map,
        )
      });
    }

    let mut chunk_sets_by_count = Vec::<ChunkCombination>::with_capacity(chunk_sets_in_graph.len());
    for chunks in chunk_sets_in_graph.values() {
      chunk_sets_by_count.push(chunks.clone());
    }

    chunk_sets_by_count.sort_unstable_by_key(|chunks| (chunks.len(), chunks.key));

    self.non_used_exports_originals = self
      .non_used_exports_chunks_keys
      .iter()
      .flatten()
      .copied()
      .collect();
    self.non_used_exports_candidates = Self::prepare_combinations(
      &mut self.combinations,
      chunk_sets_by_count,
      chunk_set_size_bounds,
      intersection_preparation.min_size_bound(),
      chunk_index_map,
      intersection_min_chunks,
      direct_candidates,
    );
  }

  pub(super) fn prepare_group_by_used_exports(
    &mut self,
    all_modules: &[ModuleIdentifier],
    exports_info_artifact: &ExportsInfoArtifact,
    chunk_by_ukey: &ChunkByUkey,
    module_chunks: &ModuleChunks,
    chunk_index_map: &FxHashMap<ChunkUkey, u32>,
    intersection_preparation: IntersectionPreparation<'_>,
  ) {
    let direct_candidates = intersection_preparation.use_direct_candidates();
    let intersection_min_chunks = intersection_preparation
      .settings
      .as_ref()
      .map(|(min_chunks, _)| *min_chunks);
    let (grouped_by_exports, used_exports_chunks): (Vec<_>, Vec<_>) = all_modules
      .par_iter()
      .enumerate()
      .map(|(module_index, module)| {
        let grouped_chunks = Self::group_chunks_by_exports(
          module,
          module_chunks
            .get(module_index)
            .expect("should have module chunks"),
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
    let mut chunk_set_size_bounds = FxHashMap::<ChunksKey, u64>::default();
    for (module_index, used_exports_chunks) in used_exports_chunks.into_iter().enumerate() {
      for chunks in used_exports_chunks {
        let chunk_key = chunks.key;
        if intersection_min_chunks.is_some() {
          let size = intersection_preparation.module_size_bound(&all_modules[module_index]);
          let total = chunk_set_size_bounds.entry(chunk_key).or_default();
          *total = total.saturating_add(size);
        }
        if let std::collections::hash_map::Entry::Vacant(entry) =
          used_exports_chunk_sets_in_graph.entry(chunk_key)
        {
          used_exports_chunk_sets_by_count.push(chunks.clone());
          entry.insert(chunks);
        }
      }
    }

    used_exports_chunk_sets_by_count.sort_unstable_by_key(|chunks| (chunks.len(), chunks.key));

    self.used_exports_originals = self.grouped_by_exports.iter().flatten().copied().collect();
    self.used_exports_candidates = Self::prepare_combinations(
      &mut self.used_exports_combinations,
      used_exports_chunk_sets_by_count,
      chunk_set_size_bounds,
      intersection_preparation.min_size_bound(),
      chunk_index_map,
      intersection_min_chunks,
      direct_candidates,
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
    let module_graph = compilation.get_module_graph();
    let module_group_map: FxDashMap<ModuleGroupKey, ModuleGroup> = FxDashMap::default();
    let name_batch_getters = self.name_batch_getters.as_deref();
    // Module callbacks still run once in the existing asynchronous pass.
    // Anonymous all-chunk candidates can then be assembled independently,
    // avoiding a shared-map lookup for every candidate/module association.
    let direct_matches: Vec<_> = cache_groups
      .iter()
      .map(|indexed| {
        let group = indexed.cache_group;
        let has_name_batch_getter = name_batch_getters.is_some_and(|getters| {
          getters
            .get(indexed.cache_group_index as usize)
            .is_some_and(Option::is_some)
        });
        (combinator.candidates(group.used_exports).is_some()
          && cache_group_uses_intersections(group, has_name_batch_getter)
          && matches!(group.chunk_filter, ChunkFilter::All))
        .then(|| {
          (0..all_modules.len())
            .map(|_| AtomicBool::new(false))
            .collect::<Vec<_>>()
        })
      })
      .collect();
    let has_name_batch_callback = name_batch_getters.is_some_and(|name_batch_getters| {
      cache_groups.iter().any(|indexed_cache_group| {
        name_batch_getters
          .get(indexed_cache_group.cache_group_index as usize)
          .is_some_and(Option::is_some)
      })
    });
    let (name_sender, name_receiver) = if has_name_batch_callback {
      let (sender, receiver) = mpsc::unbounded();
      (Some(sender), Some(receiver))
    } else {
      (None, None)
    };

    // These filters cannot yield or invoke user callbacks. Run their CPU work
    // on Rayon instead of allocating and joining one Tokio task per module.
    let use_native_preparation = !has_name_batch_callback
      && cache_groups.iter().all(|indexed| {
        let group = indexed.cache_group;
        is_default_module_type_filter(&group.r#type)
          && is_default_module_layer_filter(&group.layer)
          && !matches!(group.test, CacheGroupTest::Fn(_))
          && !group.chunk_filter.is_func()
          && !matches!(group.name, ChunkNameGetter::Fn(_))
      });
    let process_module = async |module_index: usize,
                                module_identifier: ModuleIdentifier,
                                name_sender: Option<mpsc::UnboundedSender<PendingNameRequest>>|
           -> Result<()> {
      let belong_to_chunks = module_chunks
        .get(module_index)
        .expect("should have module chunks");
      if belong_to_chunks.is_empty() {
        return Ok(());
      }

      let module = module_graph
        .module_by_identifier(&module_identifier)
        .expect("should have module")
        .as_ref();
      let mut used_exports_combinations = [None, None];
      let mut non_used_exports_combinations = [None, None];

      for (cache_group_position, indexed_cache_group) in cache_groups.iter().enumerate() {
        let cache_group = indexed_cache_group.cache_group;
        let has_name_batch_getter = name_batch_getters.is_some_and(|getters| {
          getters
            .get(indexed_cache_group.cache_group_index as usize)
            .is_some_and(Option::is_some)
        });
        let include_intersections = self.optimize_for_size
          && cache_group_uses_intersections(cache_group, has_name_batch_getter);
        if !use_native_preparation
          && (!(cache_group.r#type)(module)
            || !(cache_group.layer)(module.get_layer().map(ToString::to_string)).await?)
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
          CacheGroupTest::Fn(test) => test(CacheGroupTestFnCtx {
            compilation,
            module,
          })
          .await?
          .unwrap_or_default(),
          CacheGroupTest::Enabled => true,
        };
        if !is_match || belong_to_chunks.len() < cache_group.min_chunks as usize {
          continue;
        }
        if let Some(matches) = &direct_matches[cache_group_position] {
          matches[module_index].store(true, Ordering::Relaxed);
        }

        let combinations = if cache_group.used_exports {
          let combinations = &mut used_exports_combinations[include_intersections as usize];
          if combinations.is_none() {
            *combinations =
              Some(combinator.get_combinations(module_index, true, include_intersections));
          }
          combinations
            .as_ref()
            .expect("should have used exports combinations")
        } else {
          let combinations = &mut non_used_exports_combinations[include_intersections as usize];
          if combinations.is_none() {
            *combinations =
              Some(combinator.get_combinations(module_index, false, include_intersections));
          }
          combinations
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
            && !has_name_batch_getter
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
            ChunkFilter::Func(_) => SelectedChunks::Filtered(
              join_all(chunk_combination.iter().map(|chunk| async move {
                cache_group
                  .chunk_filter
                  .test_func(chunk, compilation)
                  .await
                  .map(|matched| (chunk, matched))
              }))
              .await
              .into_iter()
              .collect::<Result<Vec<_>>>()?
              .into_iter()
              .filter_map(|(chunk, matched)| matched.then_some(*chunk))
              .collect(),
            ),
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

          let chunk_name = if has_name_batch_getter {
            let name_sender = name_sender
              .as_ref()
              .expect("name callback should have a batch coordinator");
            let (response, response_receiver) = oneshot::channel();
            if name_sender
              .unbounded_send(PendingNameRequest {
                module: module.identifier(),
                chunks: selected_chunks.iter().copied().collect(),
                cache_group_position,
                response: Some(response),
              })
              .is_err()
            {
              return Ok(());
            }
            let Ok(chunk_name) = response_receiver.await else {
              return Ok(());
            };
            chunk_name
          } else {
            match &cache_group.name {
              ChunkNameGetter::String(name) => Some(name.clone()),
              ChunkNameGetter::Disabled => None,
              ChunkNameGetter::Fn(get_name) => {
                let chunks = selected_chunks.iter().copied().collect::<Vec<_>>();
                get_name(ChunkNameGetterFnCtx {
                  module,
                  compilation,
                  chunks: &chunks,
                  cache_group_key: &cache_group.key,
                })
                .await?
              }
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
            &module_group_map,
            chunk_index_map,
          );
        }
      }
      Ok(())
    };

    if use_native_preparation {
      all_modules
        .par_iter()
        .enumerate()
        .try_for_each(|(module_index, module_identifier)| {
          process_module(module_index, *module_identifier, None)
            .now_or_never()
            .expect("native cache-group preparation should not yield")
        })?;
    } else {
      let module_group_results = rspack_parallel::scope::<_, Result<_>>(|token| {
        if let Some(name_receiver) = name_receiver {
          let coordinator = unsafe {
            token.used((
              name_receiver,
              &cache_groups,
              name_batch_getters.expect("should have batch name getters"),
              compilation,
            ))
          };
          coordinator.spawn(
            |(name_receiver, cache_groups, name_batch_getters, compilation)| async move {
              process_name_requests(name_receiver, cache_groups, name_batch_getters, compilation)
                .await
            },
          );
        }
        all_modules
          .iter()
          .enumerate()
          .for_each(|(module_index, module_identifier)| {
            let name_sender = name_sender.clone();
            let s = unsafe {
              token.used((
                &process_module,
                module_index,
                *module_identifier,
                name_sender,
              ))
            };
            s.spawn(
              |(process_module, module_index, module_identifier, name_sender)| async move {
                process_module(module_index, module_identifier, name_sender).await
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
    }

    let module_group_count = module_group_map.len();
    let mut result = Vec::with_capacity(module_group_count);
    result.extend(module_group_map);
    if direct_matches.iter().any(Option::is_some) {
      let mut chunks_by_index = chunk_index_map
        .iter()
        .map(|(chunk, index)| (*index, *chunk))
        .collect::<Vec<_>>();
      chunks_by_index.sort_unstable_by_key(|(index, _)| *index);
      // optimize_chunks assigns dense, one-based indices to the original chunks.
      let chunks_by_index = chunks_by_index
        .into_iter()
        .map(|(_, chunk)| chunk)
        .collect::<Vec<_>>();
      for (indexed, matches) in cache_groups.iter().zip(&direct_matches) {
        if let Some(matches) = matches {
          result.extend(combinator.prepare_direct_groups(
            indexed,
            matches,
            all_modules,
            &chunks_by_index,
          ));
        }
      }
    }
    let module_group_count = result.len();
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
    let placed_chunk_mask = placed_module_chunks.chunk_mask();
    let keys_of_invalid_group = module_group_map
      .par_iter_mut()
      .map_init(Vec::new, |duplicated_modules, (key, other_module_group)| {
        // Keep the exact intersection order, but reuse this job's temporary
        // storage across groups instead of allocating for every intersection.
        duplicated_modules.clear();
        if !other_module_group.may_have_chunks_in_mask(placed_chunk_mask) {
          return None;
        }
        let original_module_count = other_module_group.modules.len();
        match (
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
            if other_module_group.modules.is_subset(modules) {
              return Some(key.clone());
            }
            other_module_group.remove_shared_modules(modules);
          }
          _ => {
            duplicated_modules.extend(other_module_group
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
            .copied());
            if duplicated_modules.len() == original_module_count {
              return Some(key.clone());
            }
            other_module_group.remove_modules(duplicated_modules.iter().copied());
          }
        };

        if other_module_group.modules.len() == original_module_count {
          return None;
        }

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
      .flatten()
      .collect::<Vec<_>>();

    let removed = keys_of_invalid_group
      .into_iter()
      .filter_map(|key| module_group_map.swap_remove(&key))
      .collect::<Vec<_>>();
    removed.into_par_iter().for_each(drop);
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
