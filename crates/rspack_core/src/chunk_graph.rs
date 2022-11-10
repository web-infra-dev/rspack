use hashbrown::{HashMap, HashSet};

use crate::{
  Chunk, ChunkByUkey, ChunkGroupUkey, ChunkUkey, ModuleGraph, ModuleGraphModule, ModuleIdentifier,
  SourceType,
};

#[derive(Debug, Default)]
pub struct ChunkGraph {
  pub(crate) split_point_module_identifier_to_chunk_ukey:
    hashbrown::HashMap<ModuleIdentifier, ChunkUkey>,

  chunk_graph_module_by_module_identifier: HashMap<ModuleIdentifier, ChunkGraphModule>,
  chunk_graph_chunk_by_chunk_ukey: HashMap<ChunkUkey, ChunkGraphChunk>,
}

impl ChunkGraph {
  pub fn add_chunk(&mut self, chunk_ukey: ChunkUkey) {
    self
      .chunk_graph_chunk_by_chunk_ukey
      .entry(chunk_ukey)
      .or_insert_with(ChunkGraphChunk::new);
  }
  pub fn add_module(&mut self, module_identifier: String) {
    self
      .chunk_graph_module_by_module_identifier
      .entry(module_identifier)
      .or_insert_with(ChunkGraphModule::new);
  }

  pub fn chunk_by_split_point_module_identifier<'a>(
    &self,
    uri: &str,
    chunk_by_ukey: &'a ChunkByUkey,
  ) -> Option<&'a Chunk> {
    let ukey = self.split_point_module_identifier_to_chunk_ukey.get(uri)?;
    chunk_by_ukey.get(ukey)
  }

  pub fn get_chunk_entry_modules(&self, chunk_ukey: &ChunkUkey) -> Vec<&String> {
    let chunk_graph_chunk = self.get_chunk_graph_chunk(chunk_ukey);

    chunk_graph_chunk.entry_modules.keys().collect()
  }

  pub fn is_module_in_chunk(&mut self, module_identifier: &str, chunk_ukey: ChunkUkey) -> bool {
    let chunk_graph_chunk = self.get_chunk_graph_chunk_mut(chunk_ukey);
    chunk_graph_chunk.modules.contains(module_identifier)
  }

  pub(crate) fn get_chunk_graph_module_mut(
    &mut self,
    module_identifier: &str,
  ) -> &mut ChunkGraphModule {
    self
      .chunk_graph_module_by_module_identifier
      .get_mut(module_identifier)
      .expect("Module should be added before")
  }

  pub(crate) fn get_chunk_graph_module(&self, module_identifier: &str) -> &ChunkGraphModule {
    self
      .chunk_graph_module_by_module_identifier
      .get(module_identifier)
      .expect("Module should be added before")
  }

  pub(crate) fn get_chunk_graph_chunk_mut(
    &mut self,
    chunk_ukey: ChunkUkey,
  ) -> &mut ChunkGraphChunk {
    self
      .chunk_graph_chunk_by_chunk_ukey
      .get_mut(&chunk_ukey)
      .expect("Chunk should be added before")
  }

  pub(crate) fn get_chunk_graph_chunk(&self, chunk_ukey: &ChunkUkey) -> &ChunkGraphChunk {
    self
      .chunk_graph_chunk_by_chunk_ukey
      .get(chunk_ukey)
      .expect("Chunk should be added before")
  }

  pub(crate) fn connect_chunk_and_entry_module(
    &mut self,
    chunk: ChunkUkey,
    module_identifier: String,
    entrypoint: ChunkGroupUkey,
  ) {
    let chunk_graph_module = self.get_chunk_graph_module_mut(&module_identifier);
    chunk_graph_module.entry_in_chunks.insert(chunk);

    let chunk_graph_chunk = self.get_chunk_graph_chunk_mut(chunk);
    chunk_graph_chunk
      .entry_modules
      .insert(module_identifier, entrypoint);
  }

  pub fn disconnect_chunk_and_module(&mut self, chunk: &ChunkUkey, module_identifier: &str) {
    let chunk_graph_module = self.get_chunk_graph_module_mut(module_identifier);
    chunk_graph_module.chunks.remove(chunk);

    let chunk_graph_chunk = self.get_chunk_graph_chunk_mut(*chunk);
    chunk_graph_chunk.modules.remove(module_identifier);
  }

  pub fn connect_chunk_and_module(&mut self, chunk: ChunkUkey, module_identifier: String) {
    let chunk_graph_module = self.get_chunk_graph_module_mut(&module_identifier);
    chunk_graph_module.chunks.insert(chunk);

    let chunk_graph_chunk = self.get_chunk_graph_chunk_mut(chunk);
    chunk_graph_chunk.modules.insert(module_identifier);
  }

  pub fn get_modules_chunks(&self, module_identifier: &str) -> &HashSet<ChunkUkey> {
    let chunk_graph_module = self
      .chunk_graph_module_by_module_identifier
      .get(module_identifier)
      .expect("Module should be added before");
    &chunk_graph_module.chunks
  }

  pub fn get_module_chunk_group<'a>(
    &self,
    module_identifier: &str,
    chunk_by_ukey: &'a ChunkByUkey,
  ) -> &'a ChunkGroupUkey {
    let chunk = self
      .chunk_by_split_point_module_identifier(module_identifier, chunk_by_ukey)
      .expect("Chunk should be added before");
    chunk
      .groups
      .iter()
      .next()
      .expect("Chunk should have at least one group")
  }

  pub fn get_chunk_modules<'module>(
    &self,
    chunk: &ChunkUkey,
    module_graph: &'module ModuleGraph,
  ) -> Vec<&'module ModuleGraphModule> {
    let chunk_graph_chunk = self.get_chunk_graph_chunk(chunk);
    chunk_graph_chunk
      .modules
      .iter()
      .filter_map(|uri| module_graph.module_by_uri(uri))
      .collect()
  }

  pub fn get_chunk_modules_by_source_type<'module>(
    &self,
    chunk: &ChunkUkey,
    source_type: SourceType,
    module_graph: &'module ModuleGraph,
  ) -> Vec<&'module ModuleGraphModule> {
    let chunk_graph_chunk = self.get_chunk_graph_chunk(chunk);
    let modules = chunk_graph_chunk
      .modules
      .iter()
      .filter_map(|uri| module_graph.module_by_uri(uri))
      .filter(|mgm| {
        module_graph
          .module_by_identifier(&mgm.module_identifier)
          .map(|module| module.source_types().contains(&source_type))
          .unwrap_or_default()
      })
      .collect::<Vec<_>>();
    modules
  }

  pub fn get_chunk_modules_size<'module>(
    &self,
    chunk: &ChunkUkey,
    module_graph: &'module ModuleGraph,
  ) -> f64 {
    self
      .get_chunk_modules(chunk, module_graph)
      .iter()
      .fold(0.0, |acc, m| {
        let module = module_graph
          .module_by_identifier(&m.module_identifier)
          .unwrap();
        acc
          + module
            .source_types()
            .iter()
            .fold(0.0, |acc, t| acc + module.size(t))
      })
  }
}

#[derive(Debug, Default)]
pub struct ChunkGraphModule {
  pub(crate) entry_in_chunks: HashSet<ChunkUkey>,
  pub(crate) chunks: HashSet<ChunkUkey>,
}

impl ChunkGraphModule {
  pub fn new() -> Self {
    Self {
      entry_in_chunks: Default::default(),
      chunks: Default::default(),
    }
  }
}

#[derive(Debug, Default)]
pub struct ChunkGraphChunk {
  /// URI of modules => ChunkGroupUkey
  ///
  /// use `LinkedHashMap` to keep the ordered from entry array.
  pub(crate) entry_modules: hashlink::LinkedHashMap<String, ChunkGroupUkey>,
  pub(crate) modules: HashSet<String>,
}

impl ChunkGraphChunk {
  pub fn new() -> Self {
    Self {
      entry_modules: Default::default(),
      modules: Default::default(),
    }
  }
}
