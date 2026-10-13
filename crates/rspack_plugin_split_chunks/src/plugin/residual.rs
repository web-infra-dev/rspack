//! Experimental reuse of native cache-group matches and source relationships.

use std::hash::{Hash, Hasher};

use itertools::Either;
use rspack_collections::{IdentifierMap, IdentifierSet};
use rspack_core::{ChunkGroupUkey, ChunkUkey, Compilation, ModuleIdentifier, SourceType};
use rustc_hash::{FxHashMap, FxHashSet, FxHasher};
use smallvec::SmallVec;

use super::{ModuleGroupMap, module_group::get_key};
use crate::{
  CacheGroup, SplitChunksPlugin,
  common::{ModuleChunkMap, ModuleSizes},
  module_group::{IndexedCacheGroup, ModuleGroup, ModuleGroupKey},
};

#[derive(Default)]
pub(super) struct ResidualSupports {
  // One row per original used-export partition, not one row per derived candidate.
  originals: FxHashMap<ModuleGroupKey, ModuleGroup>,
  seeds: Vec<(ModuleGroupKey, FxHashSet<ChunkUkey>, Option<Vec<u64>>)>,
}

impl ResidualSupports {
  pub(super) fn original_count(&self) -> usize {
    self.originals.len()
  }

  pub(super) fn add_original(
    &mut self,
    indexed: &IndexedCacheGroup<'_>,
    module: ModuleIdentifier,
    chunks_key: u64,
    chunks: &FxHashSet<ChunkUkey>,
  ) {
    self
      .originals
      .entry(ModuleGroupKey::Anonymous {
        cache_group_index: indexed.cache_group_index,
        chunks_key,
      })
      .or_insert_with(|| {
        let mut row = ModuleGroup::new(None, indexed.cache_group_index, indexed.cache_group);
        row.chunks = chunks.clone();
        row
      })
      .modules
      .insert(module);
  }

  pub(super) fn add_seed(
    &mut self,
    key: &ModuleGroupKey,
    group: &ModuleGroup,
    originals: Option<Vec<u64>>,
  ) {
    if matches!(key, ModuleGroupKey::Anonymous { .. }) {
      self.seeds.push((
        key.clone(),
        group
          .shared_module_chunks()
          .expect("anonymous source set")
          .clone(),
        originals,
      ));
    }
  }

  pub(super) fn add_fallback(
    &mut self,
    key: &ModuleGroupKey,
    group: &ModuleGroup,
    cache_groups: &[CacheGroup],
  ) {
    let mut original = ModuleGroup::new(
      group.chunk_name.clone(),
      group.cache_group_index,
      group.get_cache_group(cache_groups),
    );
    if let Some(chunks) = group.shared_module_chunks() {
      original.modules = group.modules.clone();
      original.chunks = chunks.clone();
    } else {
      for module in &group.modules {
        original.add_module(
          *module,
          group
            .get_module_chunks(module)
            .expect("selected sources")
            .iter()
            .copied(),
        );
      }
    }
    self.originals.insert(key.clone(), original);
  }
}

struct SourceRow {
  key: ModuleGroupKey,
  cache: u32,
  original: FxHashSet<ChunkUkey>,
  changed: bool,
  // Each residual belongs to this original source row. Its deleted edges
  // update every supported native seed without intersecting source sets again.
  residuals: FxHashMap<ModuleGroupKey, SmallVec<[ChunkUkey; 4]>>,
}

#[derive(Default)]
struct SizeUpperBound {
  sizes: SmallVec<[(SourceType, f64); 2]>,
  unbounded: bool,
}

impl SizeUpperBound {
  fn add(&mut self, ty: SourceType, size: f64) {
    if let Some((_, total)) = self.sizes.iter_mut().find(|(source, _)| *source == ty) {
      if size != 0.0 {
        *total = (*total + size).next_up();
      }
    } else {
      self
        .sizes
        .push((ty, if size == 0.0 { 0.0 } else { size.next_up() }));
    }
  }

  fn for_group(group: &ModuleGroup, sizes: &ModuleSizes) -> Self {
    let mut upper = Self::default();
    for module in &group.modules {
      let Some(sizes) = sizes.get(module) else {
        upper.unbounded = true;
        continue;
      };
      upper.unbounded |= sizes.is_empty();
      for (ty, size) in sizes.iter() {
        // Untyped/custom negative sizes may survive native constraints.
        // Keep these rows rather than drawing a bound from their sizes.
        if !size.is_finite() || *size < 0.0 {
          upper.unbounded = true;
        }
        upper.add(*ty, *size);
      }
    }
    upper
  }

  fn include(&mut self, other: &Self) {
    self.unbounded |= other.unbounded;
    for (ty, size) in &other.sizes {
      self.add(*ty, *size);
    }
  }

  fn may_fit(&self, group: &CacheGroup, source_count: usize) -> bool {
    // A surviving source type must fit both constraints individually. Do not
    // require ALL upper-bound types to fit: native pruning can remove a type.
    // Overlapping supports only overestimate size; round upward for custom
    // floating-point size implementations as well.
    self.unbounded
      || self.sizes.iter().any(|(ty, size)| {
        *size >= group.min_size.get(ty).copied().unwrap_or_default()
          && (*size == 0.0
            || (*size * source_count as f64).next_up()
              >= group
                .min_size_reduction
                .get(ty)
                .copied()
                .unwrap_or_default())
      })
  }
}

fn posting_index<'a>(
  rows: impl Iterator<Item = (usize, u32, &'a FxHashSet<ChunkUkey>)>,
) -> FxHashMap<(u32, ChunkUkey), Vec<usize>> {
  let mut postings = FxHashMap::<_, Vec<_>>::default();
  for (index, cache, chunks) in rows {
    for chunk in chunks {
      postings.entry((cache, *chunk)).or_default().push(index);
    }
  }
  postings
}

// Native candidates retain their owned set; complete residual rows borrow the
// existing set. Derived candidates need only a compact list while size bounds
// reject almost all of them. Materialize a HashSet only after that check.
enum ResidualSeed<'a> {
  Native(FxHashSet<ChunkUkey>),
  Existing(&'a FxHashSet<ChunkUkey>),
  Projected(SmallVec<[ChunkUkey; 4]>),
}

impl ResidualSeed<'_> {
  fn len(&self) -> usize {
    match self {
      Self::Native(chunks) => chunks.len(),
      Self::Existing(chunks) => chunks.len(),
      Self::Projected(chunks) => chunks.len(),
    }
  }

  fn iter(&self) -> impl Iterator<Item = &ChunkUkey> {
    match self {
      Self::Native(chunks) => Either::Left(chunks.iter()),
      Self::Existing(chunks) => Either::Left(chunks.iter()),
      Self::Projected(chunks) => Either::Right(chunks.iter()),
    }
  }

  fn is_subset(&self, chunks: &FxHashSet<ChunkUkey>) -> bool {
    self.len() <= chunks.len() && self.iter().all(|chunk| chunks.contains(chunk))
  }

  fn into_chunks(self) -> FxHashSet<ChunkUkey> {
    match self {
      Self::Native(chunks) => chunks,
      Self::Existing(chunks) => chunks.clone(),
      Self::Projected(chunks) => chunks.into_iter().collect(),
    }
  }
}

fn shortest_posting<'a, 'b>(
  cache: u32,
  chunks: impl Iterator<Item = &'b ChunkUkey>,
  postings: &'a FxHashMap<(u32, ChunkUkey), Vec<usize>>,
) -> Option<&'a [usize]> {
  let mut smallest: Option<&[usize]> = None;
  for chunk in chunks {
    let list = postings.get(&(cache, *chunk))?.as_slice();
    if smallest.is_none_or(|smallest| list.len() < smallest.len()) {
      smallest = Some(list);
    }
  }
  smallest
}

impl SplitChunksPlugin {
  pub(super) fn prepare_residual_candidates(
    &self,
    supports: ResidualSupports,
    occupied: &IdentifierMap<FxHashSet<ChunkUkey>>,
    indices: &FxHashMap<ChunkUkey, u32>,
    sizes: &ModuleSizes,
  ) -> ModuleGroupMap {
    let audit = std::env::var_os("RSPACK_RESIDUAL_AUDIT").is_some();
    let row_started = std::time::Instant::now();
    let retained_modules = supports
      .originals
      .values()
      .map(|row| row.modules.len())
      .sum::<usize>();
    let mut originals = supports.originals.into_iter().collect::<Vec<_>>();
    originals.sort_by(|a, b| a.0.cmp(&b.0));
    let mut rows = Vec::new();
    let mut live = FxHashMap::<ModuleGroupKey, ModuleGroup>::default();
    let mut candidates = ModuleGroupMap::default();
    for (original_key, original) in originals {
      let cache_group = original.get_cache_group(&self.cache_groups);
      if original.chunk_name.is_some() {
        // Preserve the resolved name and use only common live sources. The
        // physical-copy guard checks existing target modules before commit.
        let mut viable = IdentifierSet::default();
        let mut common: Option<FxHashSet<ChunkUkey>> = None;
        for module in &original.modules {
          let mut remaining = original
            .get_module_chunks(module)
            .expect("named sources")
            .clone();
          if let Some(removed) = occupied.get(module) {
            remaining.retain(|chunk| !removed.contains(chunk));
          }
          if remaining.len() < cache_group.min_chunks as usize || remaining.is_empty() {
            continue;
          }
          viable.insert(*module);
          if let Some(common) = &mut common {
            common.retain(|chunk| remaining.contains(chunk));
          } else {
            common = Some(remaining);
          }
        }
        let Some(common) =
          common.filter(|q| !q.is_empty() && q.len() >= cache_group.min_chunks as usize)
        else {
          continue;
        };
        let key = ModuleGroupKey::Named {
          cache_group_index: original.cache_group_index,
          chunk_name: original.chunk_name.clone().expect("resolved name"),
        };
        let group = candidates.entry(key).or_insert_with(|| {
          ModuleGroup::new(
            original.chunk_name.clone(),
            original.cache_group_index,
            cache_group,
          )
        });
        for module in viable {
          group.add_module(module, common.iter().copied());
        }
        continue;
      }
      let source = original
        .shared_module_chunks()
        .expect("anonymous source row");
      let mut row = SourceRow {
        key: original_key.clone(),
        cache: original.cache_group_index,
        original: source.clone(),
        changed: false,
        residuals: FxHashMap::default(),
      };
      for module in &original.modules {
        let removed = occupied.get(module);
        if removed.is_none_or(|removed| source.is_disjoint(removed)) {
          // Unchanged memberships are the common case. Reuse their existing
          // source key and clone the large source set once per row, not once
          // per module, and never re-sort it for each unchanged member.
          row.residuals.entry(original_key.clone()).or_default();
          live
            .entry(original_key.clone())
            .or_insert_with(|| {
              let mut group = ModuleGroup::new(None, row.cache, cache_group);
              group.chunks = source.clone();
              group
            })
            .modules
            .insert(*module);
          continue;
        }
        if occupied
          .get(module)
          .is_some_and(|removed| source.len() <= removed.len() && source.is_subset(removed))
        {
          row.changed = true;
          continue;
        }
        let mut deleted = SmallVec::<[ChunkUkey; 4]>::new();
        if let Some(removed) = removed {
          if removed.len() < source.len() {
            deleted.extend(
              removed
                .iter()
                .filter(|chunk| source.contains(chunk))
                .copied(),
            );
          } else {
            deleted.extend(
              source
                .iter()
                .filter(|chunk| removed.contains(chunk))
                .copied(),
            );
          }
        }
        row.changed |= !deleted.is_empty();
        let mut remaining = source.clone();
        for chunk in &deleted {
          remaining.remove(chunk);
        }
        if remaining.len() < cache_group.min_chunks as usize || remaining.is_empty() {
          continue;
        }
        let key = ModuleGroupKey::Anonymous {
          cache_group_index: row.cache,
          chunks_key: get_key(remaining.iter().copied(), indices),
        };
        row.residuals.entry(key.clone()).or_insert(deleted);
        live
          .entry(key)
          .or_insert_with(|| {
            let mut group = ModuleGroup::new(None, row.cache, cache_group);
            group.chunks = remaining;
            group
          })
          .modules
          .insert(*module);
      }
      rows.push(row);
    }
    let row_seconds = row_started.elapsed().as_secs_f64();
    let seed_started = std::time::Instant::now();
    let bounds_by_key = live
      .iter()
      .map(|(key, row)| (key.clone(), SizeUpperBound::for_group(row, sizes)))
      .collect::<FxHashMap<_, _>>();
    let mut chunk_bounds = FxHashMap::<(u32, ChunkUkey), SizeUpperBound>::default();
    for (key, group) in &live {
      for chunk in &group.chunks {
        chunk_bounds
          .entry((group.cache_group_index, *chunk))
          .or_default()
          .include(&bounds_by_key[key]);
      }
    }
    // Any surviving source type must fit inside EVERY source chunk. These
    // sums overestimate available support and use the maximum possible source
    // count for minSizeReduction; even this bound failing proves infeasibility.
    let useful_chunks = chunk_bounds
      .into_iter()
      .filter_map(|((cache, chunk), upper)| {
        upper
          .may_fit(&self.cache_groups[cache as usize], indices.len())
          .then_some((cache, chunk))
      })
      .collect::<FxHashSet<_>>();
    let useful_residuals = live
      .iter()
      .filter_map(|(key, group)| {
        let minimum =
          (self.cache_groups[group.cache_group_index as usize].min_chunks as usize).max(1);
        (group
          .chunks
          .iter()
          .filter(|chunk| useful_chunks.contains(&(group.cache_group_index, **chunk)))
          .take(minimum)
          .count()
          >= minimum)
          .then_some(key.clone())
      })
      .collect::<FxHashSet<_>>();
    let original_postings = posting_index(
      rows
        .iter()
        .enumerate()
        .map(|(i, r)| (i, r.cache, &r.original)),
    );
    let original_indices = rows
      .iter()
      .enumerate()
      .map(|(i, row)| (row.key.clone(), i))
      .collect::<FxHashMap<_, _>>();
    let mut seeds = FxHashMap::<ModuleGroupKey, ResidualSeed<'_>>::default();
    let mut key_scratch = Vec::<u32>::new();
    let mut insert_projected = |cache: u32, chunks: &[ChunkUkey]| {
      if chunks.is_empty()
        || chunks.len() < self.cache_groups[cache as usize].min_chunks as usize
        || !chunks
          .iter()
          .all(|chunk| useful_chunks.contains(&(cache, *chunk)))
      {
        return;
      }
      // Use the same sorted-index hash as native get_key, with one reusable
      // buffer instead of allocating a key vector for every projection.
      key_scratch.clear();
      key_scratch.extend(chunks.iter().map(|chunk| indices[chunk]));
      key_scratch.sort_unstable();
      let mut hasher = FxHasher::default();
      for chunk in &key_scratch {
        chunk.hash(&mut hasher);
      }
      let key = ModuleGroupKey::Anonymous {
        cache_group_index: cache,
        chunks_key: hasher.finish(),
      };
      seeds
        .entry(key)
        .or_insert_with(|| ResidualSeed::Projected(SmallVec::from_slice(chunks)));
    };
    // Reuse native dense indexes. Cache a surviving row's bitmap only when a
    // dense projection is larger than scanning its words; most small rows never
    // allocate one. A native seed reuses one scratch bitmap across all its rows.
    let bitmap_words = indices
      .values()
      .max()
      .map_or(0, |index| *index as usize / 64 + 1);
    let mut chunks_by_index = vec![None; bitmap_words * 64];
    for (chunk, index) in indices {
      chunks_by_index[*index as usize] = Some(*chunk);
    }
    let mut residual_bits = FxHashMap::<ModuleGroupKey, Vec<u64>>::default();
    let mut seed_bits = Vec::<u64>::new();
    let mut projected = SmallVec::<[ChunkUkey; 4]>::new();
    let mut affected = SmallVec::<[ChunkUkey; 4]>::new();
    // Reuse the native seed-to-original support relationship. Project only
    // changed rows; wholly consumed rows still wake a formerly invalid seed.
    let mut retained_seeds = Vec::new();
    let mut visited_residuals = FxHashSet::default();
    for (seed_key, mut seed, cached_originals) in supports.seeds {
      let ModuleGroupKey::Anonymous {
        cache_group_index: cache,
        ..
      } = seed_key
      else {
        unreachable!()
      };
      visited_residuals.clear();
      seed_bits.clear();
      let min_chunks = (self.cache_groups[cache as usize].min_chunks as usize).max(1);
      if seed
        .iter()
        .filter(|chunk| useful_chunks.contains(&(cache, **chunk)))
        .take(min_chunks)
        .count()
        < min_chunks
      {
        continue;
      }
      let native_support = cached_originals.is_some();
      let posting = if let Some(originals) = cached_originals {
        originals
          .into_iter()
          .filter_map(|chunks_key| {
            original_indices
              .get(&ModuleGroupKey::Anonymous {
                cache_group_index: cache,
                chunks_key,
              })
              .copied()
          })
          .collect::<SmallVec<[usize; 4]>>()
      } else {
        let Some(posting) = shortest_posting(cache, seed.iter(), &original_postings) else {
          continue;
        };
        SmallVec::from_slice(posting)
      };
      let mut changed = false;
      for index in posting {
        let row = &rows[index];
        if !row.changed || (!native_support && !seed.is_subset(&row.original)) {
          continue;
        }
        changed = true;
        // At the minimum cardinality, every eligible intersection is already
        // either this native seed or a full residual row, both queued below.
        if seed.len() == min_chunks {
          continue;
        }
        for (residual, deleted) in &row.residuals {
          if !useful_residuals.contains(residual) {
            continue;
          }
          let remaining = &live[residual].chunks;
          if remaining.len() == min_chunks || !visited_residuals.insert(residual.clone()) {
            continue;
          }
          // S was supported by this original row R, so S intersect (R-D)
          // equals S-D. For sparse D, mutate S in place and restore exactly
          // the removed edges after recording the projection. No set clone.
          projected.clear();
          if deleted.len() <= seed.len().min(remaining.len()) {
            affected.clear();
            for chunk in deleted {
              if seed.remove(chunk) {
                affected.push(*chunk);
              }
            }
            if !affected.is_empty() && seed.len() != remaining.len() && seed.len() >= min_chunks {
              projected.extend(seed.iter().copied());
              insert_projected(cache, &projected);
            }
            // Restore before considering another residual or retaining the
            // native seed. Projections must remain independent of each other.
            for chunk in affected.drain(..) {
              seed.insert(chunk);
            }
          } else if seed.len().min(remaining.len()) > bitmap_words.saturating_mul(2) {
            if seed_bits.is_empty() {
              seed_bits.resize(bitmap_words, 0);
              for chunk in &seed {
                let index = indices[chunk] as usize;
                seed_bits[index / 64] |= 1 << (index % 64);
              }
            }
            let remaining_bits = residual_bits.entry(residual.clone()).or_insert_with(|| {
              let mut bits = vec![0u64; bitmap_words];
              for chunk in remaining {
                let index = indices[chunk] as usize;
                bits[index / 64] |= 1 << (index % 64);
              }
              bits
            });
            let length = seed_bits
              .iter()
              .zip(remaining_bits.iter())
              .map(|(seed, remaining)| (seed & remaining).count_ones() as usize)
              .sum::<usize>();
            if length == seed.len() || length == remaining.len() || length < min_chunks {
              continue;
            }
            for (word, (seed, remaining)) in seed_bits.iter().zip(remaining_bits.iter()).enumerate()
            {
              let mut kept = seed & remaining;
              while kept != 0 {
                let index = word * 64 + kept.trailing_zeros() as usize;
                projected.push(chunks_by_index[index].expect("native chunk index"));
                kept &= kept - 1;
              }
            }
            insert_projected(cache, &projected);
          } else {
            // Dense deletion can make the deletion list much larger than the
            // result. Walk the smaller surviving side, not a large complement.
            if remaining.len() < seed.len() {
              projected.extend(
                remaining
                  .iter()
                  .filter(|chunk| seed.contains(chunk))
                  .copied(),
              );
            } else {
              projected.extend(
                seed
                  .iter()
                  .filter(|chunk| remaining.contains(chunk))
                  .copied(),
              );
            }
            if projected.len() != seed.len()
              && projected.len() != remaining.len()
              && projected.len() >= min_chunks
            {
              insert_projected(cache, &projected);
            }
          }
        }
      }
      if changed
        && seed
          .iter()
          .all(|chunk| useful_chunks.contains(&(cache, *chunk)))
      {
        retained_seeds.push((seed_key, seed));
      }
    }
    for row in &rows {
      if row.changed {
        for residual in row.residuals.keys() {
          let chunks = &live[residual].chunks;
          if !chunks.is_empty()
            && chunks.len() >= self.cache_groups[row.cache as usize].min_chunks as usize
            && chunks
              .iter()
              .all(|chunk| useful_chunks.contains(&(row.cache, *chunk)))
          {
            // The live row already has this exact key and set. Reuse both.
            seeds
              .entry(residual.clone())
              .or_insert(ResidualSeed::Existing(chunks));
          }
        }
      }
    }
    for (key, seed) in retained_seeds {
      seeds.entry(key).or_insert(ResidualSeed::Native(seed));
    }
    let mut live = live.iter().collect::<Vec<_>>();
    live.sort_by(|a, b| a.0.cmp(b.0));
    let seed_seconds = seed_started.elapsed().as_secs_f64();
    let refill_started = std::time::Instant::now();
    let live_postings = posting_index(
      live
        .iter()
        .enumerate()
        .map(|(i, (_, r))| (i, r.cache_group_index, &r.chunks)),
    );
    let size_bounds = live
      .iter()
      .map(|(key, _)| &bounds_by_key[key])
      .collect::<Vec<_>>();
    let seed_count = seeds.len();
    let mut seeds = seeds.into_iter().collect::<Vec<_>>();
    seeds.sort_by(|a, b| a.0.cmp(&b.0));
    let mut refill_rows = 0usize;
    let mut size_pruned = 0usize;
    let mut filled_modules = 0usize;
    for (key, seed) in seeds {
      let ModuleGroupKey::Anonymous {
        cache_group_index: cache,
        ..
      } = key
      else {
        unreachable!()
      };
      let Some(posting) = shortest_posting(cache, seed.iter(), &live_postings) else {
        continue;
      };
      let mut selected = SmallVec::<[usize; 4]>::new();
      let mut upper = SizeUpperBound::default();
      for index in posting {
        refill_rows += 1;
        let row = &live[*index].1;
        if seed.is_subset(&row.chunks) {
          selected.push(*index);
          upper.include(size_bounds[*index]);
        }
      }
      if !upper.may_fit(&self.cache_groups[cache as usize], seed.len()) {
        size_pruned += 1;
        continue;
      }
      let mut group = ModuleGroup::new(None, cache, &self.cache_groups[cache as usize]);
      for index in selected {
        filled_modules += live[index].1.modules.len();
        group.modules.extend(live[index].1.modules.iter().copied());
      }
      if !group.modules.is_empty() {
        group.chunks = seed.into_chunks();
        candidates.insert(key, group);
      }
    }
    let refill_seconds = refill_started.elapsed().as_secs_f64();
    let fit_started = std::time::Instant::now();
    let raw_candidates = candidates.len();
    for candidate in candidates.values_mut() {
      candidate.prepare_modules_for_sizes_and_compare();
    }
    self.ensure_min_size_fit(&mut candidates, sizes);
    if audit {
      eprintln!(
        "[residual-supports] {{\"supportRows\":{},\"supportModules\":{},\"derivedSeeds\":{},\"refillRows\":{},\"sizePruned\":{},\"filledModuleAssociations\":{},\"rawCandidates\":{},\"eligibleCandidates\":{},\"rowSeconds\":{},\"seedSeconds\":{},\"refillSeconds\":{},\"fitSeconds\":{}}}",
        rows.len(),
        retained_modules,
        seed_count,
        refill_rows,
        size_pruned,
        filled_modules,
        raw_candidates,
        candidates.len(),
        row_seconds,
        seed_seconds,
        refill_seconds,
        fit_started.elapsed().as_secs_f64()
      );
    }
    candidates
  }
}

/// Each replacement copy must retain the union of the consumers of its removed
/// copies. Also check existing target modules when extending a reused target.
pub(super) fn placement_preserves_copies(
  placed: &ModuleChunkMap,
  source_chunks: &FxHashSet<ChunkUkey>,
  target: ChunkUkey,
  compilation: &Compilation,
) -> bool {
  let artifact = &compilation.build_chunk_graph_artifact;
  let target_chunk = artifact.chunk_by_ukey.expect_get(&target);
  let consumers = |chunks: &FxHashSet<ChunkUkey>| -> FxHashSet<ChunkGroupUkey> {
    chunks
      .iter()
      .flat_map(|chunk| {
        artifact
          .chunk_by_ukey
          .expect_get(chunk)
          .groups()
          .iter()
          .copied()
      })
      .collect()
  };
  let mut after = consumers(source_chunks);
  after.extend(target_chunk.groups().iter().copied());
  let mut modules = artifact
    .chunk_graph
    .get_chunk_modules_identifier(&target)
    .clone();
  match placed {
    ModuleChunkMap::Shared {
      modules: moving, ..
    } => modules.extend(moving.iter().copied()),
    ModuleChunkMap::ByModule(moving) => modules.extend(moving.keys().copied()),
  }
  let mut checked_shared = false;
  for module in &modules {
    let in_target = artifact.chunk_graph.is_module_in_chunk(module, target);
    if let Some(sources) = placed.get(module) {
      if matches!(placed, ModuleChunkMap::Shared { .. }) && !in_target {
        if checked_shared {
          continue;
        }
        checked_shared = true;
      }
      let mut before = consumers(sources);
      if in_target {
        before.extend(target_chunk.groups().iter().copied());
      }
      if before != after {
        return false;
      }
    } else {
      let before: FxHashSet<_> = target_chunk.groups().iter().copied().collect();
      if before != after {
        return false;
      }
    }
  }
  !modules.is_empty()
}
