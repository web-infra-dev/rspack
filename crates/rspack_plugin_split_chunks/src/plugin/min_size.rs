use rayon::prelude::*;
use rspack_collections::IdentifierMap;
use rspack_core::{Compilation, ModuleIdentifier, SourceType};
use rustc_hash::FxHashMap;

use super::ModuleGroupMap;
use crate::{
  CacheGroup, SplitChunkSizes, SplitChunksPlugin, common::ModuleSizes, module_group::ModuleGroup,
};

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

  /// Return `true` if the `ModuleGroup` become empty.
  pub(crate) fn remove_min_size_violating_modules(
    module_group_key: &str,
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

        if Self::remove_min_size_violating_modules(
          module_group_key,
          module_group,
          cache_group,
          module_sizes,
        ) || !Self::check_min_size_reduction(
          &module_group.get_sizes(module_sizes),
          &cache_group.min_size_reduction,
          module_group.chunks.len(),
        ) {
          Some(module_group_key.clone())
        } else {
          None
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

  pub(crate) fn get_module_sizes(
    all_modules: &[ModuleIdentifier],
    compilation: &Compilation,
  ) -> ModuleSizes {
    let module_graph = compilation.get_module_graph();
    all_modules
      .par_iter()
      .map(|module| {
        let module = module_graph
          .module_by_identifier(module)
          .expect("should have module");
        let sizes = module
          .source_types(module_graph)
          .iter()
          .map(|ty| (*ty, module.size(Some(ty), Some(compilation))))
          .collect::<FxHashMap<_, _>>();
        (module.identifier(), sizes)
      })
      .collect::<IdentifierMap<_>>()
  }
}
