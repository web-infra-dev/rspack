use rayon::prelude::*;
use rspack_collections::{IdentifierMap, UkeyDashSet, UkeyMap, UkeySet};
use rspack_core::{ChunkGroupUkey, ChunkUkey, Compilation, incremental::Mutation};

/// Ensure that all entry chunks only export the exports used by other chunks,
/// this requires no other chunks depend on the entry chunk to get exports
///
/// for example entryA -> a -> b => c -> a
/// entry chunk: a, b
/// async chunk: c
/// c depends on a, so entry chunk needs to re-export symbols from a
pub(crate) fn ensure_entry_exports(compilation: &mut Compilation) {
  let module_graph = compilation.get_module_graph();
  let mut entrypoint_chunks = UkeyMap::<ChunkUkey, ChunkGroupUkey>::default();
  let mut entry_module_belongs: IdentifierMap<UkeySet<ChunkUkey>> = IdentifierMap::default();

  for entrypoint_ukey in compilation.entrypoints().values() {
    let entrypoint = compilation.chunk_group_by_ukey.expect_get(entrypoint_ukey);
    let entrypoint_chunk = entrypoint.get_entrypoint_chunk();

    // we should use get_chunk_modules instead of entrydata.modules
    // because entry modules may be moved to another chunk
    // we only care if the real entry chunk is a dependency of other chunks
    let entry_modules = compilation
      .chunk_graph
      .get_chunk_modules_identifier(&entrypoint_chunk);

    entrypoint_chunks.insert(entrypoint_chunk, *entrypoint_ukey);

    for m in entry_modules {
      entry_module_belongs
        .entry(*m)
        .or_default()
        .insert(entrypoint_chunk);
    }
  }

  let dirty_chunks = UkeyDashSet::default();

  compilation
    .chunk_by_ukey
    .iter()
    .par_bridge()
    .filter(|(ukey, _)| !entrypoint_chunks.contains_key(ukey))
    .for_each(|(ukey, _)| {
      let modules = compilation.chunk_graph.get_chunk_modules_identifier(ukey);

      for m in modules {
        // check if module has used entry chunk exports
        let outgoings = module_graph.get_active_outcoming_connections_by_module(
          m,
          None,
          &module_graph,
          &compilation.module_graph_cache_artifact,
        );
        for outgoing in outgoings.keys() {
          if let Some(entry_chunks) = entry_module_belongs.get(outgoing) {
            for entry_chunk in entry_chunks {
              dirty_chunks.insert(*entry_chunk);
            }
            break;
          }
        }
      }
    });

  // the dirty chunks need to re-exports the needed exports,
  // so move all modules inside it into another chunk.
  // And we should split a runtime chunk as well, to make sure the new chunk depends on runtime
  // will not cause circular dependency
  for entry_chunk_ukey in dirty_chunks {
    let entry_modules = compilation
      .chunk_graph
      .get_chunk_modules_identifier(&entry_chunk_ukey)
      .iter()
      .copied()
      .collect::<Vec<_>>();

    let new_chunk_ukey = Compilation::add_chunk(&mut compilation.chunk_by_ukey);
    if let Some(mutation) = compilation.incremental.mutations_write() {
      mutation.add(Mutation::ChunkAdd {
        chunk: new_chunk_ukey,
      });
    }
    compilation.chunk_graph.add_chunk(new_chunk_ukey);

    // move entrypoint runtime as well
    let entrypoint_ukey = entrypoint_chunks[&entry_chunk_ukey];
    let entrypoint = compilation.chunk_group_by_ukey.expect_get(&entrypoint_ukey);
    if entrypoint.get_runtime_chunk(&compilation.chunk_group_by_ukey) == entry_chunk_ukey {
      let entrypoint = compilation
        .chunk_group_by_ukey
        .expect_get_mut(&entrypoint_ukey);
      entrypoint.set_runtime_chunk(new_chunk_ukey);
    }

    for m in entry_modules {
      compilation
        .chunk_graph
        .disconnect_chunk_and_entry_module(&entry_chunk_ukey, m);
      compilation
        .chunk_graph
        .disconnect_chunk_and_module(&entry_chunk_ukey, m);

      compilation
        .chunk_graph
        .connect_chunk_and_module(new_chunk_ukey, m);

      let entrypoint = entrypoint_chunks[&entry_chunk_ukey];
      compilation
        .chunk_graph
        .connect_chunk_and_entry_module(new_chunk_ukey, m, entrypoint);
    }

    let [Some(entry_chunk), Some(new_chunk)] = compilation
      .chunk_by_ukey
      .get_many_mut([&entry_chunk_ukey, &new_chunk_ukey])
    else {
      unreachable!()
    };

    entry_chunk.split(new_chunk, &mut compilation.chunk_group_by_ukey);
  }
}
