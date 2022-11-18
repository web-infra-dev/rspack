use hashbrown::HashSet;

use crate::{
  ChunkGraph, ChunkGroupByUkey, ChunkGroupKind, ChunkGroupUkey, ChunkUkey, ModuleGraph,
  RuntimeSpec, SourceType,
};

#[derive(Debug, PartialEq, Eq)]
pub enum ChunkKind {
  HotUpdate,
  Normal,
}

#[derive(Debug)]
pub struct Chunk {
  pub(crate) _name: Option<String>,
  pub ukey: ChunkUkey,
  pub id: String,
  pub files: HashSet<String>,
  pub groups: HashSet<ChunkGroupUkey>,
  pub runtime: RuntimeSpec,
  pub kind: ChunkKind,
  pub hash: String,
}

impl Chunk {
  pub fn new(_name: Option<String>, id: String, kind: ChunkKind) -> Self {
    Self {
      _name,
      ukey: ChunkUkey::with_debug_info("Chunk"),
      id,
      files: Default::default(),
      groups: Default::default(),
      runtime: HashSet::default(),
      kind,
      hash: Default::default(),
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

    fn add_chunks(
      chunk_group_ukey: &ChunkGroupUkey,
      chunks: &mut HashSet<ChunkUkey>,
      chunk_group_by_ukey: &ChunkGroupByUkey,
    ) {
      let group = chunk_group_by_ukey
        .get(chunk_group_ukey)
        .expect("Group should exist");

      for chunk_ukey in group.chunks.iter() {
        chunks.insert(*chunk_ukey);
      }

      for child_group_ukey in group.children.iter() {
        add_chunks(child_group_ukey, chunks, chunk_group_by_ukey);
      }
    }

    for group_ukey in &self.groups {
      add_chunks(group_ukey, &mut chunks, chunk_group_by_ukey);
    }

    chunks
  }

  pub fn get_all_initial_chunks(
    &self,
    chunk_group_by_ukey: &ChunkGroupByUkey,
  ) -> HashSet<ChunkUkey> {
    let mut chunks = HashSet::new();

    fn add_chunks(
      chunk_group_ukey: &ChunkGroupUkey,
      chunks: &mut HashSet<ChunkUkey>,
      chunk_group_by_ukey: &ChunkGroupByUkey,
    ) {
      let group = chunk_group_by_ukey
        .get(chunk_group_ukey)
        .expect("Group should exist");

      if group.is_initial() {
        for chunk_ukey in group.chunks.iter() {
          chunks.insert(*chunk_ukey);
        }

        for child_group_ukey in group.children.iter() {
          add_chunks(child_group_ukey, chunks, chunk_group_by_ukey);
        }
      }
    }

    for group_ukey in &self.groups {
      add_chunks(group_ukey, &mut chunks, chunk_group_by_ukey);
    }

    chunks
  }

  pub fn has_runtime(&self, chunk_group_by_ukey: &ChunkGroupByUkey) -> bool {
    self
      .groups
      .iter()
      .filter_map(|ukey| chunk_group_by_ukey.get(ukey))
      .any(|group| {
        group.kind == ChunkGroupKind::Entrypoint && group.get_runtime_chunk() == self.ukey
      })
  }

  pub fn get_all_async_chunks(&self, chunk_group_by_ukey: &ChunkGroupByUkey) -> HashSet<ChunkUkey> {
    let mut queue = HashSet::new();
    let mut chunks = HashSet::new();
    let initial_chunks: HashSet<ChunkUkey> = self
      .groups
      .iter()
      .filter_map(|ukey| chunk_group_by_ukey.get(ukey))
      .flat_map(|chunk_group| chunk_group.chunks.iter())
      .cloned()
      .collect();
    let mut initial_queue = self.groups.clone();

    fn add_to_queue(
      chunk_group_by_ukey: &ChunkGroupByUkey,
      queue: &mut HashSet<ChunkGroupUkey>,
      initial_queue: &mut HashSet<ChunkGroupUkey>,
      chunk_group_ukey: &ChunkGroupUkey,
    ) {
      if let Some(chunk_group) = chunk_group_by_ukey.get(chunk_group_ukey) {
        for child_ukey in chunk_group.children.iter() {
          if let Some(chunk_group) = chunk_group_by_ukey.get(child_ukey) {
            if chunk_group.is_initial() && !initial_queue.contains(&chunk_group.ukey) {
              initial_queue.insert(chunk_group.ukey);
              add_to_queue(chunk_group_by_ukey, queue, initial_queue, &chunk_group.ukey);
            } else {
              queue.insert(chunk_group.ukey);
            }
          }
        }
      }
    }

    for chunk_group_ukey in initial_queue.clone().iter() {
      add_to_queue(
        chunk_group_by_ukey,
        &mut queue,
        &mut initial_queue,
        chunk_group_ukey,
      );
    }

    fn add_chunks(
      chunk_group_by_ukey: &ChunkGroupByUkey,
      chunks: &mut HashSet<ChunkGroupUkey>,
      initial_chunks: &HashSet<ChunkGroupUkey>,
      chunk_group_ukey: &ChunkGroupUkey,
    ) {
      if let Some(chunk_group) = chunk_group_by_ukey.get(chunk_group_ukey) {
        for chunk_ukey in chunk_group.chunks.iter() {
          if !initial_chunks.contains(chunk_ukey) {
            chunks.insert(*chunk_ukey);
          }
        }

        for group_ukey in chunk_group.children.iter() {
          add_chunks(chunk_group_by_ukey, chunks, initial_chunks, group_ukey);
        }
      }
    }

    for group_ukey in queue.iter() {
      add_chunks(
        chunk_group_by_ukey,
        &mut chunks,
        &initial_chunks,
        group_ukey,
      );
    }

    chunks
  }
}

pub fn chunk_hash_js<'a>(
  chunk: &ChunkUkey,
  chunk_graph: &'a ChunkGraph,
  module_graph: &'a ModuleGraph,
) -> bool {
  if chunk_graph.get_number_of_entry_modules(chunk) > 0 {
    return true;
  }
  if !chunk_graph
    .get_chunk_modules_by_source_type(chunk, SourceType::JavaScript, module_graph)
    .is_empty()
  {
    return true;
  }
  false
}
