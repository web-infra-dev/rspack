use hashbrown::{HashMap, HashSet};

use crate::{Chunk, ChunkByUkey, ChunkGroupUkey, ChunkUkey, RuntimeSpec};

#[derive(Debug)]
pub struct ChunkGroup {
  pub ukey: ChunkGroupUkey,
  pub chunks: Vec<ChunkUkey>,
  pub(crate) module_pre_order_indices: HashMap<String, usize>,
  pub(crate) module_post_order_indices: HashMap<String, usize>,
  pub(crate) parents: HashSet<ChunkGroupUkey>,
  pub(crate) children: HashSet<ChunkGroupUkey>,
  pub(crate) kind: ChunkGroupKind,
  // ChunkGroupInfo
  pub(crate) next_pre_order_index: usize,
  pub(crate) next_post_order_index: usize,
  pub(crate) runtime: Option<RuntimeSpec>,
  // Entrypoint
  // pub(crate) name: Option<String>,
  pub(crate) runtime_chunk: Option<ChunkUkey>,
  pub(crate) entry_point_chunk: Option<ChunkUkey>,
}

impl ChunkGroup {
  pub fn new(kind: ChunkGroupKind, name: Option<String>) -> Self {
    // TODO respect entrypoint `runtime` + `dependOn`
    let runtime = match kind {
      ChunkGroupKind::Entrypoint => Some(HashSet::from([
        name.expect("ChunkGroupKind::Entrypoint name shouldn't be none")
      ])),
      ChunkGroupKind::Normal => None,
    };
    Self {
      ukey: ChunkGroupUkey::new(),
      chunks: vec![],
      module_post_order_indices: Default::default(),
      module_pre_order_indices: Default::default(),
      parents: Default::default(),
      children: Default::default(),
      kind,
      next_pre_order_index: 0,
      next_post_order_index: 0,
      runtime,
      // name,
      runtime_chunk: None,
      entry_point_chunk: None,
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

  pub fn unshift_chunk(&mut self, chunk: &mut Chunk) {
    self.chunks.insert(0, chunk.ukey);
    chunk.add_group(self.ukey);
  }

  pub(crate) fn is_initial(&self) -> bool {
    matches!(self.kind, ChunkGroupKind::Entrypoint)
  }

  pub fn set_runtime_chunk(&mut self, chunk_ukey: ChunkUkey) {
    self.runtime_chunk = Some(chunk_ukey);
  }

  pub fn get_runtime_chunk(&self) -> ChunkUkey {
    match self.kind {
      ChunkGroupKind::Entrypoint => self
        .runtime_chunk
        .expect("EntryPoint runtime chunk not set"),
      ChunkGroupKind::Normal => unreachable!("Normal chunk group doesn't have runtime chunk"),
    }
  }

  pub fn set_entry_point_chunk(&mut self, chunk_ukey: ChunkUkey) {
    self.entry_point_chunk = Some(chunk_ukey);
  }

  pub fn get_entry_point_chunk(&self) -> ChunkUkey {
    match self.kind {
      ChunkGroupKind::Entrypoint => self
        .entry_point_chunk
        .expect("EntryPoint runtime chunk not set"),
      ChunkGroupKind::Normal => unreachable!("Normal chunk group doesn't have runtime chunk"),
    }
  }
}

#[derive(Debug, PartialEq, Eq)]
pub enum ChunkGroupKind {
  Entrypoint,
  Normal,
}
