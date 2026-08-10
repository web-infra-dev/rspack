use rayon::prelude::*;
use rspack_collections::{IdentifierMap, IdentifierSet};
use rspack_core::{ChunkUkey, ModuleIdentifier, SourceType};
use rustc_hash::FxHashSet;

use super::ModuleGroupMap;
use crate::{
  CacheGroup, SplitChunkSizes, SplitChunksPlugin, common::ModuleSizes, module_group::ModuleGroup,
};

pub trait ModulesContainer {
  fn get_sizes(&mut self, module_sizes: &ModuleSizes) -> &SplitChunkSizes;
  fn get_source_types_modules(
    &self,
    source_types: &[SourceType],
    module_sizes: &ModuleSizes,
  ) -> IdentifierSet;
  fn remove_module(&mut self, module: ModuleIdentifier);
  fn modules(&self) -> &IdentifierSet;
}

impl ModulesContainer for ModuleGroup {
  fn get_sizes(&mut self, module_sizes: &ModuleSizes) -> &SplitChunkSizes {
    ModuleGroup::get_sizes(self, module_sizes)
  }

  fn get_source_types_modules(
    &self,
    source_types: &[SourceType],
    module_sizes: &ModuleSizes,
  ) -> IdentifierSet {
    ModuleGroup::get_source_types_modules(self, source_types, module_sizes)
  }

  fn remove_module(&mut self, module: ModuleIdentifier) {
    ModuleGroup::remove_module(self, module);
  }

  fn modules(&self) -> &IdentifierSet {
    &self.modules
  }
}

/// Return `true` if the `ModuleGroup` become empty.
pub(crate) fn remove_min_size_violating_modules<T: std::fmt::Display>(
  module_group_key: &T,
  module_group: &mut ModuleGroup,
  cache_group: &CacheGroup,
  module_sizes: &ModuleSizes,
) -> bool {
  // Find out what `SourceType`'s size is not fit the min_size
  let violating_source_types: Box<[SourceType]> = module_group
  .get_sizes(module_sizes)
  .iter()
  .filter_map(|(module_group_ty, module_group_ty_size)| {
    let cache_group_ty_min_size = cache_group
      .min_size
      .get(module_group_ty)
      .copied()
      .unwrap_or_default();

    if *module_group_ty_size < cache_group_ty_min_size {
      tracing::trace!(
        "ModuleGroup({}) have violating SourceType({:?}). Reason: module_group_ty_size({:?}) < CacheGroup({}).min_size({:?})",
        module_group_key,
        module_group_ty,
        module_group_ty_size,
        cache_group.key,
        cache_group_ty_min_size,
      );
      Some(*module_group_ty)
    } else {
      None
    }
  })
  .collect::<Box<[_]>>();

  if violating_source_types.is_empty() {
    return module_group.modules.is_empty();
  }

  // Remove modules having violating SourceType
  let violating_modules =
    module_group.get_source_types_modules(&violating_source_types, module_sizes);

  // question: After removing violating modules, the size of other `SourceType`s of this `ModuleGroup`
  // may not fit again. But Webpack seems ignore this case. Not sure if it is on purpose.
  for violating_module in violating_modules {
    module_group.remove_module(violating_module);
  }

  module_group.modules.is_empty()
}

impl SplitChunksPlugin {
  pub(crate) fn check_min_size_reduction(
    sizes: &SplitChunkSizes,
    min_size_reduction: &SplitChunkSizes,
    chunk_count: usize,
  ) -> bool {
    for (ty, min_reduction_size) in min_size_reduction.iter() {
      if *min_reduction_size == 0.0f64 {
        continue;
      }

      let Some(size) = sizes.get(ty) else {
        continue;
      };
      if *size == 0.0f64 {
        continue;
      }
      if size * (chunk_count as f64) < *min_reduction_size {
        return false;
      }
    }

    true
  }

  /// Calculate the actual reduction from module-source edges. A reused destination can be part of
  /// `module_chunks`, but it keeps the module and therefore contributes no size reduction.
  pub(crate) fn check_min_size_reduction_for_module_chunks(
    module_chunks: &IdentifierMap<FxHashSet<ChunkUkey>>,
    destination_chunk: ChunkUkey,
    module_sizes: &ModuleSizes,
    min_size_reduction: &SplitChunkSizes,
  ) -> bool {
    for (ty, min_reduction_size) in min_size_reduction.iter() {
      if *min_reduction_size == 0.0f64 {
        continue;
      }

      let mut has_non_zero_size = false;
      let mut total_size_reduction = 0.0;
      for (module, chunks) in module_chunks {
        let Some(size) = module_sizes
          .get(module)
          .and_then(|module_sizes| module_sizes.get(ty))
        else {
          continue;
        };
        if *size == 0.0f64 {
          continue;
        }

        has_non_zero_size = true;
        let source_chunk_count = chunks
          .iter()
          .filter(|chunk| **chunk != destination_chunk)
          .count();
        total_size_reduction += size * source_chunk_count as f64;
      }

      if has_non_zero_size && total_size_reduction < *min_reduction_size {
        return false;
      }
    }

    true
  }

  /// Affected by `splitChunks.minSize`/`splitChunks.cacheGroups.{cacheGroup}.minSize`
  // #[tracing::instrument(skip_all)]
  pub(crate) fn ensure_min_size_fit(
    &self,
    module_group_map: &mut ModuleGroupMap,
    module_sizes: &ModuleSizes,
  ) {
    let invalidated_module_groups = module_group_map
      .par_iter_mut()
      .filter_map(|(module_group_key, module_group)| {
        let cache_group = module_group.get_cache_group(&self.cache_groups);
        // Fast path
        if cache_group.min_size.is_empty() {
          let _ = module_group.get_sizes(module_sizes);
          tracing::debug!(
            "ModuleGroup({}) skips `minSize` checking. Reason: min_size of CacheGroup({}) is empty",
            module_group_key,
            cache_group.key,
          );
          return None;
        }

        if remove_min_size_violating_modules(
          module_group_key,
          module_group,
          cache_group,
          module_sizes,
        ) {
          Some(module_group_key.clone())
        } else {
          let chunks_len = module_group.chunks.len();
          if !Self::check_min_size_reduction(
            module_group.get_sizes(module_sizes),
            &cache_group.min_size_reduction,
            chunks_len,
          ) {
            Some(module_group_key.clone())
          } else {
            None
          }
        }
      })
      .collect::<Vec<_>>();

    invalidated_module_groups.into_iter().for_each(|key| {
      tracing::debug!(
        "ModuleGroup({}) is removed. Reason: empty modules cause by `minSize` checking",
        key,
      );
      module_group_map.swap_remove(&key);
    });
  }
}
