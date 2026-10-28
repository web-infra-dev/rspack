use std::{cmp::Ordering, fmt};

use derive_more::Debug;
use rspack_collections::{IdentifierMap, IdentifierSet};
use rspack_core::{ChunkUkey, ModuleIdentifier, SourceType};
use rustc_hash::FxHashSet;
use smallvec::SmallVec;

use crate::{
  CacheGroup,
  common::{ModuleChunkMap, ModuleSizes, SplitChunkSizes, chunk_mask},
};

pub(crate) struct IndexedCacheGroup<'a> {
  pub cache_group_index: u32,
  pub cache_group: &'a CacheGroup,
}

impl<'a> IndexedCacheGroup<'a> {
  pub fn compare_by_priority(&self, other: &Self) -> Ordering {
    self
      .cache_group
      .priority
      .partial_cmp(&other.cache_group.priority)
      .unwrap_or(Ordering::Equal)
  }

  pub fn compare_by_index(&self, other: &Self) -> Ordering {
    self.cache_group_index.cmp(&other.cache_group_index)
  }
}

#[derive(Debug)]
enum ModulesForCompare {
  Unsorted(Vec<ModuleIdentifier>),
  Sorted(Vec<ModuleIdentifier>),
}

impl Default for ModulesForCompare {
  fn default() -> Self {
    Self::Unsorted(Default::default())
  }
}

impl ModulesForCompare {
  fn as_slice(&self) -> &[ModuleIdentifier] {
    match self {
      Self::Unsorted(modules) | Self::Sorted(modules) => modules,
    }
  }

  fn prepare(&mut self, modules: Vec<ModuleIdentifier>) {
    if modules.is_empty() {
      return;
    }

    if matches!(self, Self::Unsorted(modules_for_compare) if modules_for_compare.is_empty()) {
      *self = Self::Unsorted(modules);
    }
  }

  fn sorted(&mut self) -> &[ModuleIdentifier] {
    if let Self::Unsorted(modules) = self {
      modules.sort_unstable_by_key(|module| module.precomputed_hash());
      *self = Self::Sorted(std::mem::take(modules));
    }

    let Self::Sorted(modules) = self else {
      unreachable!("modules for compare should be sorted");
    };
    modules
  }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub(crate) enum ModuleGroupKey {
  Named {
    cache_group_index: u32,
    chunk_name: String,
  },
  Anonymous {
    cache_group_index: u32,
    chunks_key: u64,
  },
}

impl fmt::Display for ModuleGroupKey {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::Named {
        cache_group_index,
        chunk_name,
      } => write!(
        f,
        "named(cache_group_index={cache_group_index}, chunk_name={chunk_name})"
      ),
      Self::Anonymous {
        cache_group_index,
        chunks_key,
      } => write!(
        f,
        "anonymous(cache_group_index={cache_group_index}, chunks_key={chunks_key:x})"
      ),
    }
  }
}

/// `ModuleGroup` is a abstraction of middle step for splitting chunks.
///
/// `ModuleGroup` captures/contains a bunch of modules due to the `optimization.splitChunks` configuration.
///
/// `ModuleGroup` would be transform into `Chunk`s in the end.
///
///  A `ModuleGroup` would be transform into multiple `Chunk`s if the `name` dynamic computed
///
/// The original name of `ModuleGroup` is `ChunkInfoItem` borrowed from Webpack
#[derive(Debug)]
enum ModuleGroupChunks {
  /// Anonymous groups are keyed by their exact chunk combination. `None` means the selected chunks
  /// are still identical to `ModuleGroup::chunks`; a snapshot is created only if that union is
  /// later mutated while choosing a reused destination.
  Shared(Option<FxHashSet<ChunkUkey>>),
  /// A named group can merge modules selected from different chunk combinations.
  ByModule(IdentifierMap<FxHashSet<ChunkUkey>>),
}

pub(crate) enum CandidateUpdate {
  Unchanged,
  Updated,
  Discard,
}

#[derive(Debug)]
pub(crate) struct ModuleGroup {
  pub modules: IdentifierSet,
  /// Packed `selectable` / `placement_changed` flags. A single byte keeps
  /// `ModuleGroup` within the padding next to the other small fields, so the
  /// struct (moved by sorts and `swap_remove`) does not grow.
  candidate_flags: u8,
  /// The chunks selected for each module. Anonymous groups structurally share their exact chunk
  /// combination instead of allocating one set per module.
  #[debug(skip)]
  module_chunks: ModuleGroupChunks,
  /// the real index used for mapping the ModuleGroup to corresponding CacheGroup
  pub cache_group_index: u32,
  pub cache_group_reuse_existing_chunk: bool,
  /// If the `ModuleGroup` is going to create a chunk, which will be named using `chunk_name`
  /// A module
  pub chunk_name: Option<String>,

  /// `Chunk`s which `Module`s in this ModuleGroup belong to
  #[debug(skip)]
  pub chunks: FxHashSet<ChunkUkey>,
  /// A conservative snapshot of the group's chunks before selection starts. Later operations only
  /// remove chunks, so retaining the original bits can add false positives but never false
  /// negatives to cleanup filtering.
  chunk_mask: u64,
  modules_for_compare: ModulesForCompare,
  pending_initial_sizes: bool,
  has_modules_without_source_types: bool,
  removed: Vec<ModuleIdentifier>,
  sizes: SplitChunkSizes,
  total_size: f64,
}

// Most consecutive module sizes have the same source type. Keep that running
// total in a local value instead of hashing the type for every module. Flush on
// type changes so insertion order and each floating-point operation stay intact.
struct SizeAccumulator<'a> {
  sizes: &'a mut SplitChunkSizes,
  cached: Option<(SourceType, f64)>,
}

impl SizeAccumulator<'_> {
  fn get(&mut self, ty: SourceType) -> &mut f64 {
    if self
      .cached
      .as_ref()
      .is_some_and(|(cached, _)| *cached != ty)
    {
      let (ty, size) = self.cached.take().expect("cached source type");
      self.sizes.insert(ty, size);
    }
    &mut self
      .cached
      .get_or_insert_with(|| (ty, *self.sizes.entry(ty).or_default()))
      .1
  }

  fn finish(self) {
    if let Some((ty, size)) = self.cached {
      self.sizes.insert(ty, size);
    }
  }
}

impl ModuleGroup {
  const SELECTABLE: u8 = 1 << 0;
  const PLACEMENT_CHANGED: u8 = 1 << 1;

  /// Only threshold-invalid candidates are skipped by selection. Cleanup can
  /// make them selectable again within this priority's existing queue.
  #[inline]
  pub fn selectable(&self) -> bool {
    self.candidate_flags & Self::SELECTABLE != 0
  }

  #[inline]
  pub fn set_selectable(&mut self, selectable: bool) {
    if selectable {
      self.candidate_flags |= Self::SELECTABLE;
    } else {
      self.candidate_flags &= !Self::SELECTABLE;
    }
  }

  #[inline]
  pub fn placement_changed(&self) -> bool {
    self.candidate_flags & Self::PLACEMENT_CHANGED != 0
  }

  #[inline]
  pub fn mark_placement_changed(&mut self) {
    self.candidate_flags |= Self::PLACEMENT_CHANGED;
  }

  pub fn new(chunk_name: Option<String>, cache_group_index: u32, cache_group: &CacheGroup) -> Self {
    let module_chunks = if chunk_name.is_some() {
      ModuleGroupChunks::ByModule(Default::default())
    } else {
      ModuleGroupChunks::Shared(None)
    };
    Self {
      modules: Default::default(),
      candidate_flags: Self::SELECTABLE,
      module_chunks,
      cache_group_index,
      cache_group_reuse_existing_chunk: cache_group.reuse_existing_chunk,
      sizes: Default::default(),
      chunks: Default::default(),
      chunk_mask: 0,
      modules_for_compare: Default::default(),
      chunk_name,
      pending_initial_sizes: false,
      has_modules_without_source_types: false,
      removed: Default::default(),
      total_size: 0.0,
    }
  }

  pub fn get_source_types_modules(
    &self,
    ty: &[SourceType],
    module_sizes: &ModuleSizes,
  ) -> IdentifierSet {
    // Most candidates never violate a source-type threshold. Materialize this
    // filtered membership only when needed instead of duplicating every
    // candidate's module set while calculating its size.
    self
      .modules
      .iter()
      .filter_map(|module| {
        let sizes = module_sizes.get(module).expect("should have module size");
        if ty.iter().any(|ty| sizes.contains_key(ty)) {
          Some(*module)
        } else {
          None
        }
      })
      .collect()
  }

  pub fn has_only_source_types(&self, source_types: &[SourceType]) -> bool {
    debug_assert!(!self.pending_initial_sizes);
    // Size keys are never removed. They conservatively cover every remaining
    // module, including after partial removals, without rescanning the modules.
    !self.has_modules_without_source_types && self.sizes.keys().all(|ty| source_types.contains(ty))
  }

  pub fn add_module(
    &mut self,
    module: ModuleIdentifier,
    chunks: impl IntoIterator<Item = ChunkUkey>,
  ) {
    self.modules.insert(module);
    let ModuleGroupChunks::ByModule(module_chunks) = &mut self.module_chunks else {
      unreachable!("add_module should only be used for named module groups");
    };
    let module_chunks = module_chunks.entry(module).or_default();
    for chunk in chunks {
      module_chunks.insert(chunk);
      self.chunks.insert(chunk);
    }
  }

  pub fn add_module_with_shared_chunks(
    &mut self,
    module: ModuleIdentifier,
    chunks: impl IntoIterator<Item = ChunkUkey>,
  ) {
    self.modules.insert(module);
    let ModuleGroupChunks::Shared(shared_chunks) = &mut self.module_chunks else {
      unreachable!("shared chunks should only be used for anonymous module groups");
    };
    if self.chunks.is_empty() {
      debug_assert!(shared_chunks.is_none());
      self.chunks.extend(chunks);
    }
  }

  pub fn remove_group_chunk(&mut self, chunk: &ChunkUkey) {
    if let ModuleGroupChunks::Shared(shared_chunks) = &mut self.module_chunks
      && shared_chunks.is_none()
    {
      *shared_chunks = Some(self.chunks.clone());
    }
    self.chunks.remove(chunk);
  }

  pub fn get_module_chunks(&self, module: &ModuleIdentifier) -> Option<&FxHashSet<ChunkUkey>> {
    if !self.modules.contains(module) {
      return None;
    }
    match &self.module_chunks {
      ModuleGroupChunks::Shared(Some(chunks)) => Some(chunks),
      ModuleGroupChunks::Shared(None) => Some(&self.chunks),
      ModuleGroupChunks::ByModule(module_chunks) => module_chunks.get(module),
    }
  }

  pub fn uses_shared_module_chunks(&self) -> bool {
    matches!(self.module_chunks, ModuleGroupChunks::Shared(_))
  }

  pub fn shared_module_chunks(&self) -> Option<&FxHashSet<ChunkUkey>> {
    match &self.module_chunks {
      ModuleGroupChunks::Shared(Some(chunks)) => Some(chunks),
      ModuleGroupChunks::Shared(None) => Some(&self.chunks),
      ModuleGroupChunks::ByModule(_) => None,
    }
  }

  pub fn remove_module(&mut self, module: ModuleIdentifier) {
    self.remove_modules([module]);
  }

  pub fn remove_modules(&mut self, modules: impl IntoIterator<Item = ModuleIdentifier>) {
    match &mut self.module_chunks {
      ModuleGroupChunks::Shared(_) => {
        for module in modules {
          if self.modules.remove(&module) {
            self.removed.push(module);
          }
        }
      }
      ModuleGroupChunks::ByModule(module_chunks) => {
        for module in modules {
          if self.modules.remove(&module) {
            module_chunks.remove(&module);
            self.removed.push(module);
          }
        }
      }
    }
  }

  pub fn remove_shared_modules(&mut self, modules: &IdentifierSet) {
    debug_assert!(self.uses_shared_module_chunks());
    if self.modules.len() > modules.len() {
      // The original intersection walks `modules` in this case. Remove while
      // walking the same set to avoid probing the group twice per match.
      self.remove_modules(modules.iter().copied());
    } else {
      // Preserve the smaller group's iteration order, erasing known buckets
      // directly instead of hashing each matching module a second time.
      self.modules.retain(|module| {
        if modules.contains(module) {
          self.removed.push(*module);
          false
        } else {
          true
        }
      });
    }
  }

  /// Shrinking can only reduce a source type's size/reduction. Keep mixed-type
  /// candidates only if removing another type could leave an eligible one.
  pub fn can_recover(&mut self, cache: &CacheGroup, sizes: &ModuleSizes, chunks: usize) -> bool {
    chunks >= (cache.min_chunks as usize).max(1)
      && Self::has_recoverable_size(self.get_sizes(sizes), cache, chunks)
  }

  fn has_recoverable_size(sizes: &SplitChunkSizes, cache: &CacheGroup, chunks: usize) -> bool {
    sizes.iter().any(|(ty, size)| {
      *size > 0.0
        && *size >= cache.min_size.get(ty).copied().unwrap_or_default()
        && *size * chunks as f64
          >= cache
            .min_size_reduction
            .get(ty)
            .copied()
            .unwrap_or_default()
    })
  }

  /// Choose which dimension to delete without materializing either alternative.
  /// The caller has found an overlap; preserve each source type's subtraction order.
  fn prefer_module_removal(
    &self,
    affected: impl Iterator<Item = ModuleIdentifier>,
    remaining_chunks: usize,
    cache: &CacheGroup,
    module_sizes: &ModuleSizes,
  ) -> bool {
    let mut remaining_sizes: SmallVec<[(SourceType, f64); 2]> =
      self.sizes.iter().map(|(ty, size)| (*ty, *size)).collect();
    for module in affected {
      let sizes = &module_sizes[&module];
      for (ty, remaining) in &mut remaining_sizes {
        *remaining = (*remaining - sizes.get(ty).copied().unwrap_or_default()).max(0.0);
      }
    }
    let reduction = |ty, size: f64, chunks: usize| {
      if size >= cache.min_size.get(ty).copied().unwrap_or_default()
        && size * chunks as f64
          >= cache
            .min_size_reduction
            .get(ty)
            .copied()
            .unwrap_or_default()
      {
        size * chunks.saturating_sub(1) as f64
      } else {
        0.0
      }
    };
    let mut keep_chunks = 0.0;
    let mut keep_modules = 0.0;
    for ((ty, size), (_, remaining)) in self.sizes.iter().zip(remaining_sizes) {
      keep_chunks += reduction(ty, remaining, self.chunks.len());
      keep_modules += reduction(ty, *size, remaining_chunks);
    }
    keep_chunks > keep_modules
  }

  /// Subtract committed placements from this candidate in the existing cleanup
  /// traversal. Keep one remaining rectangle; never fork or rebuild a candidate.
  pub fn update_candidate(
    &mut self,
    placed: &ModuleChunkMap,
    cache: &CacheGroup,
    module_sizes: &ModuleSizes,
    scratch: &mut Vec<ModuleIdentifier>,
  ) -> CandidateUpdate {
    let minimum = (cache.min_chunks as usize).max(1);
    if let ModuleGroupChunks::ByModule(sources) = &mut self.module_chunks {
      let mut changed = false;
      sources.retain(|module, chunks| {
        let Some(removed) = placed.get(module) else {
          return true;
        };
        let before = chunks.len();
        chunks.retain(|chunk| !removed.contains(chunk));
        changed |= chunks.len() != before;
        if chunks.len() < minimum {
          self.modules.remove(module);
          self.removed.push(*module);
          false
        } else {
          true
        }
      });
      if !changed {
        return CandidateUpdate::Unchanged;
      }
      if self.modules.is_empty() {
        return CandidateUpdate::Discard;
      }
      // A changed named group must not introduce consumers of a module that
      // was already extracted elsewhere. Narrow its existing support in place.
      let before = self.chunks.len();
      self
        .chunks
        .retain(|chunk| sources.values().all(|chunks| chunks.contains(chunk)));
      for chunks in sources.values_mut() {
        chunks.retain(|chunk| self.chunks.contains(chunk));
      }
      self.mark_placement_changed();
      if self.chunks.len() != before {
        self.chunk_mask = chunk_mask(self.chunks.iter());
      }
      return CandidateUpdate::Updated;
    }
    let mut changed = false;
    let mut chunks_changed = false;
    match placed {
      ModuleChunkMap::Shared { modules, chunks } => {
        let shrink_chunks = if self.chunks.len() <= minimum {
          // Any overlap invalidates the chunk remainder. Keep the original
          // first-hit check and module-deletion path without counting chunks.
          if self.chunks.is_disjoint(chunks) {
            return CandidateUpdate::Unchanged;
          }
          if self.modules.is_subset(modules) {
            return CandidateUpdate::Discard;
          }
          false
        } else {
          // Finish deferred size bookkeeping before borrowing both iterators.
          self.get_sizes(module_sizes);
          let mut overlap = self.chunks.intersection(chunks);
          if overlap.next().is_none() {
            return CandidateUpdate::Unchanged;
          }
          let mut affected = self.modules.intersection(modules).copied();
          let Some(first) = affected.next() else {
            return CandidateUpdate::Unchanged;
          };
          // Count only candidates that share a module. Both first-hit checks
          // continue their existing iterators instead of starting another scan.
          let remaining = self.chunks.len() - 1 - overlap.count();
          if remaining >= minimum && Self::has_recoverable_size(&self.sizes, cache, remaining) {
            !self.prefer_module_removal(
              std::iter::once(first).chain(affected),
              remaining,
              cache,
              module_sizes,
            )
          } else {
            if self.modules.is_subset(modules) {
              return CandidateUpdate::Discard;
            }
            false
          }
        };
        if shrink_chunks {
          self.chunks.retain(|chunk| !chunks.contains(chunk));
          self.mark_placement_changed();
          changed = true;
          chunks_changed = true;
        } else {
          // The original module deletion is the viable or more beneficial update.
          let before = self.modules.len();
          self.remove_shared_modules(modules);
          changed = self.modules.len() != before;
        }
      }
      ModuleChunkMap::ByModule(placed) => {
        scratch.extend(
          placed
            .keys()
            .filter(|module| self.modules.contains(module))
            .copied(),
        );
        scratch.sort_unstable_by_key(|module| (module.precomputed_hash(), *module));
        for module in scratch.iter().copied() {
          let removed = &placed[&module];
          let overlap = self.chunks.intersection(removed).count();
          if overlap == 0 {
            continue;
          }
          changed = true;
          let remaining = self.chunks.len() - overlap;
          if self.can_recover(cache, module_sizes, remaining)
            && !self.prefer_module_removal(std::iter::once(module), remaining, cache, module_sizes)
          {
            self.chunks.retain(|chunk| !removed.contains(chunk));
            self.mark_placement_changed();
            chunks_changed = true;
          } else {
            self.remove_module(module);
          }
        }
      }
    }
    if changed {
      if self.modules.is_empty() {
        return CandidateUpdate::Discard;
      }
      // Pending candidates have not selected a reused destination, so their
      // shared per-module support is exactly the updated candidate support.
      self.module_chunks = ModuleGroupChunks::Shared(None);
      if chunks_changed {
        self.chunk_mask = chunk_mask(self.chunks.iter());
      }
      CandidateUpdate::Updated
    } else {
      CandidateUpdate::Unchanged
    }
  }

  pub fn rebuild_chunks(&mut self) {
    let ModuleGroupChunks::ByModule(module_chunks) = &self.module_chunks else {
      return;
    };
    self.chunks.clear();
    self
      .chunks
      .extend(module_chunks.values().flatten().copied());
  }

  pub fn prepare_modules_for_sizes_and_compare(&mut self) {
    self.chunk_mask = chunk_mask(self.chunks.iter());
    let modules = self.modules.iter().copied().collect::<Vec<_>>();
    self.pending_initial_sizes = !modules.is_empty();
    self.modules_for_compare.prepare(modules);
  }

  pub fn sorted_modules_for_compare(&mut self) -> &[ModuleIdentifier] {
    self.modules_for_compare.sorted()
  }

  pub fn may_have_chunks_in_mask(&self, mask: u64) -> bool {
    self.chunk_mask & mask != 0
  }

  pub fn get_cache_group<'a>(&self, cache_groups: &'a [CacheGroup]) -> &'a CacheGroup {
    &cache_groups[self.cache_group_index as usize]
  }

  pub fn get_total_size(&self) -> f64 {
    if self.pending_initial_sizes || !self.removed.is_empty() {
      unreachable!("should update sizes before get total size");
    }
    self.total_size
  }

  pub fn get_sizes(&mut self, module_sizes: &ModuleSizes) -> &SplitChunkSizes {
    if !self.pending_initial_sizes && self.removed.is_empty() {
      return &self.sizes;
    }
    let mut sizes = SizeAccumulator {
      sizes: &mut self.sizes,
      cached: None,
    };
    if self.pending_initial_sizes {
      // The comparison snapshot is captured before any module is removed and
      // before the first size comparison. It also supplies the initial size
      // accumulation in the original iteration order without a second copy.
      for module in self.modules_for_compare.as_slice() {
        let module_sizes = module_sizes.get(module).expect("should have module size");
        self.has_modules_without_source_types |= module_sizes.is_empty();
        for (ty, s) in module_sizes.iter() {
          let size = sizes.get(*ty);
          *size += s;
          self.total_size += s;
        }
      }
      self.pending_initial_sizes = false;
    }
    if !self.removed.is_empty() {
      for module in self.removed.drain(..) {
        let module_sizes = module_sizes.get(&module).expect("should have module size");
        for (ty, s) in module_sizes.iter() {
          let size = sizes.get(*ty);
          *size -= s;
          *size = size.max(0.0);
          self.total_size -= s;
        }
      }
    }

    sizes.finish();
    &self.sizes
  }
}

pub(crate) fn compare_entries(
  (a_key, a): (&ModuleGroupKey, &mut ModuleGroup),
  (b_key, b): (&ModuleGroupKey, &mut ModuleGroup),
) -> f64 {
  // 1. by priority
  // no need to compare priority anymore because we already pick all cache groups with same priority
  // let diff_priority = a.cache_group_priority - b.cache_group_priority;
  // if diff_priority != 0f64 {
  //   return diff_priority;
  // }
  // 2. by number of chunks
  let a_chunks_len = a.chunks.len();
  let b_chunks_len = b.chunks.len();
  let diff_count = a_chunks_len as f64 - b_chunks_len as f64;
  if diff_count != 0f64 {
    return diff_count;
  }

  // 3. by size reduction
  let a_size_reduce = a.get_total_size() * (a_chunks_len - 1) as f64;
  let b_size_reduce = b.get_total_size() * (b_chunks_len - 1) as f64;
  let diff_size_reduce = a_size_reduce - b_size_reduce;
  if diff_size_reduce != 0f64 {
    return diff_size_reduce;
  }

  // 4. by cache group index
  let index_diff = b.cache_group_index as f64 - a.cache_group_index as f64;
  if index_diff != 0f64 {
    return index_diff;
  }

  // 5. by number of modules (to be able to compare by identifier)
  let modules_a_len = a.modules.len();
  let modules_b_len = b.modules.len();
  let diff = modules_a_len as f64 - modules_b_len as f64;
  if diff != 0f64 {
    return diff;
  }

  let mut modules_a = a.sorted_modules_for_compare().iter();
  let mut modules_b = b.sorted_modules_for_compare().iter();

  loop {
    match (modules_a.next(), modules_b.next()) {
      (None, None) => break,
      (Some(a), Some(b)) => {
        let res = a.cmp(b);
        if !res.is_eq() {
          return res as i32 as f64;
        }
      }
      (None, Some(_)) => return -1.0,
      (Some(_), None) => return 1.0,
    }
  }

  match a_key.cmp(b_key) {
    Ordering::Less => -1.0,
    Ordering::Equal => 0.0,
    Ordering::Greater => 1.0,
  }
}
