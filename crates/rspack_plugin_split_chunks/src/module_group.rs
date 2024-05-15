use derivative::Derivative;
use rspack_core::{ChunkUkey, Module};
use rspack_identifier::IdentifierSet;
use rustc_hash::FxHashSet;

use crate::{common::SplitChunkSizes, CacheGroup};

/// `ModuleGroup` is a abstraction of middle step for splitting chunks.
///
/// `ModuleGroup` captures/contains a bunch of modules due to the `optimization.splitChunks` configuration.
///
/// `ModuleGroup` would be transform into `Chunk`s in the end.
///
///  A `ModuleGroup` would be transform into multiple `Chunk`s if the `name` dynamic computed
///
/// The original name of `ModuleGroup` is `ChunkInfoItem` borrowed from Webpack
#[derive(Derivative)]
#[derivative(Debug)]
pub(crate) struct ModuleGroup {
  /// the real index used for mapping the ModuleGroup to corresponding CacheGroup
  idx: CacheGroupIdx,
  pub modules: IdentifierSet,
  pub cache_group_index: usize,
  pub cache_group_priority: f64,
  pub cache_group_reuse_existing_chunk: bool,
  /// If the `ModuleGroup` is going to create a chunk, which will be named using `chunk_name`
  /// A module
  pub chunk_name: Option<String>,
  pub sizes: SplitChunkSizes,
  /// `Chunk`s which `Module`s in this ModuleGroup belong to
  #[derivative(Debug = "ignore")]
  pub chunks: FxHashSet<ChunkUkey>,
}

impl ModuleGroup {
  pub fn new(
    idx: CacheGroupIdx,
    chunk_name: Option<String>,
    cache_group_index: usize,
    cache_group: &CacheGroup,
  ) -> Self {
    Self {
      idx,
      modules: Default::default(),
      cache_group_index,
      cache_group_priority: cache_group.priority,
      cache_group_reuse_existing_chunk: cache_group.reuse_existing_chunk,
      sizes: Default::default(),
      chunks: Default::default(),
      chunk_name,
    }
  }

  pub fn add_module(&mut self, module: &dyn Module) {
    let old_len = self.modules.len();
    self.modules.insert(module.identifier());

    if self.modules.len() != old_len {
      module.source_types().iter().for_each(|ty| {
        let size = self.sizes.entry(*ty).or_default();
        *size += module.size(ty);
      });
    }
  }

  pub fn remove_module(&mut self, module: &dyn Module) {
    let old_len = self.modules.len();
    self.modules.remove(&module.identifier());

    if self.modules.len() != old_len {
      module.source_types().iter().for_each(|ty| {
        let size = self.sizes.entry(*ty).or_default();
        *size -= module.size(ty);
        *size = size.max(0.0)
      });
    }
  }

  pub fn get_cache_group<'a>(&self, cache_groups: &'a [CacheGroup]) -> &'a CacheGroup {
    &cache_groups[self.idx.0]
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct CacheGroupIdx(usize);

impl CacheGroupIdx {
  pub fn new(idx: usize) -> Self {
    Self(idx)
  }
}

pub(crate) fn compare_entries(a: &ModuleGroup, b: &ModuleGroup) -> f64 {
  // 1. by priority
  let diff_priority = a.cache_group_priority - b.cache_group_priority;
  if diff_priority != 0f64 {
    return diff_priority;
  }
  // 2. by number of chunks
  let diff_count = a.chunks.len() as f64 - b.chunks.len() as f64;
  if diff_count != 0f64 {
    return diff_count;
  }

  // 3. by size reduction
  let a_size_reduce = total_size(&a.sizes) * (a.chunks.len() - 1) as f64;
  let b_size_reduce = total_size(&b.sizes) * (b.chunks.len() - 1) as f64;
  let diff_size_reduce = a_size_reduce - b_size_reduce;
  if diff_size_reduce != 0f64 {
    return diff_size_reduce;
  }

  // 4. by number of modules (to be able to compare by identifier)
  let modules_a_len = a.modules.len();
  let modules_b_len = b.modules.len();
  let diff = modules_a_len as f64 - modules_b_len as f64;
  if diff != 0f64 {
    return diff;
  }

  let mut modules_a = a.modules.iter().collect::<Vec<_>>();
  let mut modules_b = b.modules.iter().collect::<Vec<_>>();
  modules_a.sort_unstable();
  modules_b.sort_unstable();

  loop {
    match (modules_a.pop(), modules_b.pop()) {
      (None, None) => break,
      (Some(a), Some(b)) => {
        let res = a.cmp(b);
        if !res.is_eq() {
          return res as i32 as f64;
        }
      }
      _ => unreachable!(),
    }
  }

  // 5. by cache group index
  b.cache_group_index as f64 - a.cache_group_index as f64
}

fn total_size(sizes: &SplitChunkSizes) -> f64 {
  let mut size = 0f64;
  for ty_size in sizes.0.values() {
    size += ty_size;
  }
  size
}
