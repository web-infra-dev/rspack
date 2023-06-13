//!  There are methods whose verb is `ChunkGraphChunk`

use rspack_identifier::{IdentifierLinkedMap, IdentifierSet};
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};

use crate::ChunkGraph;
use crate::{
  find_graph_roots, BoxModule, Chunk, ChunkByUkey, ChunkGroupByUkey, ChunkGroupUkey, ChunkUkey,
  Module, ModuleGraph, ModuleGraphModule, ModuleIdentifier, RuntimeGlobals, SourceType,
};

#[derive(Debug, Clone, Default)]
pub struct ChunkGraphChunk {
  /// URI of modules => ChunkGroupUkey
  ///
  /// use `LinkedHashMap` to keep the ordered from entry array.
  pub(crate) entry_modules: IdentifierLinkedMap<ChunkGroupUkey>,
  pub modules: IdentifierSet,
  pub(crate) runtime_requirements: RuntimeGlobals,
  pub(crate) runtime_modules: Vec<ModuleIdentifier>,
}

impl ChunkGraphChunk {
  pub fn new() -> Self {
    Self {
      entry_modules: Default::default(),
      modules: Default::default(),
      runtime_requirements: Default::default(),
      runtime_modules: Default::default(),
    }
  }
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

  pub fn get_chunk_entry_modules(&self, chunk_ukey: &ChunkUkey) -> Vec<ModuleIdentifier> {
    let chunk_graph_chunk = self.get_chunk_graph_chunk(chunk_ukey);

    chunk_graph_chunk.entry_modules.keys().cloned().collect()
  }

  pub fn get_chunk_entry_modules_with_chunk_group_iterable(
    &self,
    chunk_ukey: &ChunkUkey,
  ) -> &IdentifierLinkedMap<ChunkGroupUkey> {
    let cgc = self.get_chunk_graph_chunk(chunk_ukey);
    &cgc.entry_modules
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

  pub fn get_chunk_modules<'module>(
    &self,
    chunk: &ChunkUkey,
    module_graph: &'module ModuleGraph,
  ) -> Vec<&'module BoxModule> {
    let chunk_graph_chunk = self.get_chunk_graph_chunk(chunk);
    chunk_graph_chunk
      .modules
      .iter()
      .filter_map(|uri| module_graph.module_by_identifier(uri))
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
  ) -> Vec<&'module BoxModule> {
    let mut modules = self.get_chunk_modules(chunk, module_graph);
    // SAFETY: module identifier is unique
    modules.sort_unstable_by_key(|m| m.identifier().as_str());
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

  pub fn get_chunk_modules_iterable_by_source_type<'module_graph: 'me, 'me>(
    &'me self,
    chunk: &ChunkUkey,
    source_type: SourceType,
    module_graph: &'module_graph ModuleGraph,
  ) -> impl Iterator<Item = &'module_graph dyn Module> + 'me {
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
        acc + m.source_types().iter().fold(0.0, |acc, t| acc + m.size(t))
      })
  }

  pub fn get_number_of_chunk_modules(&self, chunk: &ChunkUkey) -> usize {
    let cgc = self.get_chunk_graph_chunk(chunk);
    cgc.modules.len()
  }

  pub fn get_number_of_entry_modules(&self, chunk: &ChunkUkey) -> usize {
    let cgc = self.get_chunk_graph_chunk(chunk);
    cgc.entry_modules.len()
  }

  pub fn add_chunk_runtime_requirements(
    &mut self,
    chunk_ukey: &ChunkUkey,
    runtime_requirements: RuntimeGlobals,
  ) {
    let cgc = self.get_chunk_graph_chunk_mut(*chunk_ukey);
    cgc.runtime_requirements.add(runtime_requirements);
  }

  pub fn add_tree_runtime_requirements(
    &mut self,
    chunk_ukey: &ChunkUkey,
    runtime_requirements: RuntimeGlobals,
  ) {
    self.add_chunk_runtime_requirements(chunk_ukey, runtime_requirements);
  }

  pub fn get_chunk_runtime_requirements(&self, chunk_ukey: &ChunkUkey) -> &RuntimeGlobals {
    let cgc = self.get_chunk_graph_chunk(chunk_ukey);
    &cgc.runtime_requirements
  }

  pub fn get_tree_runtime_requirements(&self, chunk_ukey: &ChunkUkey) -> &RuntimeGlobals {
    self.get_chunk_runtime_requirements(chunk_ukey)
  }

  pub fn get_chunk_runtime_modules_in_order(
    &self,
    chunk_ukey: &ChunkUkey,
  ) -> &Vec<ModuleIdentifier> {
    let cgc = self.get_chunk_graph_chunk(chunk_ukey);
    &cgc.runtime_modules
  }

  pub fn get_chunk_runtime_modules_iterable(
    &self,
    chunk_ukey: &ChunkUkey,
  ) -> impl Iterator<Item = &ModuleIdentifier> {
    let cgc = self.get_chunk_graph_chunk(chunk_ukey);
    cgc.runtime_modules.iter()
  }

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

  pub fn get_chunk_root_modules(
    &self,
    chunk: &ChunkUkey,
    module_graph: &ModuleGraph,
  ) -> Vec<ModuleIdentifier> {
    let cgc = self.get_chunk_graph_chunk(chunk);
    let mut input = cgc.modules.iter().cloned().collect::<Vec<_>>();
    input.sort_unstable();
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

    modules.sort_unstable();

    modules
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

  pub fn has_chunk_entry_dependent_chunks(
    &self,
    chunk_ukey: &ChunkUkey,
    chunk_group_by_ukey: &ChunkGroupByUkey,
  ) -> bool {
    let cgc = self.get_chunk_graph_chunk(chunk_ukey);
    for (_, chunk_group_ukey) in cgc.entry_modules.iter() {
      let chunk_group = chunk_group_by_ukey
        .get(chunk_group_ukey)
        .expect("should have chunk group");
      for c in chunk_group.chunks.iter() {
        if c != chunk_ukey {
          return true;
        }
      }
    }
    false
  }

  pub fn get_chunk_entry_dependent_chunks_iterable(
    &self,
    chunk_ukey: &ChunkUkey,
    chunk_by_ukey: &ChunkByUkey,
    chunk_group_by_ukey: &ChunkGroupByUkey,
  ) -> impl Iterator<Item = ChunkUkey> {
    let chunk = chunk_by_ukey.get(chunk_ukey).expect("should have chunk");
    let mut set = HashSet::default();
    for chunk_group_ukey in chunk.groups.iter() {
      let chunk_group = chunk_group_by_ukey
        .get(chunk_group_ukey)
        .expect("should have chunk group");
      if chunk_group.is_initial() {
        let entry_point_chunk = chunk_group.get_entry_point_chunk();
        let cgc = self.get_chunk_graph_chunk(&entry_point_chunk);
        for (_, chunk_group_ukey) in cgc.entry_modules.iter() {
          let chunk_group = chunk_group_by_ukey
            .get(chunk_group_ukey)
            .expect("should have chunk group");
          for c in chunk_group.chunks.iter() {
            let chunk = chunk_by_ukey.get(c).expect("should have chunk");
            if c != chunk_ukey && c != &entry_point_chunk && !chunk.has_runtime(chunk_group_by_ukey)
            {
              set.insert(*c);
            }
          }
        }
      }
    }
    set.into_iter()
  }
}
