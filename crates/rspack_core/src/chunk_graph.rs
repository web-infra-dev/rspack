use hashbrown::{HashMap, HashSet};

use crate::{
  Chunk, ChunkByUkey, ChunkGroupByUkey, ChunkGroupUkey, ChunkUkey, Module, ModuleGraph,
  ModuleGraphModule, ModuleIdentifier, RuntimeModule, RuntimeSpec, RuntimeSpecMap, RuntimeSpecSet,
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

  pub fn connect_chunk_and_runtime_module(
    &mut self,
    chunk: ChunkUkey,
    module: Box<dyn RuntimeModule>,
  ) {
    let cgm = self.get_chunk_graph_module_mut(&module.identifier());
    cgm.runtime_in_chunks.insert(chunk);

    let cgc = self.get_chunk_graph_chunk_mut(chunk);
    if cgc
      .runtime_modules
      .iter()
      .any(|m| m.identifier() == module.identifier())
    {
      cgc.runtime_modules.push(module);
    }
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
      .filter_map(|uri| module_graph.module_graph_module_by_identifier(uri))
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
      .filter_map(|uri| module_graph.module_graph_module_by_identifier(uri))
      .filter(|mgm| {
        module_graph
          .module_by_identifier(&mgm.module_identifier)
          .map(|module| module.source_types().contains(&source_type))
          .unwrap_or_default()
      })
      .collect::<Vec<_>>();
    modules
  }

  pub fn get_chunk_modules_iterable_by_source_type<'module, 'me: 'module>(
    &'me self,
    chunk: &ChunkUkey,
    source_type: SourceType,
    module_graph: &'module ModuleGraph,
  ) -> impl Iterator<Item = &'module dyn Module> + 'module {
    let chunk_graph_chunk = self.get_chunk_graph_chunk(chunk);
    chunk_graph_chunk
      .modules
      .iter()
      .filter_map(|uri| module_graph.module_by_identifier(uri))
      .filter(move |module| module.source_types().contains(&source_type))
      .map(|m| m.as_ref())
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

  pub fn get_number_of_module_chunks(&self, module_identifier: &ModuleIdentifier) -> usize {
    let cgm = self.get_chunk_graph_module(module_identifier);
    cgm.chunks.len()
  }

  pub fn get_number_of_entry_modules(&self, chunk: &ChunkUkey) -> usize {
    let cgc = self.get_chunk_graph_chunk(chunk);
    cgc.entry_modules.len()
  }

  pub fn add_module_runtime_requirements(
    &mut self,
    module_identifier: &str,
    runtime: &RuntimeSpec,
    runtime_requirements: HashSet<String>,
  ) {
    let mut cgm = self.get_chunk_graph_module_mut(module_identifier);

    if let Some(runtime_requirements_map) = &mut cgm.runtime_requirements {
      if let Some(value) = runtime_requirements_map.get_mut(runtime) {
        value.extend(runtime_requirements);
      } else {
        runtime_requirements_map.set(runtime.clone(), runtime_requirements);
      }
    } else {
      let mut runtime_requirements_map = RuntimeSpecMap::default();
      runtime_requirements_map.set(runtime.clone(), runtime_requirements);
      cgm.runtime_requirements = Some(runtime_requirements_map);
    }
  }

  pub fn add_chunk_runtime_requirements(
    &mut self,
    chunk_ukey: &ChunkUkey,
    runtime_requirements: HashSet<String>,
  ) {
    let cgc = self.get_chunk_graph_chunk_mut(*chunk_ukey);
    cgc.runtime_requirements.extend(runtime_requirements);
  }

  pub fn add_tree_runtime_requirements(
    &mut self,
    chunk_ukey: &ChunkUkey,
    runtime_requirements: HashSet<String>,
  ) {
    self.add_chunk_runtime_requirements(chunk_ukey, runtime_requirements);
  }

  pub fn get_module_runtime_requirements(
    &self,
    module_identifier: &str,
    _runtime: &RuntimeSpec,
  ) -> Option<&HashSet<String>> {
    let cgm = self.get_chunk_graph_module(module_identifier);
    if let Some(runtime_requirements) = &cgm.runtime_requirements {
      if let Some(runtime_requirements) = runtime_requirements.get(_runtime) {
        return Some(runtime_requirements);
      }
    }
    None
  }

  pub fn get_chunk_runtime_requirements(&self, chunk_ukey: &ChunkUkey) -> &HashSet<String> {
    let cgc = self.get_chunk_graph_chunk(chunk_ukey);
    &cgc.runtime_requirements
  }

  pub fn get_tree_runtime_requirements(&self, chunk_ukey: &ChunkUkey) -> &HashSet<String> {
    self.get_chunk_runtime_requirements(chunk_ukey)
  }

  pub fn get_module_runtimes(
    &self,
    module_identifier: &ModuleIdentifier,
    chunk_by_ukey: &ChunkByUkey,
  ) -> RuntimeSpecSet {
    let cgm = self.get_chunk_graph_module(module_identifier);
    let mut runtimes = RuntimeSpecSet::default();
    for chunk_ukey in cgm.chunks.iter() {
      let chunk = chunk_by_ukey.get(chunk_ukey).expect("Chunk should exist");
      runtimes.set(chunk.runtime.clone());
    }
    runtimes
  }

  pub fn get_chunk_runtime_modules_in_order(
    &self,
    chunk_ukey: &ChunkUkey,
  ) -> &Vec<Box<dyn RuntimeModule>> {
    let cgc = self.get_chunk_graph_chunk(chunk_ukey);
    &cgc.runtime_modules
  }

  pub fn set_module_hashes(
    &mut self,
    module_identifier: &ModuleIdentifier,
    runtime: &RuntimeSpec,
    hash: String,
  ) {
    let cgm = self.get_chunk_graph_module_mut(module_identifier);

    if let Some(runtime_spec_map) = &mut cgm.hashes {
      if let Some(value) = runtime_spec_map.get(runtime) {
        unreachable!("Hash for runtime already set: {}", value);
      } else {
        runtime_spec_map.set(runtime.clone(), hash);
      }
    } else {
      let mut runtime_spec_map = RuntimeSpecMap::default();
      runtime_spec_map.set(runtime.clone(), hash);
      cgm.hashes = Some(runtime_spec_map);
    }
  }

  pub fn get_module_hash(
    &mut self,
    module_identifier: &ModuleIdentifier,
    runtime: &RuntimeSpec,
  ) -> Option<&String> {
    let cgm = self.get_chunk_graph_module(module_identifier);
    if let Some(runtime_spec_map) = &cgm.hashes {
      if let Some(value) = runtime_spec_map.get(runtime) {
        return Some(value);
      }
    }
    None
  }

  pub fn get_chunk_condition_map<F: Fn(&ChunkUkey, &ChunkGraph, &ModuleGraph) -> bool>(
    &self,
    chunk_ukey: &ChunkUkey,
    chunk_by_ukey: &ChunkByUkey,
    chunk_group_by_ukey: &ChunkGroupByUkey,
    module_graph: &ModuleGraph,
    filter: F,
  ) -> HashMap<String, bool> {
    let mut map = HashMap::new();

    let chunk = chunk_by_ukey.get(chunk_ukey).expect("Chunk should exist");
    for c in chunk.get_all_referenced_chunks(chunk_group_by_ukey).iter() {
      let chunk = chunk_by_ukey.get(c).expect("Chunk should exist");
      map.insert(chunk.id.clone(), filter(c, self, module_graph));
    }

    map
  }
}

#[derive(Debug, Default)]
pub struct ChunkGraphModule {
  pub(crate) entry_in_chunks: HashSet<ChunkUkey>,
  pub(crate) chunks: HashSet<ChunkUkey>,
  pub(crate) runtime_requirements: Option<RuntimeSpecMap<HashSet<String>>>,
  pub(crate) runtime_in_chunks: HashSet<ChunkUkey>,
  pub(crate) hashes: Option<RuntimeSpecMap<String>>,
}

impl ChunkGraphModule {
  pub fn new() -> Self {
    Self {
      entry_in_chunks: Default::default(),
      chunks: Default::default(),
      runtime_requirements: None,
      runtime_in_chunks: Default::default(),
      hashes: None,
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
  pub(crate) runtime_requirements: HashSet<String>,
  pub(crate) runtime_modules: Vec<Box<dyn RuntimeModule>>,
}

impl ChunkGraphChunk {
  pub fn new() -> Self {
    Self {
      entry_modules: Default::default(),
      modules: Default::default(),
      runtime_requirements: HashSet::default(),
      runtime_modules: Default::default(),
    }
  }
}
