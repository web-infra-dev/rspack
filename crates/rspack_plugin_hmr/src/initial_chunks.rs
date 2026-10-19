use rspack_core::{
  AsyncDependenciesBlockIdentifier, Compilation, GroupOptions, chunk_graph_chunk::ChunkId,
};
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};

#[derive(Debug, PartialEq, Eq, Hash)]
pub(crate) enum EntryKey {
  Named(String),
  Async(AsyncDependenciesBlockIdentifier),
}

pub(crate) type EntryChunks = HashMap<EntryKey, (ChunkId, Vec<ChunkId>)>;

/// Record startup dependencies for ordinary and async entries, including dependOn parents.
pub(crate) fn collect_entry_chunks(compilation: &Compilation) -> EntryChunks {
  let graph = &compilation.build_chunk_graph_artifact;
  let module_graph = compilation.get_module_graph();
  let entries = compilation
    .entrypoints()
    .iter()
    .map(|(name, group)| (EntryKey::Named(name.clone()), *group));
  // Keep async entries tied to their dependency blocks across compilations,
  // even when splitting changes their physical entry chunk IDs.
  let async_entries = module_graph
    .blocks()
    .values()
    .filter(|block| matches!(block.get_group_options(), Some(GroupOptions::Entrypoint(_))))
    .filter_map(|block| {
      graph
        .chunk_graph
        .get_block_chunk_group(&block.identifier(), &graph.chunk_group_by_ukey)
        .map(|group| (EntryKey::Async(block.identifier()), group.ukey()))
    });
  entries
    .chain(async_entries)
    .map(|(key, entry_group_ukey)| {
      let entry = graph.chunk_group_by_ukey.expect_get(&entry_group_ukey);
      let entry_chunk = graph
        .chunk_by_ukey
        .expect_get(&entry.get_entrypoint_chunk());
      let mut groups = vec![entry_group_ukey];
      let mut visited_groups = HashSet::default();
      let mut visited_chunks = HashSet::default();
      let mut chunks = Vec::new();
      while let Some(group_ukey) = groups.pop() {
        let group = graph.chunk_group_by_ukey.expect_get(&group_ukey);
        if (group_ukey != entry_group_ukey && !group.is_initial())
          || !visited_groups.insert(group_ukey)
        {
          continue;
        }
        if group.is_initial() {
          groups.extend(group.parents.iter().copied());
        }
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
      (key, (entry_chunk.expect_id().clone(), chunks))
    })
    .collect()
}
