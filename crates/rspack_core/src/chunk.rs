use hashbrown::HashSet;

use crate::{ChunkGraph, ChunkGroupByUkey, ChunkGroupUkey, ChunkUkey};

#[derive(Debug)]
pub struct Chunk {
  pub name: Option<String>,
  pub ukey: ChunkUkey,
  pub id: String,
  pub files: HashSet<String>,
  pub groups: HashSet<ChunkGroupUkey>,
}

impl Chunk {
  pub fn new(name: Option<String>, id: String) -> Self {
    Self {
      name,
      ukey: ChunkUkey::with_debug_info("Chunk"),
      id,
      files: Default::default(),
      groups: Default::default(),
    }
  }

  pub(crate) fn add_group(&mut self, group: ChunkGroupUkey) {
    self.groups.insert(group);
  }

  pub fn split(&mut self, new_chunk: &mut Chunk, chunk_group_by_ukey: &mut ChunkGroupByUkey) {
    self.groups.iter().for_each(|group| {
      let group = chunk_group_by_ukey
        .get_mut(group)
        .expect("Group should exist");
      group.chunks.push(new_chunk.ukey);
      new_chunk.add_group(group.ukey);
    });
  }

  pub fn can_be_initial(&self, chunk_group_by_ukey: &ChunkGroupByUkey) -> bool {
    self
      .groups
      .iter()
      .filter_map(|ukey| chunk_group_by_ukey.get(ukey))
      .any(|group| group.is_initial())
  }

  pub fn is_only_initial(&self, chunk_group_by_ukey: &ChunkGroupByUkey) -> bool {
    self
      .groups
      .iter()
      .filter_map(|ukey| chunk_group_by_ukey.get(ukey))
      .all(|group| group.is_initial())
  }

  pub fn has_entry_module(&self, chunk_graph: &ChunkGraph) -> bool {
    !chunk_graph.get_chunk_entry_modules(&self.ukey).is_empty()
  }
}
