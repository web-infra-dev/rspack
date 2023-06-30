use itertools::Itertools;
use rspack_database::DatabaseItem;
use rspack_identifier::IdentifierMap;
use rustc_hash::FxHashSet as HashSet;

use crate::{
  Chunk, ChunkByUkey, ChunkGroupByUkey, ChunkGroupUkey, ChunkUkey, Compilation, EntryOptions,
  ModuleIdentifier, RuntimeSpec,
};

impl DatabaseItem for ChunkGroup {
  fn ukey(&self) -> rspack_database::Ukey<Self> {
    self.ukey
  }
}

#[derive(Debug, Clone)]
pub struct ChunkGroup {
  pub ukey: ChunkGroupUkey,
  pub chunks: Vec<ChunkUkey>,
  pub options: ChunkGroupOptions,
  pub info: ChunkGroupInfo,
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
  pub(crate) runtime_chunk: Option<ChunkUkey>,
  pub(crate) entry_point_chunk: Option<ChunkUkey>,
}

impl ChunkGroup {
  pub fn new(
    kind: ChunkGroupKind,
    runtime: RuntimeSpec,
    group_options: ChunkGroupOptions,
    info: ChunkGroupInfo,
  ) -> Self {
    Self {
      ukey: ChunkGroupUkey::new(),
      chunks: vec![],
      options: group_options,
      info,
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

  pub fn parents_iterable(&self) -> impl Iterator<Item = &ChunkGroupUkey> {
    self.parents.iter()
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

  pub fn is_initial(&self) -> bool {
    matches!(self.kind, ChunkGroupKind::Entrypoint { initial } if initial)
  }

  pub fn set_runtime_chunk(&mut self, chunk_ukey: ChunkUkey) {
    self.runtime_chunk = Some(chunk_ukey);
  }

  pub fn get_runtime_chunk(&self) -> ChunkUkey {
    match self.kind {
      ChunkGroupKind::Entrypoint { .. } => self
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
      ChunkGroupKind::Entrypoint { .. } => self
        .entry_point_chunk
        .expect("EntryPoint runtime chunk not set"),
      ChunkGroupKind::Normal => unreachable!("Normal chunk group doesn't have runtime chunk"),
    }
  }

  pub fn ancestors(&self, chunk_group_by_ukey: &ChunkGroupByUkey) -> HashSet<ChunkGroupUkey> {
    let mut queue = vec![];
    let mut ancestors = HashSet::default();

    queue.extend(self.parents.iter().copied());

    while let Some(chunk_group_ukey) = queue.pop() {
      if ancestors.contains(&chunk_group_ukey) {
        continue;
      }
      ancestors.insert(chunk_group_ukey);
      let chunk_group = chunk_group_by_ukey
        .get(&chunk_group_ukey)
        .expect("should have chunk group");
      for parent in &chunk_group.parents {
        queue.push(*parent);
      }
    }

    ancestors
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

  pub fn id(&self, compilation: &Compilation) -> String {
    self
      .chunks
      .iter()
      .filter_map(|chunk| {
        compilation
          .chunk_by_ukey
          .get(chunk)
          .and_then(|chunk| chunk.id.as_ref())
      })
      .join("+")
  }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChunkGroupKind {
  Entrypoint { initial: bool },
  Normal,
}

impl ChunkGroupKind {
  pub fn new_entrypoint(initial: bool) -> Self {
    Self::Entrypoint { initial }
  }

  pub fn is_entrypoint(&self) -> bool {
    matches!(self, Self::Entrypoint { .. })
  }
}

// TODO: split ChunkGroupOptions and EntryOptions, put options on kind
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct ChunkGroupOptions {
  pub name: Option<String>,
  pub entry_options: Option<EntryOptions>,
}

impl ChunkGroupOptions {
  pub fn name(mut self, v: impl Into<String>) -> Self {
    self.name = Some(v.into());
    self
  }

  pub fn name_optional<T: Into<String>>(mut self, v: Option<T>) -> Self {
    self.name = v.map(|v| v.into());
    self
  }

  pub fn entry_options(mut self, v: impl Into<EntryOptions>) -> Self {
    self.entry_options = Some(v.into());
    self
  }

  pub fn entry_options_optional<T: Into<EntryOptions>>(mut self, v: Option<T>) -> Self {
    self.entry_options = v.map(|v| v.into());
    self
  }
}

#[derive(Debug, Default, Clone)]
pub struct ChunkGroupInfo {
  pub chunk_loading: bool,
  pub async_chunks: bool,
}
