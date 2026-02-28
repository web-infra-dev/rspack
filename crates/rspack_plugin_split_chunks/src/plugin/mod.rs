mod chunk;
mod max_request;
pub mod max_size;
pub mod min_size;
mod module_group;

use std::{borrow::Cow, cmp::Ordering, fmt::Debug};

use itertools::Itertools;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use rspack_collections::{DatabaseItem, IdentifierMap, UkeyMap, UkeySet};
use rspack_core::{ChunkUkey, Compilation, CompilationOptimizeChunks, Logger, Plugin};
use rspack_error::Result;
use rspack_hook::{plugin, plugin_hook};
use rspack_util::{fx_hash::FxIndexMap, tracing_preset::TRACING_BENCH_TARGET};
use tracing::instrument;

use crate::{
  CacheGroup, SplitChunkSizes,
  common::FallbackCacheGroup,
  get_module_sizes,
  module_group::{IndexedCacheGroup, ModuleGroup},
};

type ModuleGroupMap = FxIndexMap<String, ModuleGroup>;

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
  #[instrument(name = "Compilation:SplitChunks",target=TRACING_BENCH_TARGET, skip_all)]
  async fn inner_impl(&self, compilation: &mut Compilation) -> Result<()> {
    let logger = compilation.get_logger(self.name());
    let start = logger.time("prepare module data");

    let mut all_modules = compilation
      .get_module_graph()
      .modules_keys()
      .copied()
      .collect::<Vec<_>>();
    // Sort modules to ensure deterministic processing order
    all_modules.sort_unstable();

    let module_sizes = get_module_sizes(all_modules.par_iter().copied(), compilation);
    let module_chunks = Self::get_module_chunks(&all_modules, compilation);
    logger.time_end(start);

    let chunk_index_map: UkeyMap<ChunkUkey, u64> = {
      let mut ordered_chunks = compilation
        .build_chunk_graph_artifact
        .chunk_by_ukey
        .values()
        .collect::<Vec<_>>();

      ordered_chunks.sort_by_cached_key(|chunk| {
        // sort by (group.index, chunk index in group)
        let group = chunk
          .groups()
          .iter()
          .map(|group| {
            compilation
              .build_chunk_graph_artifact
              .chunk_group_by_ukey
              .expect_get(group)
          })
          .min_by(|group1, group2| group1.index.cmp(&group2.index))
          .expect("chunk should have at least one group");
        let chunk_index = group
          .chunks
          .iter()
          .position(|c| *c == chunk.ukey())
          .expect("chunk should be in its group");
        (group.index, chunk_index)
      });

      ordered_chunks
        .iter()
        .enumerate()
        .map(|(index, chunk)| (chunk.ukey(), index as u64 + 1))
        .collect()
    };

    let start = logger.time("prepare cache groups");
    let mut priority_cache_groups = vec![];

    for (priority, cache_groups) in &self
      .cache_groups
      .iter()
      .enumerate()
      .map(|v| IndexedCacheGroup {
        cache_group_index: v.0,
        cache_group: v.1,
      })
      .sorted_by(|a, b| match b.compare_by_priority(a) {
        Ordering::Equal => a.compare_by_index(b),
        v => v,
      })
      .chunk_by(|v| v.cache_group.priority)
    {
      priority_cache_groups.push((priority, cache_groups.into_iter().collect::<Vec<_>>()));
    }

    let mut max_size_setting_map: UkeyMap<ChunkUkey, MaxSizeSetting> = Default::default();
    let mut removed_module_chunks: IdentifierMap<UkeySet<ChunkUkey>> = IdentifierMap::default();

    let mut combinator = module_group::Combinator::default();

    if self
      .cache_groups
      .iter()
      .any(|cache_group| !cache_group.used_exports)
    {
      combinator.prepare_group_by_chunks(&all_modules, &module_chunks, &chunk_index_map);
    }

    if self
      .cache_groups
      .iter()
      .any(|cache_group| cache_group.used_exports)
    {
      combinator.prepare_group_by_used_exports(
        &all_modules,
        &compilation.exports_info_artifact,
        &compilation.build_chunk_graph_artifact.chunk_by_ukey,
        &module_chunks,
        &chunk_index_map,
      );
    }

    logger.time_end(start);

    let start = logger.time("process cache groups");
    let priority_len = priority_cache_groups.len();
    for (index, (_, cache_groups)) in priority_cache_groups.into_iter().enumerate() {
      let mut module_group_map = self
        .prepare_module_group_map(
          &combinator,
          &all_modules,
          cache_groups,
          &removed_module_chunks,
          compilation,
          &module_chunks,
          &chunk_index_map,
        )
        .await?;
      tracing::trace!("prepared module_group_map {:#?}", module_group_map);

      self.ensure_min_size_fit(&mut module_group_map, &module_sizes);

      while !module_group_map.is_empty() {
        let (module_group_key, mut module_group) =
          self.find_best_module_group(&mut module_group_map);

        tracing::trace!(
          "ModuleGroup({}) wins, {:?} `ModuleGroup` remains",
          module_group_key,
          module_group_map.len(),
        );
        let cache_group = module_group.get_cache_group(&self.cache_groups);

        let mut is_reuse_existing_chunk = false;
        let mut is_reuse_existing_chunk_with_all_modules = false;
        let new_chunk = self.get_corresponding_chunk(
          compilation,
          &mut module_group,
          &mut is_reuse_existing_chunk,
          &mut is_reuse_existing_chunk_with_all_modules,
        );

        let new_chunk_mut = compilation
          .build_chunk_graph_artifact
          .chunk_by_ukey
          .expect_get_mut(&new_chunk);
        tracing::trace!(
          "{module_group_key}, get Chunk {:?} with is_reuse_existing_chunk: {is_reuse_existing_chunk:?} and {is_reuse_existing_chunk_with_all_modules:?}",
          new_chunk_mut.chunk_reason()
        );

        if let Some(chunk_reason) = new_chunk_mut.chunk_reason_mut() {
          chunk_reason.push_str(&format!(" (cache group: {})", cache_group.key.as_str()));
          if let Some(chunk_name) = &module_group.chunk_name {
            chunk_reason.push_str(&format!(" (name: {chunk_name})"));
          }
        }

        if let Some(filename) = &cache_group.filename {
          new_chunk_mut.set_filename_template(Some(filename.clone()));
        }

        new_chunk_mut.add_id_name_hints(cache_group.id_hint.clone());

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
            tracing::trace!(
              "ModuleGroup({module_group_key}) is skipped. Reason: used_chunks_len({used_chunks_len:?}) < cache_group.min_chunks({:?})",
              cache_group.min_chunks
            );
            continue;
            // return;
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
          &module_sizes,
        );

        if index != priority_len - 1 {
          for module in module_group.modules.iter() {
            removed_module_chunks
              .entry(*module)
              .or_default()
              .extend(module_group.chunks.iter().copied());
          }
        }
      }
    }
    logger.time_end(start);

    let start = logger.time("ensure max size fit");
    self
      .ensure_max_size_fit(compilation, &max_size_setting_map)
      .await?;
    logger.time_end(start);

    rayon::spawn(move || drop(combinator));

    Ok(())
  }
}

impl Debug for SplitChunksPlugin {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("SplitChunksPlugin").finish()
  }
}

#[plugin_hook(CompilationOptimizeChunks for SplitChunksPlugin, stage = Compilation::OPTIMIZE_CHUNKS_STAGE_ADVANCED)]
async fn optimize_chunks(&self, compilation: &mut Compilation) -> Result<Option<bool>> {
  self.inner_impl(compilation).await?;
  compilation
    .build_chunk_graph_artifact
    .chunk_graph
    .generate_dot(compilation, "after-split-chunks")
    .await;
  Ok(None)
}

impl Plugin for SplitChunksPlugin {
  fn name(&self) -> &'static str {
    "rspack.SplitChunksPlugin"
  }

  fn apply(&self, ctx: &mut rspack_core::ApplyContext<'_>) -> Result<()> {
    ctx
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
