use std::hash::Hash;

use rspack_core::{ChunkGroupByUkey, ChunkGroupUkey, ChunkUkey, Compilation};
use rspack_identifier::IdentifierLinkedMap;
use rustc_hash::FxHashSet as HashSet;
use xxhash_rust::xxh3::Xxh3;

pub fn update_hash_for_entry_startup(
  hasher: &mut Xxh3,
  compilation: &Compilation,
  entries: &IdentifierLinkedMap<ChunkGroupUkey>,
  chunk: &ChunkUkey,
) {
  for (module, entry) in entries {
    if let Some(module_id) = compilation
      .module_graph
      .module_graph_module_by_identifier(module)
      .map(|module| module.id(&compilation.chunk_graph))
    {
      module_id.hash(hasher);
    }

    if let Some(runtime_chunk) = compilation
      .chunk_group_by_ukey
      .get(entry)
      .map(|e| e.get_runtime_chunk())
    {
      for chunk_ukey in get_all_chunks(
        entry,
        chunk,
        &runtime_chunk,
        &compilation.chunk_group_by_ukey,
      ) {
        if let Some(chunk) = compilation.chunk_by_ukey.get(&chunk_ukey) {
          chunk.id.hash(hasher);
        }
      }
    }
  }
}

pub fn get_all_chunks(
  entrypoint: &ChunkGroupUkey,
  exclude_chunk1: &ChunkUkey,
  exclude_chunk2: &ChunkUkey,
  chunk_group_by_ukey: &ChunkGroupByUkey,
) -> HashSet<ChunkUkey> {
  fn add_chunks(
    chunk_group_by_ukey: &ChunkGroupByUkey,
    chunks: &mut HashSet<ChunkUkey>,
    entrypoint_ukey: &ChunkGroupUkey,
    exclude_chunk1: &ChunkUkey,
    exclude_chunk2: &ChunkUkey,
  ) {
    if let Some(entrypoint) = chunk_group_by_ukey.get(entrypoint_ukey) {
      for chunk in &entrypoint.chunks {
        if chunk == exclude_chunk1 || chunk == exclude_chunk2 {
          continue;
        }
        chunks.insert(*chunk);
      }

      for parent in entrypoint.ancestors(chunk_group_by_ukey) {
        if let Some(chunk_group) = chunk_group_by_ukey.get(&parent) {
          if chunk_group.is_initial() {
            add_chunks(
              chunk_group_by_ukey,
              chunks,
              &chunk_group.ukey,
              exclude_chunk1,
              exclude_chunk2,
            );
          }
        }
      }
    }
  }

  let mut chunks: HashSet<ChunkUkey> = HashSet::default();

  add_chunks(
    chunk_group_by_ukey,
    &mut chunks,
    entrypoint,
    exclude_chunk1,
    exclude_chunk2,
  );

  chunks
}
