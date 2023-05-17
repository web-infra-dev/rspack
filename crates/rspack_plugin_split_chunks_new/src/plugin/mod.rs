mod chunk;
mod max_request;
mod max_size;
mod min_size;
mod module_group;

use std::{borrow::Cow, fmt::Debug};

use rspack_core::{ChunkUkey, Compilation, Plugin};
use rustc_hash::FxHashMap;

use crate::{
  cache_group::CacheGroup, common::FallbackCacheGroup, module_group::ModuleGroup, SplitChunkSizes,
};

type ModuleGroupMap = FxHashMap<String, ModuleGroup>;

#[derive(Debug)]
pub struct PluginOptions {
  pub cache_groups: Vec<CacheGroup>,
  pub fallback_cache_group: FallbackCacheGroup,
}

pub struct SplitChunksPlugin {
  cache_groups: Box<[CacheGroup]>,
  fallback_cache_group: FallbackCacheGroup,
}

impl SplitChunksPlugin {
  pub fn new(options: PluginOptions) -> Self {
    tracing::debug!("Create `SplitChunksPlugin` with {:#?}", options);
    Self {
      cache_groups: options.cache_groups.into(),
      fallback_cache_group: options.fallback_cache_group,
    }
  }

  async fn inner_impl(&self, compilation: &mut Compilation) {
    let mut module_group_map = self.prepare_module_group_map(compilation).await;
    tracing::trace!("prepared module_group_map {:#?}", module_group_map);

    self.ensure_min_size_fit(compilation, &mut module_group_map);

    let mut max_size_setting_map: FxHashMap<ChunkUkey, MaxSizeSetting> = Default::default();

    while !module_group_map.is_empty() {
      let (module_group_key, mut module_group) = self.find_best_module_group(&mut module_group_map);
      tracing::trace!(
        "ModuleGroup({}) wins, {:?} `ModuleGroup` remains",
        module_group_key,
        module_group_map.len(),
      );
      let process_span = tracing::trace_span!("Process ModuleGroup({})", module_group_key);

      process_span.in_scope(|| {
        let cache_group = &self.cache_groups[module_group.cache_group_index];

      let mut is_reuse_existing_chunk = false;
      let mut is_reuse_existing_chunk_with_all_modules = false;
      let new_chunk = self.get_corresponding_chunk(
        compilation,
        &mut module_group,
        &mut is_reuse_existing_chunk,
        &mut is_reuse_existing_chunk_with_all_modules,
      );

      let new_chunk_mut = new_chunk.as_mut(&mut compilation.chunk_by_ukey);
      tracing::trace!("{module_group_key}, get Chunk {} with is_reuse_existing_chunk: {is_reuse_existing_chunk:?} and {is_reuse_existing_chunk_with_all_modules:?}", new_chunk_mut.chunk_reasons.join("~"));

      new_chunk_mut
        .chunk_reasons
        .push(["(cache group: ", cache_group.key.as_str(), ")"].join(""));

      new_chunk_mut
        .id_name_hints
        .insert(cache_group.id_hint.clone());

      if is_reuse_existing_chunk {
        // The chunk is not new but created in code splitting. We need remove `new_chunk` since we would remove
        // modules in this `Chunk/ModuleGroup` from other chunks. Other chunks is stored in `ModuleGroup.chunks`.
        module_group.chunks.remove(&new_chunk);
      }

      let mut used_chunks = Cow::Borrowed(&module_group.chunks);

      self.ensure_max_request_fit(compilation, cache_group, &mut used_chunks);

      if used_chunks.len() != module_group.chunks.len() {
        // There are some chunks removed by `ensure_max_request_fit`
        let used_chunks_len = if is_reuse_existing_chunk {
          used_chunks.len() + 1
        } else {
          used_chunks.len()
        };

        if used_chunks_len < cache_group.min_chunks as usize {
          // `min_size` is not satisfied, ignore this invalid `ModuleGroup`
          tracing::trace!("ModuleGroup({module_group_key}) is skipped. Reason: used_chunks_len({used_chunks_len:?}) < cache_group.min_chunks({:?})", cache_group.min_chunks);
          return;
        }
      }

      if !cache_group.max_initial_size.is_empty() || !cache_group.max_async_size.is_empty() {
        max_size_setting_map.insert(
          new_chunk,
          MaxSizeSetting {
            min_size: cache_group.min_size.clone(),
            max_async_size: cache_group.max_async_size.clone(),
            max_initial_size: cache_group.max_initial_size.clone(),
          },
        );
      }

      self.move_modules_to_new_chunk_and_remove_from_old_chunks(
        &module_group,
        new_chunk,
        &used_chunks,
        compilation,
      );

      self.split_from_original_chunks(&module_group, &used_chunks, new_chunk, compilation);

      self.remove_all_modules_from_other_module_groups(
        &module_group,
        &mut module_group_map,
        &used_chunks,
        compilation,
      );
      })
    }

    self.ensure_max_size_fit(compilation, max_size_setting_map);
  }
}

impl Debug for SplitChunksPlugin {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("SplitChunksPlugin").finish()
  }
}

#[async_trait::async_trait]
impl Plugin for SplitChunksPlugin {
  async fn optimize_chunks(
    &mut self,
    _ctx: rspack_core::PluginContext,
    args: rspack_core::OptimizeChunksArgs<'_>,
  ) -> rspack_core::PluginOptimizeChunksOutput {
    // use std::time::Instant;
    // let start = Instant::now();
    self.inner_impl(args.compilation).await;

    // let duration = start.elapsed();
    // tracing::trace!("SplitChunksPlugin is: {:?}", duration);
    Ok(())
  }
}

#[derive(Debug)]
struct MaxSizeSetting {
  pub min_size: SplitChunkSizes,
  pub max_async_size: SplitChunkSizes,
  pub max_initial_size: SplitChunkSizes,
}
