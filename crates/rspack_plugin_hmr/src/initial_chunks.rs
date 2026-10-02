use rspack_core::{Compilation, chunk_graph_chunk::ChunkId};
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};

pub(crate) type EntryChunks = HashMap<String, (ChunkId, Vec<ChunkId>)>;

/// Record each entry's initial dependencies for HMR, including dependOn parents.
pub(crate) fn collect_entry_chunks(compilation: &Compilation) -> EntryChunks {
  let graph = &compilation.build_chunk_graph_artifact;
  compilation
    .entrypoints()
    .iter()
    .map(|(name, group_ukey)| {
      let entry = graph.chunk_group_by_ukey.expect_get(group_ukey);
      let entry_chunk = graph
        .chunk_by_ukey
        .expect_get(&entry.get_entrypoint_chunk());
      let mut groups = vec![*group_ukey];
      let mut visited_groups = HashSet::default();
      let mut visited_chunks = HashSet::default();
      let mut chunks = Vec::new();
      while let Some(group_ukey) = groups.pop() {
        let group = graph.chunk_group_by_ukey.expect_get(&group_ukey);
        if !group.is_initial() || !visited_groups.insert(group_ukey) {
          continue;
        }
        groups.extend(group.parents.iter().copied());
        for chunk_ukey in &group.chunks {
          if visited_chunks.insert(*chunk_ukey) {
            chunks.push(
              graph
                .chunk_by_ukey
                .expect_get(chunk_ukey)
                .expect_id()
                .clone(),
            );
          }
        }
      }
      // This records membership only. Keep manifest serialization deterministic.
      chunks.sort_unstable();
      (name.clone(), (entry_chunk.expect_id().clone(), chunks))
    })
    .collect()
}
