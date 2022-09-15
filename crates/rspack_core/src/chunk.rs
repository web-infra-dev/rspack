use hashbrown::HashSet;

use crate::{ChunkGroupByUkey, ChunkGroupUkey, ChunkUkey};

#[derive(Debug)]
pub struct Chunk {
  pub(crate) _name: Option<String>,
  pub ukey: ChunkUkey,
  pub id: String,
  pub kind: ChunkKind,
  pub files: HashSet<String>,
  pub groups: HashSet<ChunkGroupUkey>,
}

impl Chunk {
  pub fn new(_name: Option<String>, id: String, kind: ChunkKind) -> Self {
    Self {
      _name,
      ukey: ChunkUkey::with_debug_info("Chunk"),
      id,
      kind,
      files: Default::default(),
      groups: Default::default(),
    }
  }

  pub(crate) fn add_group(&mut self, group: ChunkGroupUkey) {
    self.groups.insert(group);
  }

  pub(crate) fn split(
    &mut self,
    new_chunk: &mut Chunk,
    chunk_group_by_ukey: &mut ChunkGroupByUkey,
  ) {
    self.groups.iter().for_each(|group| {
      let group = chunk_group_by_ukey
        .get_mut(group)
        .expect("Group should exist");
      group.chunks.push(new_chunk.ukey);
      new_chunk.add_group(group.ukey);
    });
  }
}

#[derive(Debug)]
pub enum ChunkKind {
  Entry,
  Normal,
  // TODO: support it.
  // Initial,
}

impl ChunkKind {
  pub fn is_entry(&self) -> bool {
    matches!(self, ChunkKind::Entry { .. })
  }
  pub fn is_normal(&self) -> bool {
    matches!(self, ChunkKind::Normal)
  }
}
