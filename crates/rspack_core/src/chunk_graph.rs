use rspack_identifier::{IdentifierLinkedMap, IdentifierMap, IdentifierSet};
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};

use crate::{
  find_graph_roots, Chunk, ChunkByUkey, ChunkGroup, ChunkGroupByUkey, ChunkGroupUkey, ChunkUkey,
  Module, ModuleGraph, ModuleGraphModule, ModuleIdentifier, RuntimeSpec, RuntimeSpecMap,
  RuntimeSpecSet, SourceType,
};

#[derive(Debug, Clone, Default)]
pub struct ChunkGraph {
  pub split_point_module_identifier_to_chunk_ukey: IdentifierMap<ChunkUkey>,

  /// If a module is imported dynamically, it will be assigned to a unique ChunkGroup
  pub(crate) block_to_chunk_group_ukey: IdentifierMap<ChunkGroupUkey>,

  pub chunk_graph_module_by_module_identifier: IdentifierMap<ChunkGraphModule>,
  chunk_graph_chunk_by_chunk_ukey: HashMap<ChunkUkey, ChunkGraphChunk>,
}

impl ChunkGraph {
  pub fn add_chunk(&mut self, chunk_ukey: ChunkUkey) {
    self
      .chunk_graph_chunk_by_chunk_ukey
      .entry(chunk_ukey)
      .or_insert_with(ChunkGraphChunk::new);
  }
  pub fn add_chunk_wit_chunk_graph_chunk(&mut self, chunk_ukey: ChunkUkey, cgc: ChunkGraphChunk) {
    debug_assert!(!self
      .chunk_graph_chunk_by_chunk_ukey
      .contains_key(&chunk_ukey));
    self.chunk_graph_chunk_by_chunk_ukey.insert(chunk_ukey, cgc);
  }
  pub fn add_module(&mut self, module_identifier: ModuleIdentifier) {
    self
      .chunk_graph_module_by_module_identifier
      .entry(module_identifier)
      .or_insert_with(ChunkGraphModule::new);
  }

  pub fn get_chunk_entry_modules(&self, chunk_ukey: &ChunkUkey) -> Vec<ModuleIdentifier> {
    let chunk_graph_chunk = self.get_chunk_graph_chunk(chunk_ukey);

    chunk_graph_chunk.entry_modules.keys().cloned().collect()
  }

  pub fn get_chunk_entry_modules_with_chunk_group(
    &self,
    chunk_ukey: &ChunkUkey,
  ) -> HashSet<&ChunkGroupUkey> {
    let chunk_graph_chunk = self.get_chunk_graph_chunk(chunk_ukey);

    chunk_graph_chunk
      .entry_modules
      .iter()
      .map(|(_, chunk_group_ukey)| chunk_group_ukey)
      .collect()
  }

  pub fn is_module_in_chunk(
    &self,
    module_identifier: &ModuleIdentifier,
    chunk_ukey: ChunkUkey,
  ) -> bool {
    let chunk_graph_chunk = self.get_chunk_graph_chunk(&chunk_ukey);
    chunk_graph_chunk.modules.contains(module_identifier)
  }

  pub(crate) fn get_chunk_graph_module_mut(
    &mut self,
    module_identifier: ModuleIdentifier,
  ) -> &mut ChunkGraphModule {
    self
      .chunk_graph_module_by_module_identifier
      .get_mut(&module_identifier)
      .expect("Module should be added before")
  }

  pub(crate) fn get_chunk_graph_module(
    &self,
    module_identifier: ModuleIdentifier,
  ) -> &ChunkGraphModule {
    self
      .chunk_graph_module_by_module_identifier
      .get(&module_identifier)
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
    module_identifier: ModuleIdentifier,
    entrypoint: ChunkGroupUkey,
  ) {
    let chunk_graph_module = self.get_chunk_graph_module_mut(module_identifier);
    chunk_graph_module.entry_in_chunks.insert(chunk);

    let chunk_graph_chunk = self.get_chunk_graph_chunk_mut(chunk);
    chunk_graph_chunk
      .entry_modules
      .insert(module_identifier, entrypoint);
  }

  pub fn disconnect_chunk_and_module(
    &mut self,
    chunk: &ChunkUkey,
    module_identifier: ModuleIdentifier,
  ) {
    let chunk_graph_module = self.get_chunk_graph_module_mut(module_identifier);
    chunk_graph_module.chunks.remove(chunk);

    let chunk_graph_chunk = self.get_chunk_graph_chunk_mut(*chunk);
    chunk_graph_chunk.modules.remove(&module_identifier);
  }

  pub fn connect_chunk_and_module(
    &mut self,
    chunk: ChunkUkey,
    module_identifier: ModuleIdentifier,
  ) {
    let chunk_graph_module = self.get_chunk_graph_module_mut(module_identifier);
    chunk_graph_module.chunks.insert(chunk);

    let chunk_graph_chunk = self.get_chunk_graph_chunk_mut(chunk);
    chunk_graph_chunk.modules.insert(module_identifier);
  }

  pub fn connect_chunk_and_runtime_module(
    &mut self,
    chunk: ChunkUkey,
    identifier: ModuleIdentifier,
  ) {
    let cgm = self.get_chunk_graph_module_mut(identifier);
    cgm.runtime_in_chunks.insert(chunk);

    let cgc = self.get_chunk_graph_chunk_mut(chunk);
    if !cgc.runtime_modules.contains(&identifier) {
      cgc.runtime_modules.push(identifier);
    }
  }

  pub fn get_modules_chunks(&self, module_identifier: ModuleIdentifier) -> &HashSet<ChunkUkey> {
    let chunk_graph_module = self
      .chunk_graph_module_by_module_identifier
      .get(&module_identifier)
      .expect("Module should be added before");
    &chunk_graph_module.chunks
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

  pub fn get_chunk_module_identifiers(&self, chunk: &ChunkUkey) -> &IdentifierSet {
    let chunk_graph_chunk = self.get_chunk_graph_chunk(chunk);
    &chunk_graph_chunk.modules
  }

  pub fn get_ordered_chunk_modules<'module>(
    &self,
    chunk: &ChunkUkey,
    module_graph: &'module ModuleGraph,
  ) -> Vec<&'module ModuleGraphModule> {
    let mut modules = self.get_chunk_modules(chunk, module_graph);
    // SAFETY: module identifier is unique
    modules.sort_unstable_by_key(|m| m.module_identifier.as_str());
    modules
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

  pub fn get_chunk_modules_size(&self, chunk: &ChunkUkey, module_graph: &ModuleGraph) -> f64 {
    self
      .get_chunk_modules(chunk, module_graph)
      .iter()
      .fold(0.0, |acc, m| {
        let module = module_graph
          .module_by_identifier(&m.module_identifier)
          .unwrap_or_else(|| panic!("Module({}) does not exist", m.module_identifier));
        acc
          + module
            .source_types()
            .iter()
            .fold(0.0, |acc, t| acc + module.size(t))
      })
  }

  pub fn get_number_of_module_chunks(&self, module_identifier: ModuleIdentifier) -> usize {
    let cgm = self.get_chunk_graph_module(module_identifier);
    cgm.chunks.len()
  }

  pub fn get_number_of_chunk_modules(&self, chunk: &ChunkUkey) -> usize {
    let cgc = self.get_chunk_graph_chunk(chunk);
    cgc.modules.len()
  }

  pub fn get_number_of_entry_modules(&self, chunk: &ChunkUkey) -> usize {
    let cgc = self.get_chunk_graph_chunk(chunk);
    cgc.entry_modules.len()
  }

  pub fn add_module_runtime_requirements(
    &mut self,
    module_identifier: ModuleIdentifier,
    runtime: &RuntimeSpec,
    runtime_requirements: HashSet<&'static str>,
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
    runtime_requirements: HashSet<&'static str>,
  ) {
    let cgc = self.get_chunk_graph_chunk_mut(*chunk_ukey);
    cgc.runtime_requirements.extend(runtime_requirements);
  }

  pub fn add_tree_runtime_requirements(
    &mut self,
    chunk_ukey: &ChunkUkey,
    runtime_requirements: HashSet<&'static str>,
  ) {
    self.add_chunk_runtime_requirements(chunk_ukey, runtime_requirements);
  }

  pub fn get_module_runtime_requirements(
    &self,
    module_identifier: ModuleIdentifier,
    _runtime: &RuntimeSpec,
  ) -> Option<&HashSet<&'static str>> {
    let cgm = self.get_chunk_graph_module(module_identifier);
    if let Some(runtime_requirements) = &cgm.runtime_requirements {
      if let Some(runtime_requirements) = runtime_requirements.get(_runtime) {
        return Some(runtime_requirements);
      }
    }
    None
  }

  pub fn get_chunk_runtime_requirements(&self, chunk_ukey: &ChunkUkey) -> &HashSet<&'static str> {
    let cgc = self.get_chunk_graph_chunk(chunk_ukey);
    &cgc.runtime_requirements
  }

  pub fn get_tree_runtime_requirements(&self, chunk_ukey: &ChunkUkey) -> &HashSet<&'static str> {
    self.get_chunk_runtime_requirements(chunk_ukey)
  }

  pub fn get_module_runtimes(
    &self,
    module_identifier: ModuleIdentifier,
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
  ) -> &Vec<ModuleIdentifier> {
    let cgc = self.get_chunk_graph_chunk(chunk_ukey);
    &cgc.runtime_modules
  }

  // pub fn set_module_hashes(
  //   &mut self,
  //   module_identifier: ModuleIdentifier,
  //   runtime: &RuntimeSpec,
  //   hash: u64,
  // ) {
  //   let cgm = self.get_chunk_graph_module_mut(module_identifier);

  //   if let Some(runtime_spec_map) = &mut cgm.hashes {
  //     if let Some(value) = runtime_spec_map.get(runtime) {
  //       unreachable!("Hash for runtime already set: {}", value);
  //     } else {
  //       runtime_spec_map.set(runtime.clone(), hash);
  //     }
  //   } else {
  //     let mut runtime_spec_map = RuntimeSpecMap::default();
  //     runtime_spec_map.set(runtime.clone(), hash);
  //     cgm.hashes = Some(runtime_spec_map);
  //   }
  // }

  // pub fn get_module_hash(
  //   &self,
  //   module_identifier: ModuleIdentifier,
  //   runtime: &RuntimeSpec,
  // ) -> Option<&u64> {
  //   let cgm = self.get_chunk_graph_module(module_identifier);
  //   if let Some(runtime_spec_map) = &cgm.hashes {
  //     if let Some(value) = runtime_spec_map.get(runtime) {
  //       return Some(value);
  //     }
  //   }
  //   None
  // }

  pub fn get_chunk_condition_map<F: Fn(&ChunkUkey, &ChunkGraph, &ModuleGraph) -> bool>(
    &self,
    chunk_ukey: &ChunkUkey,
    chunk_by_ukey: &ChunkByUkey,
    chunk_group_by_ukey: &ChunkGroupByUkey,
    module_graph: &ModuleGraph,
    filter: F,
  ) -> HashMap<String, bool> {
    let mut map = HashMap::default();

    let chunk = chunk_by_ukey.get(chunk_ukey).expect("Chunk should exist");
    for c in chunk.get_all_referenced_chunks(chunk_group_by_ukey).iter() {
      let chunk = chunk_by_ukey.get(c).expect("Chunk should exist");
      map.insert(chunk.expect_id().to_string(), filter(c, self, module_graph));
    }

    map
  }

  pub fn get_module_id(&self, module_identifier: ModuleIdentifier) -> &Option<String> {
    let cgm = self.get_chunk_graph_module(module_identifier);
    &cgm.id
  }

  pub fn set_module_id(&mut self, module_identifier: ModuleIdentifier, id: String) {
    let cgm = self.get_chunk_graph_module_mut(module_identifier);
    cgm.id = Some(id);
  }

  pub fn get_chunk_root_modules(
    &self,
    chunk: &ChunkUkey,
    module_graph: &ModuleGraph,
  ) -> Vec<ModuleIdentifier> {
    let cgc = self.get_chunk_graph_chunk(chunk);
    let mut input = cgc.modules.iter().cloned().collect::<Vec<_>>();
    input.sort();
    let mut modules = find_graph_roots(input, |module| {
      let mut set: IdentifierSet = Default::default();
      fn add_dependencies(
        module: ModuleIdentifier,
        set: &mut IdentifierSet,
        module_graph: &ModuleGraph,
      ) {
        let module = module_graph
          .module_by_identifier(&module)
          .expect("should exist");
        for connection in module_graph.get_outgoing_connections(module) {
          // TODO: consider activeState
          // if (activeState === ModuleGraphConnection.TRANSITIVE_ONLY) {
          //   add_dependencies(connection.module_identifier, set, module_graph);
          //   continue;
          // }
          set.insert(connection.module_identifier);
        }
      }

      add_dependencies(module, &mut set, module_graph);
      set.into_iter().collect()
    });

    modules.sort();

    modules
  }

  /// Notice, you should only call this function with a ModuleIdentifier that's imported dynamically or
  /// is entry module.
  pub fn get_block_chunk_group<'a>(
    &self,
    block: &ModuleIdentifier,
    chunk_group_by_ukey: &'a ChunkGroupByUkey,
  ) -> &'a ChunkGroup {
    let ukey = self
      .block_to_chunk_group_ukey
      .get(block)
      .unwrap_or_else(|| panic!("Block({block:?}) doesn't have corresponding ChunkGroup"));
    chunk_group_by_ukey
      .get(ukey)
      .unwrap_or_else(|| panic!("ChunkGroup({ukey:?}) doesn't exist"))
  }

  pub fn connect_block_and_chunk_group(
    &mut self,
    block: ModuleIdentifier,
    chunk_group: ChunkGroupUkey,
  ) {
    self.block_to_chunk_group_ukey.insert(block, chunk_group);
  }

  pub fn disconnect_chunk(
    &mut self,
    chunk: &mut Chunk,
    chunk_group_by_ukey: &mut ChunkGroupByUkey,
  ) {
    let chunk_ukey = &chunk.ukey;
    let cgc = self.get_chunk_graph_chunk_mut(*chunk_ukey);
    let cgc_modules = std::mem::take(&mut cgc.modules);
    for module in cgc_modules {
      let cgm = self.get_chunk_graph_module_mut(module);
      cgm.chunks.remove(chunk_ukey);
    }
    chunk.disconnect_from_groups(chunk_group_by_ukey)
  }
}

#[derive(Debug, Clone, Default)]
pub struct ChunkGraphModule {
  pub id: Option<String>,
  pub(crate) entry_in_chunks: HashSet<ChunkUkey>,
  pub chunks: HashSet<ChunkUkey>,
  pub(crate) runtime_requirements: Option<RuntimeSpecMap<HashSet<&'static str>>>,
  pub(crate) runtime_in_chunks: HashSet<ChunkUkey>,
  // pub(crate) hashes: Option<RuntimeSpecMap<u64>>,
}

impl ChunkGraphModule {
  pub fn new() -> Self {
    Self {
      id: None,
      entry_in_chunks: Default::default(),
      chunks: Default::default(),
      runtime_requirements: None,
      runtime_in_chunks: Default::default(),
      // hashes: None,
    }
  }
}

#[derive(Debug, Clone, Default)]
pub struct ChunkGraphChunk {
  /// URI of modules => ChunkGroupUkey
  ///
  /// use `LinkedHashMap` to keep the ordered from entry array.
  pub(crate) entry_modules: IdentifierLinkedMap<ChunkGroupUkey>,
  pub modules: IdentifierSet,
  pub(crate) runtime_requirements: HashSet<&'static str>,
  pub(crate) runtime_modules: Vec<ModuleIdentifier>,
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
