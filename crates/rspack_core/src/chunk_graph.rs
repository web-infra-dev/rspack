use hashbrown::{HashMap, HashSet};

use crate::{
  Chunk, ChunkByUkey, ChunkGroupUkey, ChunkUkey, ModuleGraph, ModuleGraphModule, SourceType,
};

#[derive(Debug, Default)]
pub struct ChunkGraph {
  split_point_module_uri_to_chunk_ref: hashbrown::HashMap<String, ChunkUkey>,

  chunk_graph_module_by_module_url: HashMap<String, ChunkGraphModule>,
  chunk_graph_chunk_by_chunk_ukey: HashMap<ChunkUkey, ChunkGraphChunk>,
}

impl ChunkGraph {
  pub fn add_chunk(&mut self, chunk_ukey: ChunkUkey) {
    self
      .chunk_graph_chunk_by_chunk_ukey
      .entry(chunk_ukey)
      .or_insert_with(ChunkGraphChunk::new);
  }
  pub fn add_module(&mut self, module_uri: String) {
    self
      .chunk_graph_module_by_module_url
      .entry(module_uri)
      .or_insert_with(ChunkGraphModule::new);
  }

  pub fn chunk_by_split_point_module_uri<'a>(
    &self,
    uri: &str,
    chunk_by_ukey: &'a ChunkByUkey,
  ) -> Option<&'a Chunk> {
    let ukey = self.split_point_module_uri_to_chunk_ref.get(uri)?;
    chunk_by_ukey.get(ukey)
  }

  pub fn get_chunk_entry_modules(&self, chunk_ukey: &ChunkUkey) -> HashSet<&String> {
    let cgc = self.get_chunk_graph_chunk(chunk_ukey);

    cgc.entry_modules.keys().collect()
  }

  pub(crate) fn is_module_in_chunk(&mut self, module_uri: &str, chunk_ukey: ChunkUkey) -> bool {
    let cgc = self.get_chunk_graph_chunk_mut(chunk_ukey);
    cgc.modules.contains(module_uri)
  }

  pub(crate) fn get_chunk_graph_module_mut(&mut self, module_uri: &str) -> &mut ChunkGraphModule {
    self
      .chunk_graph_module_by_module_url
      .get_mut(module_uri)
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
    module_uri: String,
    entrypoint: ChunkGroupUkey,
  ) {
    let cgm = self.get_chunk_graph_module_mut(&module_uri);
    cgm.entry_in_chunks.insert(chunk);

    let cgc = self.get_chunk_graph_chunk_mut(chunk);
    cgc.entry_modules.insert(module_uri, entrypoint);
  }

  pub(crate) fn disconnect_chunk_and_module(&mut self, chunk: &ChunkUkey, module_uri: &str) {
    let cgm = self.get_chunk_graph_module_mut(module_uri);
    cgm.chunks.remove(chunk);

    let cgc = self.get_chunk_graph_chunk_mut(*chunk);
    cgc.modules.remove(module_uri);
  }

  pub(crate) fn connect_chunk_and_module(&mut self, chunk: ChunkUkey, module_uri: String) {
    let cgm = self.get_chunk_graph_module_mut(&module_uri);
    cgm.chunks.insert(chunk);

    let cgc = self.get_chunk_graph_chunk_mut(chunk);
    cgc.modules.insert(module_uri);
  }

  pub fn get_modules_chunks(&self, module_uri: &str) -> &HashSet<ChunkUkey> {
    let cgm = self
      .chunk_graph_module_by_module_url
      .get(module_uri)
      .expect("Module should be added before");
    &cgm.chunks
  }

  pub fn get_chunk_modules<'module>(
    &self,
    chunk: &ChunkUkey,
    module_graph: &'module ModuleGraph,
  ) -> Vec<&'module ModuleGraphModule> {
    let cgc = self.get_chunk_graph_chunk(chunk);
    cgc
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
    let cgc = self.get_chunk_graph_chunk(chunk);
    let modules = cgc
      .modules
      .iter()
      .filter_map(|uri| module_graph.module_by_uri(uri))
      .filter(|mgm| mgm.module.source_types().contains(&source_type))
      .collect::<Vec<_>>();
    modules
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
  // URI of modules => ChunkGroupUkey
  pub(crate) entry_modules: HashMap<String, ChunkGroupUkey>,
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
