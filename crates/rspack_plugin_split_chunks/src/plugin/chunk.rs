use rayon::prelude::*;
use rspack_collections::{DatabaseItem, IdentifierMap, UkeySet};
use rspack_core::{Chunk, ChunkUkey, Compilation, ModuleIdentifier, incremental::Mutation};

use crate::{SplitChunksPlugin, common::ModuleChunks, module_group::ModuleGroup};

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
    let chunk_graph = &compilation.chunk_graph;
    all_modules
      .par_iter()
      .map(|module| (*module, chunk_graph.get_module_chunks(*module).clone()))
      .collect::<IdentifierMap<_>>()
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
      let chunk = compilation.chunk_by_ukey.expect_get(chunk);

      if compilation
        .chunk_graph
        .get_number_of_chunk_modules(&chunk.ukey())
        != module_group.modules.len()
      {
        // Fast path
        return None;
      }

      if module_group.chunks.len() > 1
        && compilation
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
      if let Some(chunk) = compilation.named_chunks.get(chunk_name) {
        *is_reuse_existing_chunk = true;
        *chunk
      } else {
        let (new_chunk_ukey, created) = Compilation::add_named_chunk(
          chunk_name.clone(),
          &mut compilation.chunk_by_ukey,
          &mut compilation.named_chunks,
        );
        if created && let Some(mut mutations) = compilation.incremental.mutations_write() {
          mutations.add(Mutation::ChunkAdd {
            chunk: new_chunk_ukey,
          });
        }
        let new_chunk = compilation.chunk_by_ukey.expect_get_mut(&new_chunk_ukey);

        put_split_chunk_reason(
          new_chunk.chunk_reason_mut(),
          *is_reuse_existing_chunk_with_all_modules,
        );

        compilation.chunk_graph.add_chunk(new_chunk.ukey());
        new_chunk.ukey()
      }
    } else if let Some(reusable_chunk) =
      self.find_the_best_reusable_chunk(compilation, module_group)
      && module_group.cache_group_reuse_existing_chunk
    {
      *is_reuse_existing_chunk = true;
      *is_reuse_existing_chunk_with_all_modules = true;
      reusable_chunk
    } else {
      let new_chunk_ukey = Compilation::add_chunk(&mut compilation.chunk_by_ukey);
      if let Some(mut mutations) = compilation.incremental.mutations_write() {
        mutations.add(Mutation::ChunkAdd {
          chunk: new_chunk_ukey,
        });
      }
      let new_chunk = compilation.chunk_by_ukey.expect_get_mut(&new_chunk_ukey);

      put_split_chunk_reason(
        new_chunk.chunk_reason_mut(),
        *is_reuse_existing_chunk_with_all_modules,
      );

      compilation.chunk_graph.add_chunk(new_chunk.ukey());
      new_chunk.ukey()
    }
  }

  /// This de-duplicated each module fro other chunks, make sure there's only one copy of each module.
  // #[tracing::instrument(skip_all)]
  pub(crate) fn move_modules_to_new_chunk_and_remove_from_old_chunks(
    &self,
    item: &ModuleGroup,
    new_chunk: ChunkUkey,
    original_chunks: &UkeySet<ChunkUkey>,
    compilation: &mut Compilation,
  ) {
    let modules = item
      .modules
      .iter()
      .filter(|mid| {
        if let Some(module) = compilation.module_by_identifier(mid)
          && module
            .chunk_condition(&new_chunk, compilation)
            .is_some_and(|condition| !condition)
        {
          return false;
        }
        true
      })
      .copied()
      .collect::<Vec<_>>();

    let chunks = original_chunks.iter().copied().collect::<Vec<_>>();

    compilation
      .chunk_graph
      .disconnect_chunks_and_modules(&chunks, &modules);

    compilation
      .chunk_graph
      .connect_chunk_and_modules(new_chunk, &modules);
  }

  /// Since the modules are moved into the `new_chunk`, we should
  /// create a connection between the `new_chunk` and `original_chunks`.
  /// Thus, if `original_chunks` want to know which chunk contains moved modules,
  /// it could easily find out.
  // #[tracing::instrument(skip_all)]
  pub(crate) fn split_from_original_chunks(
    &self,
    _item: &ModuleGroup,
    original_chunks: &UkeySet<ChunkUkey>,
    new_chunk: ChunkUkey,
    compilation: &mut Compilation,
  ) {
    let new_chunk_ukey = new_chunk;
    for original_chunk_ukey in original_chunks {
      debug_assert!(&new_chunk_ukey != original_chunk_ukey);
      let [Some(new_chunk), Some(original_chunk)] = compilation
        .chunk_by_ukey
        .get_many_mut([&new_chunk_ukey, original_chunk_ukey])
      else {
        panic!("split_from_original_chunks failed")
      };
      original_chunk.split(new_chunk, &mut compilation.chunk_group_by_ukey);
      if let Some(mut mutations) = compilation.incremental.mutations_write() {
        mutations.add(Mutation::ChunkSplit {
          from: *original_chunk_ukey,
          to: new_chunk_ukey,
        });
      }
    }
  }
}
