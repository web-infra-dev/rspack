use hashbrown::HashSet;

use crate::{ChunkGroupUkey, ChunkUkey};

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
