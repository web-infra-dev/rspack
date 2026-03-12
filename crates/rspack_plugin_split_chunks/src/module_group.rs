use std::cmp::Ordering;

use derive_more::Debug;
use itertools::Itertools;
use rspack_collections::IdentifierSet;
use rspack_core::{ChunkUkey, ModuleIdentifier, SourceType};
use rustc_hash::{FxHashMap, FxHashSet};

use crate::{
  CacheGroup,
  common::{ModuleSizes, SplitChunkSizes},
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
pub(crate) struct ModuleGroup {
  pub modules: IdentifierSet,
  /// the real index used for mapping the ModuleGroup to corresponding CacheGroup
  pub cache_group_index: u32,
  pub cache_group_reuse_existing_chunk: bool,
  /// If the `ModuleGroup` is going to create a chunk, which will be named using `chunk_name`
  /// A module
  pub chunk_name: Option<String>,

  pub source_types_modules: FxHashMap<SourceType, IdentifierSet>,
  /// `Chunk`s which `Module`s in this ModuleGroup belong to
  #[debug(skip)]
  pub chunks: FxHashSet<ChunkUkey>,
  added: Vec<ModuleIdentifier>,
  removed: Vec<ModuleIdentifier>,
  sizes: SplitChunkSizes,
  total_size: f64,
}

impl ModuleGroup {
  pub fn new(chunk_name: Option<String>, cache_group_index: u32, cache_group: &CacheGroup) -> Self {
    Self {
      modules: Default::default(),
      cache_group_index,
      cache_group_reuse_existing_chunk: cache_group.reuse_existing_chunk,
      sizes: Default::default(),
      source_types_modules: Default::default(),
      chunks: Default::default(),
      chunk_name,
      added: Default::default(),
      removed: Default::default(),
      total_size: 0.0,
    }
  }

  pub fn collect_source_types_modules(
    &self,
    ty: &[SourceType],
    module_sizes: &ModuleSizes,
  ) -> Vec<ModuleIdentifier> {
    // if there is only one source type, we can just use the `source_types_modules` directly
    // instead of iterating over all modules
    if ty.len() == 1 {
      self
        .source_types_modules
        .get(ty.first().expect("should have at least one source type"))
        .map(|modules| modules.iter().copied().collect())
        .unwrap_or_default()
    } else {
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
  }

  pub fn add_module(&mut self, module: ModuleIdentifier) {
    if self.modules.insert(module) {
      self.added.push(module);
    }
  }

  pub fn remove_module(&mut self, module: ModuleIdentifier) {
    if self.modules.remove(&module) {
      self.removed.push(module);
    }
  }

  pub fn get_cache_group<'a>(&self, cache_groups: &'a [CacheGroup]) -> &'a CacheGroup {
    &cache_groups[self.cache_group_index as usize]
  }

  pub fn ordered_module_identifiers(&self) -> Vec<ModuleIdentifier> {
    self
      .modules
      .iter()
      .copied()
      .sorted_unstable_by(
        |a, b| match a.precomputed_hash().cmp(&b.precomputed_hash()) {
          Ordering::Equal => a.cmp(b),
          other => other,
        },
      )
      .collect()
  }

  pub fn get_total_size(&self) -> f64 {
    if !self.added.is_empty() || !self.removed.is_empty() {
      unreachable!("should update sizes before get total size");
    }
    self.total_size
  }

  pub fn get_sizes(&mut self, module_sizes: &ModuleSizes) -> &SplitChunkSizes {
    if !self.added.is_empty() {
      let added = std::mem::take(&mut self.added);
      for module in added {
        let module_sizes = module_sizes.get(&module).expect("should have module size");
        for (ty, s) in module_sizes.iter() {
          let size = self.sizes.entry(*ty).or_default();
          *size += s;
          self.total_size += s;
          self
            .source_types_modules
            .entry(*ty)
            .or_default()
            .insert(module);
        }
      }
    }
    if !self.removed.is_empty() {
      let removed = std::mem::take(&mut self.removed);
      for module in removed {
        let module_sizes = module_sizes.get(&module).expect("should have module size");
        for (ty, s) in module_sizes.iter() {
          let size = self.sizes.entry(*ty).or_default();
          *size -= s;
          *size = size.max(0.0);
          self.total_size -= s;
          self
            .source_types_modules
            .entry(*ty)
            .or_default()
            .remove(&module);
        }
      }
    }

    &self.sizes
  }
}
