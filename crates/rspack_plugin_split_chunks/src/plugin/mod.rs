mod chunk;
mod max_request;
mod max_size;
mod min_size;
mod module_group;

use std::{borrow::Cow, fmt::Debug};

use rspack_core::{ChunkUkey, Compilation, CompilationOptimizeChunks, Logger, Plugin};
use rspack_error::Result;
use rspack_hook::{plugin, plugin_hook};
use rustc_hash::FxHashMap;

use crate::common::FallbackCacheGroup;
use crate::module_group::ModuleGroup;
use crate::{CacheGroup, SplitChunkSizes};

type ModuleGroupMap = FxHashMap<String, ModuleGroup>;

#[derive(Debug)]
pub struct PluginOptions {
  pub cache_groups: Vec<CacheGroup>,
  pub fallback_cache_group: FallbackCacheGroup,
  pub hide_path_info: Option<bool>,
}

#[plugin]
pub struct SplitChunksPlugin {
  cache_groups: Box<[CacheGroup]>,
  fallback_cache_group: FallbackCacheGroup,
  hide_path_info: bool,
}

impl SplitChunksPlugin {
  pub fn new(options: PluginOptions) -> Self {
    tracing::debug!("Create `SplitChunksPlugin` with {:#?}", options);
    Self::new_inner(
      options.cache_groups.into(),
      options.fallback_cache_group,
      options.hide_path_info.unwrap_or(false),
    )
  }

  fn inner_impl(&self, compilation: &mut Compilation) -> Result<()> {
    let logger = compilation.get_logger(self.name());
    let start = logger.time("prepare module group map");
    let mut module_group_map = self.prepare_module_group_map(compilation)?;
    tracing::trace!("prepared module_group_map {:#?}", module_group_map);
    logger.time_end(start);

    let start = logger.time("ensure min size fit");
    self.ensure_min_size_fit(compilation, &mut module_group_map);
    logger.time_end(start);

    let start = logger.time("process module group map");
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
        let cache_group = module_group.get_cache_group(&self.cache_groups);

      let mut is_reuse_existing_chunk = false;
      let mut is_reuse_existing_chunk_with_all_modules = false;
      let new_chunk = self.get_corresponding_chunk(
        compilation,
        &mut module_group,
        &mut is_reuse_existing_chunk,
        &mut is_reuse_existing_chunk_with_all_modules,
      );

      let new_chunk_mut = new_chunk.as_mut(&mut compilation.chunk_by_ukey);
      tracing::trace!("{module_group_key}, get Chunk {} with is_reuse_existing_chunk: {is_reuse_existing_chunk:?} and {is_reuse_existing_chunk_with_all_modules:?}", new_chunk_mut.chunk_reason.join("~"));

      new_chunk_mut
        .chunk_reason
        .push(["(cache group: ", cache_group.key.as_str(), ")"].join(""));

      if let Some(filename) = &cache_group.filename {
        new_chunk_mut.filename_template = Some(filename.clone());
      }

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
            automatic_name_delimiter: cache_group.automatic_name_delimiter.clone(),
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
    logger.time_end(start);

    let start = logger.time("ensure max size fit");
    self.ensure_max_size_fit(compilation, max_size_setting_map)?;
    logger.time_end(start);

    Ok(())
  }
}

impl Debug for SplitChunksPlugin {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("SplitChunksPlugin").finish()
  }
}

#[plugin_hook(CompilationOptimizeChunks for SplitChunksPlugin, stage = Compilation::OPTIMIZE_CHUNKS_STAGE_ADVANCED)]
fn optimize_chunks(&self, compilation: &mut Compilation) -> Result<Option<bool>> {
  self.inner_impl(compilation)?;
  Ok(None)
}

impl Plugin for SplitChunksPlugin {
  fn name(&self) -> &'static str {
    "rspack.SplitChunksPlugin"
  }

  fn apply(
    &self,
    ctx: rspack_core::PluginContext<&mut rspack_core::ApplyContext>,
    _options: &mut rspack_core::CompilerOptions,
  ) -> Result<()> {
    ctx
      .context
      .compilation_hooks
      .optimize_chunks
      .tap(optimize_chunks::new(self));
    Ok(())
  }
}

#[derive(Debug)]
struct MaxSizeSetting {
  pub min_size: SplitChunkSizes,
  pub max_async_size: SplitChunkSizes,
  pub max_initial_size: SplitChunkSizes,
  pub automatic_name_delimiter: String,
}
