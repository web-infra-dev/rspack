use hashbrown::HashSet;

use crate::{ChunkGraph, ChunkGroupByUkey, ChunkGroupUkey, ChunkUkey, RuntimeSpec};

#[derive(Debug)]
pub struct Chunk {
  pub(crate) _name: Option<String>,
  pub ukey: ChunkUkey,
  pub id: String,
  pub files: HashSet<String>,
  pub groups: HashSet<ChunkGroupUkey>,
  pub runtime: RuntimeSpec,
}

impl Chunk {
  pub fn new(_name: Option<String>, id: String) -> Self {
    Self {
      _name,
      ukey: ChunkUkey::with_debug_info("Chunk"),
      id,
      files: Default::default(),
      groups: Default::default(),
      runtime: HashSet::default(),
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
    new_chunk.runtime.extend(self.runtime.clone());
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

  pub fn get_all_referenced_chunks(
    &self,
    chunk_group_by_ukey: &ChunkGroupByUkey,
  ) -> HashSet<ChunkUkey> {
    let mut chunks = HashSet::new();

    for group_ukey in self.groups.iter() {
      let group = chunk_group_by_ukey
        .get(&group_ukey)
        .expect("Group should exist");
      for chunk in &group.chunks {
        chunks.insert(*chunk);
      }
      for chunk in &group.children {
        chunks.insert(*chunk);
      }
    }

    chunks
  }
}
