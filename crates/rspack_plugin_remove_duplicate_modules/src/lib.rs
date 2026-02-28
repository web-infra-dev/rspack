use std::sync::Arc;

use dashmap::DashMap;
use rayon::prelude::*;
use rspack_collections::IdentifierSet;
use rspack_core::{
  ChunkUkey, Compilation, CompilationOptimizeChunks, ModuleIdentifier, Plugin,
  incremental::Mutation,
};
use rspack_error::Result;
use rspack_hook::{plugin, plugin_hook};

#[derive(Debug)]
#[plugin]
pub struct RemoveDuplicateModulesPlugin {}

impl std::default::Default for RemoveDuplicateModulesPlugin {
  fn default() -> Self {
    Self {
      inner: Arc::new(RemoveDuplicateModulesPluginInner {}),
    }
  }
}

fn find_reusable_chunk(
  compilation: &Compilation,
  chunks: &[ChunkUkey],
  modules: &[ModuleIdentifier],
) -> Option<ChunkUkey> {
  let filter = |chunk: &&ChunkUkey| {
    let chunk_modules = compilation
      .build_chunk_graph_artifact
      .chunk_graph
      .get_chunk_modules_identifier(chunk);
    modules.len() == chunk_modules.len()
      && modules.iter().all(|module| chunk_modules.contains(module))
  };

  if chunks.len() > 10 {
    chunks.par_iter().find_first(filter).copied()
  } else {
    chunks.iter().find(filter).copied()
  }
}

#[plugin_hook(CompilationOptimizeChunks for RemoveDuplicateModulesPlugin)]
async fn optimize_chunks(&self, compilation: &mut Compilation) -> Result<Option<bool>> {
  let module_graph = compilation.get_module_graph();
  let chunk_graph = &compilation.build_chunk_graph_artifact.chunk_graph;

  let chunk_map: DashMap<Vec<ChunkUkey>, Vec<ModuleIdentifier>> = DashMap::default();

  module_graph.modules_par().for_each(|(identifier, _)| {
    let chunks = chunk_graph.get_module_chunks(*identifier);
    let mut sorted_chunks = chunks.iter().copied().collect::<Vec<_>>();
    sorted_chunks.sort();
    chunk_map
      .entry(sorted_chunks)
      .or_default()
      .push(*identifier);
  });

  /*
    sort chunks so that do max effort to find reusable chunk
    eg. 3 entry
    entry1: [main, foo, bar]
    entry2: [foo, bar]
    entry3: [bar]

    the chunk map is
    main:[entry1]
    foo: [entry1, entry2]
    bar: [entry1, entry2, entry3]

    sorted
    1. so bar gets split first,
      found usable chunk entry3!
    2. then split foo, found usable chunk entry2!

    the result chunk is
    main -> foo -> bar

    the algorithm is easy and cannot cover all optimization possibilities, but
    its performance is good and it works for most sceneries, if you have better
    algorithm feel free to contribute, thanks
  */
  let mut chunk_map = chunk_map.into_iter().collect::<Vec<_>>();
  chunk_map.sort_by_key(|(chunks, _)| chunks.len());

  for (chunks, modules) in chunk_map.into_iter().rev() {
    if chunks.len() <= 1 {
      continue;
    }

    // split chunks from original chunks and create new chunk
    let new_chunk_ukey = if let Some(chunk) = find_reusable_chunk(compilation, &chunks, &modules) {
      // we can use this chunk directly
      // all modules are into existing chunk, the chunkMap needs update

      chunk
    } else {
      let new_chunk_ukey =
        Compilation::add_chunk(&mut compilation.build_chunk_graph_artifact.chunk_by_ukey);
      if let Some(mut mutations) = compilation.incremental.mutations_write() {
        mutations.add(Mutation::ChunkAdd {
          chunk: new_chunk_ukey,
        });
      };
      let new_chunk = compilation
        .build_chunk_graph_artifact
        .chunk_by_ukey
        .expect_get_mut(&new_chunk_ukey);
      *new_chunk.chunk_reason_mut() = Some("modules are shared across multiple chunks".into());
      compilation
        .build_chunk_graph_artifact
        .chunk_graph
        .add_chunk(new_chunk_ukey);

      new_chunk_ukey
    };

    let mut entry_modules = IdentifierSet::default();

    for chunk_ukey in &chunks {
      if chunk_ukey == &new_chunk_ukey {
        continue;
      }

      let [Some(new_chunk), Some(origin)] = compilation
        .build_chunk_graph_artifact
        .chunk_by_ukey
        .get_many_mut([&new_chunk_ukey, chunk_ukey])
      else {
        panic!("should have both chunks")
      };
      entry_modules.extend(
        compilation
          .build_chunk_graph_artifact
          .chunk_graph
          .get_chunk_entry_modules(chunk_ukey),
      );
      origin.split(
        new_chunk,
        &mut compilation.build_chunk_graph_artifact.chunk_group_by_ukey,
      );
      if let Some(mut mutations) = compilation.incremental.mutations_write() {
        mutations.add(Mutation::ChunkSplit {
          from: *chunk_ukey,
          to: new_chunk_ukey,
        });
      }
    }

    for m in modules {
      let is_entry = entry_modules.contains(&m);
      for chunk_ukey in &chunks {
        if chunk_ukey == &new_chunk_ukey {
          continue;
        }
        compilation
          .build_chunk_graph_artifact
          .chunk_graph
          .disconnect_chunk_and_module(chunk_ukey, m);

        if is_entry {
          compilation
            .build_chunk_graph_artifact
            .chunk_graph
            .disconnect_chunk_and_entry_module(chunk_ukey, m);
        }
      }

      compilation
        .build_chunk_graph_artifact
        .chunk_graph
        .connect_chunk_and_module(new_chunk_ukey, m);

      if is_entry {
        let chunk = compilation
          .build_chunk_graph_artifact
          .chunk_by_ukey
          .expect_get(&new_chunk_ukey);
        for group in chunk.groups().iter().filter(|group| {
          let group = compilation
            .build_chunk_graph_artifact
            .chunk_group_by_ukey
            .expect_get(group);

          group.is_initial() && group.kind.is_entrypoint()
        }) {
          compilation
            .build_chunk_graph_artifact
            .chunk_graph
            .connect_chunk_and_entry_module(new_chunk_ukey, m, *group);
        }
      }
    }
  }

  Ok(None)
}

impl Plugin for RemoveDuplicateModulesPlugin {
  fn apply(&self, ctx: &mut rspack_core::ApplyContext<'_>) -> Result<()> {
    ctx
      .compilation_hooks
      .optimize_chunks
      .tap(optimize_chunks::new(self));
    Ok(())
  }
}
