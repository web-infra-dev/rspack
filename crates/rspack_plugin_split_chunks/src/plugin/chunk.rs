use rayon::prelude::*;
use rspack_collections::{IdentifierMap, IdentifierSet};
use rspack_core::{
  Chunk, ChunkSplitData, ChunkUkey, Compilation, ModuleIdentifier, incremental::Mutation,
  module_chunk_condition,
};
use rspack_error::Result;
use rustc_hash::FxHashSet;

use crate::{
  SplitChunksPlugin,
  common::{ModuleChunkMap, ModuleChunks},
  module_group::ModuleGroup,
};

fn put_split_chunk_reason(
  chunk_reason: &mut Option<String>,
  is_reuse_existing_chunk_with_all_modules: bool,
) {
  let reason = if is_reuse_existing_chunk_with_all_modules {
    "reused as split chunk".to_string()
  } else {
    "split chunk".to_string()
  };
  if let Some(chunk_reason) = chunk_reason {
    chunk_reason.push(',');
    chunk_reason.push_str(&reason);
  } else {
    *chunk_reason = Some(reason);
  }
}

impl SplitChunksPlugin {
  pub(crate) fn get_module_chunks(
    all_modules: &[ModuleIdentifier],
    compilation: &Compilation,
  ) -> ModuleChunks {
    let chunk_graph = &compilation.build_chunk_graph_artifact.chunk_graph;
    all_modules
      .par_iter()
      .map(|module| chunk_graph.get_module_chunks(*module).clone())
      .collect::<Vec<_>>()
  }
  /// Affected by `splitChunks.cacheGroups.{cacheGroup}.reuseExistingChunk`
  ///
  /// If there is a code splitting chunk that the contains the same modules as the current `ModuleGroup`,
  /// with `reuseExistingChunk: true`,the code splitting chunks will be reused instead of create a new one
  /// for the current `ModuleGroup`.
  pub(crate) fn find_the_best_reusable_chunk(
    &self,
    compilation: &mut Compilation,
    module_group: &mut ModuleGroup,
  ) -> Option<ChunkUkey> {
    let candidates = module_group.chunks.iter().filter_map(|chunk| {
      let chunk = compilation
        .build_chunk_graph_artifact
        .chunk_by_ukey
        .expect_get(chunk);

      if compilation
        .build_chunk_graph_artifact
        .chunk_graph
        .get_number_of_chunk_modules(&chunk.ukey())
        != module_group.modules.len()
      {
        // Fast path
        return None;
      }

      if module_group.chunks.len() > 1
        && compilation
          .build_chunk_graph_artifact
          .chunk_graph
          .get_number_of_entry_modules(&chunk.ukey())
          > 0
      {
        // `module_group.chunks.len() > 1`: This `ModuleGroup` are related to multiple code splitting chunks.
        // `get_number_of_entry_modules(&chunk.ukey) > 0`:  Current chunk is an initial chunk.

        // I(hyf0) don't see why the process needs to bailout for this condition. But ChatGPT3.5 told me:
        // The condition means that if there are multiple chunks in item and the current chunk is an
        // entry chunk, then it cannot be reused. This is because entry chunks typically contain the core
        // code of an application, while other chunks contain various parts of the application. If
        // an entry chunk is used for other purposes, it may cause the application broken.
        return None;
      }

      let is_all_module_in_chunk = module_group.modules.iter().all(|each_module| {
        compilation
          .build_chunk_graph_artifact
          .chunk_graph
          .is_module_in_chunk(each_module, chunk.ukey())
      });
      if !is_all_module_in_chunk {
        return None;
      }

      Some(chunk)
    });

    /// Port https://github.com/webpack/webpack/blob/b471a6bfb71020f6d8f136ef10b7efb239ef5bbf/lib/optimize/SplitChunksPlugin.js#L1360-L1373
    fn best_reusable_chunk<'a>(first: &'a Chunk, second: &'a Chunk) -> &'a Chunk {
      match (first.name(), second.name()) {
        (None, None) => first,
        (None, Some(_)) => second,
        (Some(_), None) => first,
        (Some(first_name), Some(second_name)) => match first_name.len().cmp(&second_name.len()) {
          std::cmp::Ordering::Greater => second,
          std::cmp::Ordering::Less => first,
          std::cmp::Ordering::Equal => {
            if matches!(second_name.cmp(first_name), std::cmp::Ordering::Less) {
              second
            } else {
              first
            }
          }
        },
      }
    }

    let best_reusable_chunk = candidates.reduce(|best, each| best_reusable_chunk(best, each));

    best_reusable_chunk.map(|c| c.ukey())
  }

  pub(crate) fn get_corresponding_chunk(
    &self,
    compilation: &mut Compilation,
    module_group: &mut ModuleGroup,
    is_reuse_existing_chunk: &mut bool,
    is_reuse_existing_chunk_with_all_modules: &mut bool,
  ) -> ChunkUkey {
    if let Some(chunk_name) = &module_group.chunk_name {
      if let Some(chunk) = compilation
        .build_chunk_graph_artifact
        .named_chunks
        .get(chunk_name)
      {
        *is_reuse_existing_chunk = true;
        *chunk
      } else {
        let (new_chunk_ukey, created) = Compilation::add_named_chunk(
          chunk_name.clone(),
          &mut compilation.build_chunk_graph_artifact.chunk_by_ukey,
          &mut compilation.build_chunk_graph_artifact.named_chunks,
        );
        if created && let Some(mut mutations) = compilation.incremental.mutations_write() {
          mutations.add(Mutation::ChunkAdd {
            chunk: new_chunk_ukey,
          });
        }
        let new_chunk = compilation
          .build_chunk_graph_artifact
          .chunk_by_ukey
          .expect_get_mut(&new_chunk_ukey);

        put_split_chunk_reason(
          new_chunk.chunk_reason_mut(),
          *is_reuse_existing_chunk_with_all_modules,
        );

        compilation
          .build_chunk_graph_artifact
          .chunk_graph
          .add_chunk(new_chunk.ukey());
        new_chunk.ukey()
      }
    } else if module_group.cache_group_reuse_existing_chunk
      && let Some(reusable_chunk) = self.find_the_best_reusable_chunk(compilation, module_group)
    {
      *is_reuse_existing_chunk = true;
      *is_reuse_existing_chunk_with_all_modules = true;
      reusable_chunk
    } else {
      let new_chunk_ukey =
        Compilation::add_chunk(&mut compilation.build_chunk_graph_artifact.chunk_by_ukey);
      if let Some(mut mutations) = compilation.incremental.mutations_write() {
        mutations.add(Mutation::ChunkAdd {
          chunk: new_chunk_ukey,
        });
      }
      let new_chunk = compilation
        .build_chunk_graph_artifact
        .chunk_by_ukey
        .expect_get_mut(&new_chunk_ukey);

      put_split_chunk_reason(
        new_chunk.chunk_reason_mut(),
        *is_reuse_existing_chunk_with_all_modules,
      );

      compilation
        .build_chunk_graph_artifact
        .chunk_graph
        .add_chunk(new_chunk.ukey());
      new_chunk.ukey()
    }
  }

  /// Returns the selected source chunks for modules that can move into `new_chunk`.
  // #[tracing::instrument(skip_all)]
  pub(crate) async fn get_module_chunks_to_move(
    &self,
    item: &ModuleGroup,
    new_chunk: ChunkUkey,
    original_chunks: &FxHashSet<ChunkUkey>,
    compilation: &Compilation,
  ) -> Result<ModuleChunkMap> {
    if item.uses_shared_module_chunks() {
      debug_assert!(original_chunks.is_subset(&item.chunks));
      let chunks = original_chunks.clone();
      let mut modules = IdentifierSet::default();
      for mid in &item.modules {
        if let Some(module) = compilation.module_by_identifier(mid)
          && module_chunk_condition(module.as_ref(), &new_chunk, compilation)
            .await?
            .is_some_and(|condition| !condition)
        {
          continue;
        }
        modules.insert(*mid);
      }
      return Ok(ModuleChunkMap::Shared { modules, chunks });
    }

    let mut module_chunks = IdentifierMap::default();
    for mid in &item.modules {
      let Some(selected_chunks) = item.get_module_chunks(mid) else {
        continue;
      };
      if let Some(module) = compilation.module_by_identifier(mid)
        && module_chunk_condition(module.as_ref(), &new_chunk, compilation)
          .await?
          .is_some_and(|condition| !condition)
      {
        continue;
      }

      let chunks = original_chunks
        .iter()
        .filter(|chunk| {
          selected_chunks.contains(chunk)
            && compilation
              .build_chunk_graph_artifact
              .chunk_graph
              .is_module_in_chunk(mid, **chunk)
        })
        .copied()
        .collect::<FxHashSet<_>>();
      if !chunks.is_empty() {
        module_chunks.insert(*mid, chunks);
      }
    }
    Ok(ModuleChunkMap::ByModule(module_chunks))
  }

  /// Moves `modules` into `new_chunk` and removes their selected source placements.
  // #[tracing::instrument(skip_all)]
  pub(crate) fn move_modules_to_new_chunk_and_remove_from_old_chunks(
    &self,
    placed_module_chunks: &ModuleChunkMap,
    new_chunk: ChunkUkey,
    destination_has_all_modules: bool,
    compilation: &mut Compilation,
  ) {
    let chunk_graph = &mut compilation.build_chunk_graph_artifact.chunk_graph;
    if let ModuleChunkMap::Shared { modules, chunks } = placed_module_chunks {
      let modules = modules.iter().copied().collect::<Vec<_>>();
      let chunks = chunks
        .iter()
        .filter(|chunk| **chunk != new_chunk)
        .copied()
        .collect::<Vec<_>>();
      if !chunks.is_empty() {
        chunk_graph.disconnect_chunks_and_modules(&chunks, &modules);
        if !destination_has_all_modules {
          chunk_graph.connect_chunk_and_modules(new_chunk, &modules);
        }
      }
      return;
    }

    let ModuleChunkMap::ByModule(module_chunks) = placed_module_chunks else {
      unreachable!();
    };
    let module_count = module_chunks.len();
    let mut module_identifiers = Vec::with_capacity(module_count);
    let mut move_module = |module: &ModuleIdentifier, chunks: &FxHashSet<ChunkUkey>| {
      let mut has_source_placement = false;
      for chunk in chunks {
        if *chunk != new_chunk {
          has_source_placement = true;
          chunk_graph.disconnect_chunk_and_module(chunk, *module);
        }
      }
      if has_source_placement {
        module_identifiers.push(*module);
      }
    };

    for (module, chunks) in module_chunks {
      move_module(module, chunks);
    }

    if !destination_has_all_modules {
      chunk_graph.connect_chunk_and_modules(new_chunk, &module_identifiers);
    }
  }

  /// Since the modules are moved into the `new_chunk`, we should
  /// create a connection between the `new_chunk` and `original_chunks`.
  /// Thus, if `original_chunks` want to know which chunk contains moved modules,
  /// it could easily find out.
  // #[tracing::instrument(skip_all)]
  pub(crate) fn split_from_original_chunks(
    &self,
    _item: &ModuleGroup,
    original_chunks: &FxHashSet<ChunkUkey>,
    new_chunk: ChunkUkey,
    compilation: &mut Compilation,
  ) {
    let new_chunk_ukey = new_chunk;
    if original_chunks.is_empty() {
      return;
    }

    {
      let chunk_by_ukey = &mut compilation.build_chunk_graph_artifact.chunk_by_ukey;
      let chunk_group_by_ukey = &mut compilation.build_chunk_graph_artifact.chunk_group_by_ukey;
      let mut split_data = ChunkSplitData::default();
      for original_chunk_ukey in original_chunks {
        debug_assert!(&new_chunk_ukey != original_chunk_ukey);
        let original_chunk = chunk_by_ukey
          .get(original_chunk_ukey)
          .expect("split_from_original_chunks failed");
        original_chunk.split_collect_new_chunk_data(
          new_chunk_ukey,
          chunk_group_by_ukey,
          &mut split_data,
        );
      }

      let new_chunk = chunk_by_ukey
        .get_mut(&new_chunk_ukey)
        .expect("split_from_original_chunks failed");
      split_data.apply_to(new_chunk);
    }

    if let Some(mut mutations) = compilation.incremental.mutations_write() {
      for original_chunk_ukey in original_chunks {
        mutations.add(Mutation::ChunkSplit {
          from: *original_chunk_ukey,
          to: new_chunk_ukey,
        });
      }
    }
  }
}
