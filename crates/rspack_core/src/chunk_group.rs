use rspack_database::DatabaseItem;
use rspack_identifier::IdentifierMap;
use rustc_hash::FxHashSet as HashSet;

use crate::{
  Chunk, ChunkByUkey, ChunkGroupByUkey, ChunkGroupUkey, ChunkUkey, ModuleIdentifier, RuntimeSpec,
};

impl DatabaseItem for ChunkGroup {
  fn ukey(&self) -> rspack_database::Ukey<Self> {
    self.ukey
  }
}

#[derive(Debug)]
pub struct ChunkGroup {
  pub ukey: ChunkGroupUkey,
  pub chunks: Vec<ChunkUkey>,
  pub(crate) module_pre_order_indices: IdentifierMap<usize>,
  pub(crate) module_post_order_indices: IdentifierMap<usize>,
  pub(crate) parents: HashSet<ChunkGroupUkey>,
  pub(crate) children: HashSet<ChunkGroupUkey>,
  pub(crate) kind: ChunkGroupKind,
  // ChunkGroupInfo
  pub(crate) next_pre_order_index: usize,
  pub(crate) next_post_order_index: usize,
  pub(crate) runtime: RuntimeSpec,
  // Entrypoint
  // pub(crate) name: Option<String>,
  pub(crate) runtime_chunk: Option<ChunkUkey>,
  pub(crate) entry_point_chunk: Option<ChunkUkey>,
}

impl ChunkGroup {
  pub fn new(kind: ChunkGroupKind, runtime: RuntimeSpec, _name: Option<String>) -> Self {
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

  pub fn module_post_order_index(&self, module_identifier: &ModuleIdentifier) -> Option<usize> {
    // A module could split into another ChunkGroup, which doesn't have the module_post_order_indices of the module
    self
      .module_post_order_indices
      .get(module_identifier)
      .copied()
  }

  pub fn get_files(&self, chunk_by_ukey: &ChunkByUkey) -> Vec<String> {
    self
      .chunks
      .iter()
      .flat_map(|chunk_ukey| {
        chunk_by_ukey
          .get(chunk_ukey)
          .unwrap_or_else(|| panic!("Chunk({chunk_ukey:?}) not found in ChunkGroup: {self:?}"))
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

  pub fn ancestors(&self, chunk_group_by_ukey: &ChunkGroupByUkey) -> HashSet<ChunkGroupUkey> {
    self
      .parents
      .iter()
      .filter_map(|ukey| chunk_group_by_ukey.get(ukey))
      .flat_map(|group| {
        group
          .ancestors(chunk_group_by_ukey)
          .into_iter()
          .chain([group.ukey])
      })
      .collect()
  }

  pub fn insert_chunk(&mut self, chunk: ChunkUkey, before: ChunkUkey) -> bool {
    let old_idx = self.chunks.iter().position(|ukey| *ukey == chunk);
    let idx = self
      .chunks
      .iter()
      .position(|ukey| *ukey == before)
      .expect("before chunk not found");

    if let Some(old_idx) = old_idx && old_idx > idx {
      self.chunks.remove(old_idx);
      self.chunks.insert(idx, chunk);
    } else if old_idx.is_none() {
      self.chunks.insert(idx, chunk);
      return true
    }

    false
  }

  pub fn remove_chunk(&mut self, chunk: &ChunkUkey) -> bool {
    let idx = self.chunks.iter().position(|ukey| ukey == chunk);
    if let Some(idx) = idx {
      self.chunks.remove(idx);
      return true;
    }

    false
  }
}

#[derive(Debug, PartialEq, Eq)]
pub enum ChunkGroupKind {
  Entrypoint,
  Normal,
}
