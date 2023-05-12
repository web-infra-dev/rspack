use std::{
  fmt::{Debug, Formatter, Result},
  hash::Hasher,
};

use rspack_database::DatabaseItem;
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};
use xxhash_rust::xxh3::Xxh3;

use crate::{
  ChunkGraph, ChunkGroupByUkey, ChunkGroupKind, ChunkGroupUkey, ChunkUkey, ModuleGraph,
  RuntimeSpec, SourceType,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChunkKind {
  HotUpdate,
  Normal,
}

#[derive(Clone)]
pub struct Chunk {
  // - If the chunk is create by entry, the name is the entry name
  // - (Rspack doesn't support it yet)If the chunk is create by dynamic import, the name
  // is the valid in MagicComment `import(/* webpackChunkName: "someChunk" * / './someModule.js')`.
  // - TODO: HMR chunk will have name. Not sure this is expected. Need to discuss with underfin
  pub name: Option<String>,
  pub ukey: ChunkUkey,
  pub id: Option<String>,
  pub ids: Vec<String>,
  pub id_name_hints: HashSet<String>,
  pub files: HashSet<String>,
  pub groups: HashSet<ChunkGroupUkey>,
  pub runtime: RuntimeSpec,
  pub kind: ChunkKind,
  pub hash: Xxh3,
  pub content_hash: HashMap<SourceType, String>,
  pub chunk_reasons: Vec<String>,
}

impl DatabaseItem for Chunk {
  fn ukey(&self) -> rspack_database::Ukey<Self> {
    self.ukey
  }
}

impl Debug for Chunk {
  fn fmt(&self, f: &mut Formatter<'_>) -> Result {
    f.debug_struct("Chunk")
      .field("name", &self.name)
      .field("ukey", &self.ukey)
      .field("id", &self.id)
      .field("files", &self.files)
      .field("groups", &self.groups)
      .field("runtime", &self.runtime)
      .field("kind", &self.kind)
      .field("hash", &self.hash.finish())
      .field("content_hash", &self.content_hash)
      .finish()
  }
}

impl Chunk {
  pub fn new(name: Option<String>, id: Option<String>, kind: ChunkKind) -> Self {
    Self {
      name,
      ukey: ChunkUkey::new(),
      id,
      ids: vec![],
      id_name_hints: Default::default(),
      files: Default::default(),
      groups: Default::default(),
      runtime: HashSet::default(),
      kind,
      hash: Default::default(),
      content_hash: HashMap::default(),
      chunk_reasons: Default::default(),
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
      group.insert_chunk(new_chunk.ukey, self.ukey);
      new_chunk.add_group(group.ukey);
    });
    new_chunk.id_name_hints.extend(self.id_name_hints.clone());
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
    chunk_graph.get_number_of_entry_modules(&self.ukey) > 0
  }

  pub fn get_all_referenced_chunks(
    &self,
    chunk_group_by_ukey: &ChunkGroupByUkey,
  ) -> HashSet<ChunkUkey> {
    let mut chunks: std::collections::HashSet<
      rspack_database::Ukey<Chunk>,
      std::hash::BuildHasherDefault<rustc_hash::FxHasher>,
    > = HashSet::default();
    let mut visit_chunk_groups = HashSet::default();

    fn add_chunks(
      chunk_group_ukey: &ChunkGroupUkey,
      chunks: &mut HashSet<ChunkUkey>,
      chunk_group_by_ukey: &ChunkGroupByUkey,
      visit_chunk_groups: &mut HashSet<ChunkGroupUkey>,
    ) {
      let group = chunk_group_by_ukey
        .get(chunk_group_ukey)
        .expect("Group should exist");

      for chunk_ukey in group.chunks.iter() {
        chunks.insert(*chunk_ukey);
      }

      for child_group_ukey in group.children.iter() {
        if !visit_chunk_groups.contains(child_group_ukey) {
          visit_chunk_groups.insert(*child_group_ukey);
          add_chunks(
            child_group_ukey,
            chunks,
            chunk_group_by_ukey,
            visit_chunk_groups,
          );
        }
      }
    }

    for group_ukey in &self.groups {
      visit_chunk_groups.insert(*group_ukey);
      add_chunks(
        group_ukey,
        &mut chunks,
        chunk_group_by_ukey,
        &mut visit_chunk_groups,
      );
    }

    chunks
  }

  pub fn get_all_initial_chunks(
    &self,
    chunk_group_by_ukey: &ChunkGroupByUkey,
  ) -> HashSet<ChunkUkey> {
    let mut chunks = HashSet::default();
    let mut visit_chunk_groups = HashSet::default();

    fn add_chunks(
      chunk_group_ukey: &ChunkGroupUkey,
      chunks: &mut HashSet<ChunkUkey>,
      chunk_group_by_ukey: &ChunkGroupByUkey,
      visit_chunk_groups: &mut HashSet<ChunkGroupUkey>,
    ) {
      let group = chunk_group_by_ukey
        .get(chunk_group_ukey)
        .expect("Group should exist");

      if group.is_initial() {
        for chunk_ukey in group.chunks.iter() {
          chunks.insert(*chunk_ukey);
        }

        for child_group_ukey in group.children.iter() {
          if !visit_chunk_groups.contains(child_group_ukey) {
            visit_chunk_groups.insert(*child_group_ukey);
            add_chunks(
              child_group_ukey,
              chunks,
              chunk_group_by_ukey,
              visit_chunk_groups,
            );
          }
        }
      }
    }

    for group_ukey in &self.groups {
      add_chunks(
        group_ukey,
        &mut chunks,
        chunk_group_by_ukey,
        &mut visit_chunk_groups,
      );
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
    let mut queue = HashSet::default();
    let mut chunks = HashSet::default();
    let initial_chunks: HashSet<ChunkUkey> = self
      .groups
      .iter()
      .filter_map(|ukey| chunk_group_by_ukey.get(ukey))
      .flat_map(|chunk_group| chunk_group.chunks.iter())
      .cloned()
      .collect();
    let mut initial_queue = self.groups.clone();
    let mut visit_chunk_groups = HashSet::default();

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
      chunks: &mut HashSet<ChunkUkey>,
      initial_chunks: &HashSet<ChunkUkey>,
      chunk_group_ukey: &ChunkGroupUkey,
      visit_chunk_groups: &mut HashSet<ChunkGroupUkey>,
    ) {
      if let Some(chunk_group) = chunk_group_by_ukey.get(chunk_group_ukey) {
        for chunk_ukey in chunk_group.chunks.iter() {
          if !initial_chunks.contains(chunk_ukey) {
            chunks.insert(*chunk_ukey);
          }
        }

        for group_ukey in chunk_group.children.iter() {
          if !visit_chunk_groups.contains(group_ukey) {
            visit_chunk_groups.insert(*group_ukey);
            add_chunks(
              chunk_group_by_ukey,
              chunks,
              initial_chunks,
              group_ukey,
              visit_chunk_groups,
            );
          }
        }
      }
    }

    for group_ukey in queue.iter() {
      add_chunks(
        chunk_group_by_ukey,
        &mut chunks,
        &initial_chunks,
        group_ukey,
        &mut visit_chunk_groups,
      );
    }

    chunks
  }

  // pub fn get_all_referenced_async_entry_points() -> HashSet<ChunkUkey> {}

  pub fn get_render_hash(&self) -> String {
    format!("{:016x}", self.hash.finish())
  }

  pub fn expect_id(&self) -> &str {
    self
      .id
      .as_ref()
      .expect("Should set id before calling expect_id")
  }

  pub fn name_for_filename_template(&self) -> Option<&str> {
    if self.name.is_some() {
      self.name.as_deref()
    } else {
      self.id.as_deref()
    }
  }

  pub fn is_in_group(&self, chunk_group: &ChunkGroupUkey) -> bool {
    self.groups.contains(chunk_group)
  }

  pub fn disconnect_from_groups(&mut self, chunk_group_by_ukey: &mut ChunkGroupByUkey) {
    for group_ukey in self.groups.iter() {
      let group = chunk_group_by_ukey
        .get_mut(group_ukey)
        .expect("Group should exist");
      group.remove_chunk(&self.ukey);
    }
    self.groups.clear();
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
