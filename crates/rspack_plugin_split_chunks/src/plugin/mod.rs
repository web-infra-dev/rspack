mod chunk;
mod max_request;
pub mod max_size;
pub mod min_size;
mod module_group;

use std::{borrow::Cow, cmp::Ordering, fmt::Debug};

use itertools::Itertools;
use rayon::iter::{
  IndexedParallelIterator, IntoParallelRefIterator, IntoParallelRefMutIterator, ParallelIterator,
};
use rspack_collections::{IdentifierMap, IdentifierSet};
use rspack_core::{ChunkUkey, Compilation, CompilationOptimizeChunks, Logger, Plugin};
use rspack_error::Result;
use rspack_hook::{plugin, plugin_hook};
use rspack_util::{fx_hash::FxIndexMap, tracing_preset::TRACING_BENCH_TARGET};
use rustc_hash::{FxHashMap, FxHashSet};
use tracing::instrument;

use crate::{
  CacheGroup, SplitChunkSizes,
  common::FallbackCacheGroup,
  get_module_sizes,
  module_group::{IndexedCacheGroup, ModuleGroup, ModuleGroupKey},
};

type ModuleGroupMap = FxIndexMap<ModuleGroupKey, ModuleGroup>;

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
    // Sort modules to ensure deterministic processing order.
    // Use the precomputed identifier hash first to avoid repeated long string comparisons.
    all_modules.sort_unstable_by_key(|module| (module.precomputed_hash(), *module));

    let module_sizes = get_module_sizes(all_modules.par_iter().copied(), compilation);
    let module_chunks = Self::get_module_chunks(&all_modules, compilation);
    logger.time_end(start);

    let chunk_index_map: FxHashMap<ChunkUkey, u32> = {
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
        .map(|(index, chunk)| {
          (
            chunk.ukey(),
            u32::try_from(index + 1).expect("chunk index should fit in u32"),
          )
        })
        .collect()
    };

    let start = logger.time("prepare cache groups");
    let mut priority_cache_groups = vec![];

    for (priority, cache_groups) in &self
      .cache_groups
      .iter()
      .enumerate()
      .map(|v| IndexedCacheGroup {
        cache_group_index: u32::try_from(v.0).expect("cache group index should fit in u32"),
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

    let mut max_size_setting_map: FxHashMap<ChunkUkey, MaxSizeSetting> = Default::default();
    let mut removed_module_chunks: IdentifierMap<FxHashSet<ChunkUkey>> = IdentifierMap::default();

    logger.time_end(start);

    let start = logger.time("process cache groups");
    let priority_len = priority_cache_groups.len();
    for (index, (_, cache_groups)) in priority_cache_groups.into_iter().enumerate() {
      // A higher-priority cache group consumes module-chunk edges, not the whole module. Build the
      // combinations for this priority from the original chunk sets minus the consumed edges so a
      // lower-priority cache group can still group a newly formed residual chunk set. Do not read
      // the current chunk graph here because it also contains chunks created by earlier splits.
      let available_module_chunks = if removed_module_chunks.is_empty() {
        Cow::Borrowed(&module_chunks)
      } else {
        Cow::Owned(
          all_modules
            .par_iter()
            .enumerate()
            .map(|(module_index, module)| {
              let chunks = module_chunks
                .get(module_index)
                .expect("should have module chunks");
              if let Some(removed_chunks) = removed_module_chunks.get(module) {
                chunks.difference(removed_chunks).copied().collect()
              } else {
                chunks.clone()
              }
            })
            .collect(),
        )
      };

      let mut combinator = module_group::Combinator::default();
      let non_used_exports_min_chunks = cache_groups
        .iter()
        .filter(|cache_group| !cache_group.cache_group.used_exports)
        .map(|cache_group| cache_group.cache_group.min_chunks as usize)
        .min();

      if let Some(min_chunks) = non_used_exports_min_chunks {
        combinator.prepare_group_by_chunks(
          &all_modules,
          available_module_chunks.as_ref(),
          &chunk_index_map,
          min_chunks,
        );
      }

      if cache_groups
        .iter()
        .any(|cache_group| cache_group.cache_group.used_exports)
      {
        combinator.prepare_group_by_used_exports(
          &all_modules,
          &compilation.exports_info_artifact,
          &compilation.build_chunk_graph_artifact.chunk_by_ukey,
          available_module_chunks.as_ref(),
          &chunk_index_map,
        );
      }

      let mut module_group_map = self
        .prepare_module_group_map(
          &combinator,
          &all_modules,
          cache_groups,
          compilation,
          available_module_chunks.as_ref(),
          &chunk_index_map,
        )
        .await?;
      rayon::spawn(move || drop(combinator));
      tracing::trace!("prepared module_group_map {:#?}", module_group_map);

      module_group_map
        .par_iter_mut()
        .for_each(|(_, module_group)| module_group.prepare_modules_for_sizes_and_compare());
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

        tracing::trace!(
          "{module_group_key}, get Chunk {:?} with is_reuse_existing_chunk: {is_reuse_existing_chunk:?} and {is_reuse_existing_chunk_with_all_modules:?}",
          compilation
            .build_chunk_graph_artifact
            .chunk_by_ukey
            .expect_get(&new_chunk)
            .chunk_reason()
        );

        if is_reuse_existing_chunk {
          // The chunk is not new but created in code splitting. We need remove `new_chunk` since we would remove
          // modules in this `Chunk/ModuleGroup` from other chunks. Other chunks is stored in `ModuleGroup.chunks`.
          module_group.chunks.remove(&new_chunk);
        }

        // If the module group size exceeds enforceSizeThreshold, skip maxRequest constraints
        // https://webpack.js.org/plugins/split-chunks-plugin/#splitchunksenforcesizethreshold
        let enforce_size_exceeded = !cache_group.enforce_size_threshold.is_empty()
          && module_group
            .get_sizes(&module_sizes)
            .bigger_than(&cache_group.enforce_size_threshold);

        let mut used_chunks = Cow::Borrowed(&module_group.chunks);

        if !enforce_size_exceeded {
          self.ensure_max_request_fit(compilation, cache_group, &mut used_chunks);
        }

        // `ensure_max_request_fit` can remove all source chunks for only some modules in a named
        // group. Track the exact original module-chunk edges that this split will consume instead
        // of treating the remaining modules and chunks as a cross product.
        let mut used_chunks = used_chunks.into_owned();
        let mut placed_module_chunks =
          self.get_module_chunks_to_move(&module_group, new_chunk, &used_chunks, compilation);
        let mut modules_to_move = placed_module_chunks
          .keys()
          .copied()
          .collect::<IdentifierSet>();
        {
          let chunk_graph = &compilation.build_chunk_graph_artifact.chunk_graph;
          if is_reuse_existing_chunk {
            // A module already in the reused destination does not need to move, but that original
            // placement still belongs to the winning group and must be unavailable to competing
            // groups at this and lower priorities.
            for module in module_group
              .modules
              .iter()
              .filter(|module| chunk_graph.is_module_in_chunk(module, new_chunk))
            {
              placed_module_chunks
                .entry(*module)
                .or_default()
                .insert(new_chunk);
            }
          }
        }

        let modules_without_placement = module_group
          .modules
          .iter()
          .filter(|module| {
            placed_module_chunks
              .get(module)
              .is_none_or(|chunks| chunks.len() < cache_group.min_chunks as usize)
          })
          .copied()
          .collect::<Vec<_>>();
        for module in modules_without_placement {
          module_group.remove_module(module);
        }

        // Max-request pruning and chunk conditions can change the module subset, so size checks
        // performed before selecting `used_chunks` are no longer sufficient.
        if min_size::remove_min_size_violating_modules(
          &module_group_key,
          &mut module_group,
          cache_group,
          &module_sizes,
        ) {
          tracing::trace!(
            "ModuleGroup({module_group_key}) is skipped after selecting its actual placements because it violates min_size {:#?}",
            cache_group.min_size,
          );
          continue;
        }

        placed_module_chunks.retain(|module, _| module_group.modules.contains(module));
        modules_to_move.retain(|module| module_group.modules.contains(module));

        let placement_chunks = placed_module_chunks
          .values()
          .flatten()
          .copied()
          .collect::<FxHashSet<_>>();
        used_chunks.retain(|chunk| placement_chunks.contains(chunk));

        if placement_chunks.len() < cache_group.min_chunks as usize {
          tracing::trace!(
            "ModuleGroup({module_group_key}) is skipped. Reason: placement_chunks.len()({:?}) < cache_group.min_chunks({:?})",
            placement_chunks.len(),
            cache_group.min_chunks
          );
          continue;
        }

        if !Self::check_min_size_reduction(
          module_group.get_sizes(&module_sizes),
          &cache_group.min_size_reduction,
          placement_chunks.len(),
        ) {
          tracing::trace!(
            "ModuleGroup({module_group_key}) is skipped after selecting its actual placements because it violates min_size_reduction {:#?}",
            cache_group.min_size_reduction,
          );
          continue;
        }

        // Only mutate metadata on an existing destination after the winning group has passed all
        // checks. A group skipped after max-request pruning must not leave cache-group metadata on
        // a chunk it did not actually use.
        let new_chunk_mut = compilation
          .build_chunk_graph_artifact
          .chunk_by_ukey
          .expect_get_mut(&new_chunk);
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
          &modules_to_move,
          &placed_module_chunks,
          new_chunk,
          compilation,
        );

        self.split_from_original_chunks(&module_group, &used_chunks, new_chunk, compilation);

        self.remove_all_modules_from_other_module_groups(
          &placed_module_chunks,
          &mut module_group_map,
          &module_sizes,
        );

        if index != priority_len - 1 {
          for (module, chunks) in &placed_module_chunks {
            removed_module_chunks
              .entry(*module)
              .or_default()
              .extend(chunks.iter().copied());
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
