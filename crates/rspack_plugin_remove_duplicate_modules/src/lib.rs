use std::sync::Arc;

use rayon::prelude::*;
use rspack_collections::IdentifierSet;
use rspack_core::{
  ChunkUkey, Compilation, CompilationOptimizeChunks, ModuleIdentifier, Plugin,
  incremental::Mutation,
};
use rspack_error::Result;
use rspack_hook::{plugin, plugin_hook};
use rspack_util::fx_hash::FxDashMap;

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

fn has_non_initial_entry_module(
  compilation: &Compilation,
  chunks: &[ChunkUkey],
  modules: &[ModuleIdentifier],
) -> bool {
  chunks.iter().any(|chunk| {
    compilation
      .build_chunk_graph_artifact
      .chunk_graph
      .get_chunk_entry_modules_with_chunk_group_iterable(chunk)
      .iter()
      .any(|(module, group)| {
        if !modules.contains(module) {
          return false;
        }

        let group = compilation
          .build_chunk_graph_artifact
          .chunk_group_by_ukey
          .expect_get(group);
        group.kind.is_entrypoint() && !group.is_initial()
      })
  })
}

fn move_empty_non_initial_entrypoints(
  compilation: &mut Compilation,
  chunks: &[ChunkUkey],
  modules: &[ModuleIdentifier],
  new_chunk_ukey: ChunkUkey,
) {
  let mut entrypoints = Vec::new();

  for chunk_ukey in chunks {
    if chunk_ukey == &new_chunk_ukey
      || compilation
        .build_chunk_graph_artifact
        .chunk_graph
        .get_number_of_chunk_modules(chunk_ukey)
        != 0
    {
      continue;
    }

    for (module, group) in compilation
      .build_chunk_graph_artifact
      .chunk_graph
      .get_chunk_entry_modules_with_chunk_group_iterable(chunk_ukey)
    {
      let chunk_group = compilation
        .build_chunk_graph_artifact
        .chunk_group_by_ukey
        .expect_get(group);
      if modules.contains(module)
        && chunk_group.kind.is_entrypoint()
        && !chunk_group.is_initial()
        && chunk_group.get_entrypoint_chunk() == *chunk_ukey
      {
        entrypoints.push((*chunk_ukey, *module, *group));
      }
    }
  }

  for (chunk_ukey, module, group) in entrypoints {
    // The replacement becomes the entrypoint chunk, so it also owns the async entry's output
    // identity. Clear that identity from the old runtime chunk to avoid `[name]` collisions.
    let name = {
      let [Some(new_chunk), Some(old_chunk)] = compilation
        .build_chunk_graph_artifact
        .chunk_by_ukey
        .get_many_mut([&new_chunk_ukey, &chunk_ukey])
      else {
        panic!("should have both chunks")
      };

      let name = old_chunk.name().map(ToOwned::to_owned);
      let filename_template = old_chunk.filename_template().cloned();
      if name.is_some() {
        old_chunk.set_name(None);
        old_chunk.set_filename_template(None);

        if new_chunk.name().is_none() {
          new_chunk.set_name(name.clone());
          new_chunk.set_filename_template(filename_template);
        }
      }

      name
    };

    if let Some(name) = name {
      compilation
        .build_chunk_graph_artifact
        .named_chunks
        .insert(name, new_chunk_ukey);
    }

    compilation
      .build_chunk_graph_artifact
      .chunk_graph
      .disconnect_chunk_and_entry_module(&chunk_ukey, module);
    compilation
      .build_chunk_graph_artifact
      .chunk_graph
      .connect_chunk_and_entry_module(new_chunk_ukey, module, group);
    compilation
      .build_chunk_graph_artifact
      .chunk_group_by_ukey
      .expect_get_mut(&group)
      .set_entrypoint_chunk(new_chunk_ukey);
  }
}

#[plugin_hook(CompilationOptimizeChunks for RemoveDuplicateModulesPlugin)]
async fn optimize_chunks(&self, compilation: &mut Compilation) -> Result<Option<bool>> {
  let module_graph = compilation.get_module_graph();
  let chunk_graph = &compilation.build_chunk_graph_artifact.chunk_graph;

  let chunk_map: FxDashMap<Vec<ChunkUkey>, Vec<ModuleIdentifier>> = FxDashMap::default();

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

    let preserve_entry_chunks = has_non_initial_entry_module(compilation, &chunks, &modules);

    // split chunks from original chunks and create new chunk
    // A non-initial entrypoint needs to keep its own runtime/startup chunk. Reusing one of the
    // existing chunks would make that entrypoint point at a chunk owned by another runtime.
    let reusable_chunk = if preserve_entry_chunks {
      None
    } else {
      find_reusable_chunk(compilation, &chunks, &modules)
    };
    let new_chunk_ukey = if let Some(chunk) = reusable_chunk {
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

    for &m in &modules {
      let is_entry = entry_modules.contains(&m);
      for chunk_ukey in &chunks {
        if chunk_ukey == &new_chunk_ukey {
          continue;
        }
        compilation
          .build_chunk_graph_artifact
          .chunk_graph
          .disconnect_chunk_and_module(chunk_ukey, m);

        // Keep entry-module associations on their original startup chunks when the shared
        // modules include a non-initial entrypoint. Only the module content moves.
        if is_entry && !preserve_entry_chunks {
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

      if is_entry && !preserve_entry_chunks {
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

    // Empty async entry chunks are not emitted by module output. Point the entrypoint at the
    // shared module chunk while keeping its original runtime chunk.
    if preserve_entry_chunks && compilation.options.output.module {
      move_empty_non_initial_entrypoints(compilation, &chunks, &modules, new_chunk_ukey);
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
