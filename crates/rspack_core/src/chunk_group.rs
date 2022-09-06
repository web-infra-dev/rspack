use hashbrown::{HashMap, HashSet};

use crate::{ChunkByUkey, ChunkGroupUkey, ChunkUkey};

#[derive(Debug, Default)]
pub struct ChunkGroup {
  pub ukey: ChunkGroupUkey,
  pub(crate) chunks: Vec<ChunkUkey>,
  module_pre_order_indices: HashMap<String, usize>,
  module_post_order_indices: HashMap<String, usize>,
  parents: HashSet<ChunkGroupUkey>,
  children: HashSet<ChunkGroupUkey>,
}

impl ChunkGroup {
  pub fn new() -> Self {
    Self {
      ukey: ChunkGroupUkey::new(),
      chunks: vec![],
      module_post_order_indices: Default::default(),
      module_pre_order_indices: Default::default(),
      parents: Default::default(),
      children: Default::default(),
    }
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
}

pub(crate) type ChunkGroupByUkey = HashMap<ChunkGroupUkey, ChunkGroup>;
