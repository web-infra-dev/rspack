use hashbrown::{HashMap, HashSet};

use crate::{Chunk, ChunkByUkey, ChunkGroupUkey, ChunkUkey};

#[derive(Debug)]
pub struct ChunkGroup {
  pub ukey: ChunkGroupUkey,
  pub chunks: Vec<ChunkUkey>,
  pub(crate) module_pre_order_indices: HashMap<String, usize>,
  pub(crate) module_post_order_indices: HashMap<String, usize>,
  _parents: HashSet<ChunkGroupUkey>,
  _children: HashSet<ChunkGroupUkey>,
  kind: ChunkGroupKind,
  // ChunkGroupInfo
  pub(crate) next_pre_order_index: usize,
  pub(crate) next_post_order_index: usize,
}

impl ChunkGroup {
  pub fn new(kind: ChunkGroupKind) -> Self {
    Self {
      ukey: ChunkGroupUkey::new(),
      chunks: vec![],
      module_post_order_indices: Default::default(),
      module_pre_order_indices: Default::default(),
      _parents: Default::default(),
      _children: Default::default(),
      kind,
      next_pre_order_index: 0,
      next_post_order_index: 0,
    }
  }

  pub fn module_post_order_index(&self, module_identifier: &str) -> usize {
    *self
      .module_post_order_indices
      .get(module_identifier)
      .expect("module not found")
  }

  pub fn get_files(&self, chunk_by_ukey: &ChunkByUkey) -> HashSet<String> {
    self
      .chunks
      .iter()
      .flat_map(|chunk_ukey| {
        chunk_by_ukey
          .get(chunk_ukey)
          .unwrap()
          .files
          .iter()
          .map(|file| file.to_string())
      })
      .collect()
  }

  pub(crate) fn connect_chunk(&mut self, chunk: &mut Chunk) {
    self.chunks.push(chunk.ukey);
    chunk.add_group(self.ukey);
  }

  pub(crate) fn is_initial(&self) -> bool {
    matches!(self.kind, ChunkGroupKind::Entrypoint)
  }
}

#[derive(Debug)]
pub enum ChunkGroupKind {
  Entrypoint,
  Normal,
}
