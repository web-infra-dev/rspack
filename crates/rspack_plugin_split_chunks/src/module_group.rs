use std::{cmp::Ordering, fmt};

use derive_more::Debug;
use rspack_collections::{IdentifierMap, IdentifierSet};
use rspack_core::{ChunkUkey, ModuleIdentifier, SourceType};
use rustc_hash::FxHashSet;

use crate::{
  CacheGroup,
  common::{ModuleSizes, SplitChunkSizes},
};

pub(crate) struct IndexedCacheGroup<'a> {
  pub cache_group_index: u32,
  pub cache_group: &'a CacheGroup,
  pub has_default_type_filter: bool,
  pub has_default_layer_filter: bool,
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

#[derive(Debug)]
pub(crate) struct ModuleGroup {
  pub modules: IdentifierSet,
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
  modules_for_compare: Vec<ModuleIdentifier>,
  removed: Vec<ModuleIdentifier>,
  sizes: SplitChunkSizes,
  total_size: f64,
}

impl ModuleGroup {
  pub fn new(chunk_name: Option<String>, cache_group_index: u32, cache_group: &CacheGroup) -> Self {
    let module_chunks = if chunk_name.is_some() {
      ModuleGroupChunks::ByModule(Default::default())
    } else {
      ModuleGroupChunks::Shared(None)
    };
    Self {
      modules: Default::default(),
      module_chunks,
      cache_group_index,
      cache_group_reuse_existing_chunk: cache_group.reuse_existing_chunk,
      sizes: Default::default(),
      chunks: Default::default(),
      modules_for_compare: Default::default(),
      chunk_name,
      removed: Default::default(),
      total_size: 0.0,
    }
  }

  pub fn get_source_types_modules(
    &self,
    ty: &[SourceType],
    module_sizes: &ModuleSizes,
  ) -> IdentifierSet {
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

  pub fn remove_shared_modules_in(&mut self, modules: &IdentifierSet) -> bool {
    debug_assert!(matches!(self.module_chunks, ModuleGroupChunks::Shared(_)));
    let removed_start = self.removed.len();
    if self.modules.len() > modules.len() {
      for module in modules {
        if self.modules.remove(module) {
          self.removed.push(*module);
        }
      }
    } else {
      let removed = &mut self.removed;
      self.modules.retain(|module| {
        if modules.contains(module) {
          removed.push(*module);
          false
        } else {
          true
        }
      });
    }
    self.removed.len() != removed_start
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

  pub fn rebuild_chunks(&mut self) {
    let ModuleGroupChunks::ByModule(module_chunks) = &self.module_chunks else {
      return;
    };
    self.chunks.clear();
    self
      .chunks
      .extend(module_chunks.values().flatten().copied());
  }

  pub fn prepare_modules_for_sizes_and_compare(&mut self, module_sizes: &ModuleSizes) {
    let mut modules = Vec::with_capacity(self.modules.len());
    for module in &self.modules {
      modules.push(*module);
      let module_sizes = module_sizes.get(module).expect("should have module size");
      for (ty, size) in module_sizes {
        *self.sizes.entry(*ty).or_default() += size;
        self.total_size += size;
      }
    }
    modules.sort_unstable_by_key(|module| module.precomputed_hash());
    self.modules_for_compare = modules;
    self.removed.reserve(self.modules.len());
  }

  pub fn sorted_modules_for_compare(&self) -> &[ModuleIdentifier] {
    &self.modules_for_compare
  }

  pub fn get_cache_group<'a>(&self, cache_groups: &'a [CacheGroup]) -> &'a CacheGroup {
    &cache_groups[self.cache_group_index as usize]
  }

  pub fn get_total_size(&self) -> f64 {
    if !self.removed.is_empty() {
      unreachable!("should update sizes before get total size");
    }
    self.total_size
  }

  pub fn get_sizes(&mut self, module_sizes: &ModuleSizes) -> &SplitChunkSizes {
    if !self.removed.is_empty() {
      for module in self.removed.drain(..) {
        let module_sizes = module_sizes.get(&module).expect("should have module size");
        for (ty, s) in module_sizes.iter() {
          let size = self
            .sizes
            .get_mut(ty)
            .expect("removed module source type should have group size");
          *size -= s;
          *size = size.max(0.0);
          self.total_size -= s;
        }
      }
    }

    &self.sizes
  }
}

pub(crate) fn is_better_entry(
  (a_key, a): (&ModuleGroupKey, &ModuleGroup),
  (b_key, b): (&ModuleGroupKey, &ModuleGroup),
) -> bool {
  // 1. by priority
  // no need to compare priority anymore because we already pick all cache groups with same priority
  // let diff_priority = a.cache_group_priority - b.cache_group_priority;
  // if diff_priority != 0f64 {
  //   return diff_priority;
  // }
  // 2. by number of chunks
  let a_chunks_len = a.chunks.len();
  let b_chunks_len = b.chunks.len();
  if a_chunks_len != b_chunks_len {
    return a_chunks_len > b_chunks_len;
  }

  // 3. by size reduction
  let a_size_reduce = a.get_total_size() * (a_chunks_len - 1) as f64;
  let b_size_reduce = b.get_total_size() * (b_chunks_len - 1) as f64;
  let diff_size_reduce = a_size_reduce - b_size_reduce;
  if diff_size_reduce != 0f64 {
    return diff_size_reduce > 0f64;
  }

  // 4. by cache group index
  if a.cache_group_index != b.cache_group_index {
    return a.cache_group_index < b.cache_group_index;
  }

  // 5. by number of modules (to be able to compare by identifier)
  let modules_a_len = a.modules.len();
  let modules_b_len = b.modules.len();
  if modules_a_len != modules_b_len {
    return modules_a_len > modules_b_len;
  }

  let mut modules_a = a.sorted_modules_for_compare().iter();
  let mut modules_b = b.sorted_modules_for_compare().iter();

  loop {
    match (modules_a.next(), modules_b.next()) {
      (None, None) => break,
      (Some(a), Some(b)) => {
        let res = a.cmp(b);
        if !res.is_eq() {
          return res.is_gt();
        }
      }
      (None, Some(_)) => return false,
      (Some(_), None) => return true,
    }
  }

  a_key > b_key
}
