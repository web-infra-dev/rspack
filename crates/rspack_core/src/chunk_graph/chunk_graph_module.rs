//!  There are methods whose verb is `ChunkGraphModule`

use std::io::Write;

use rspack_hash::RspackHash;
use rspack_util::ext::DynHash;
use rustc_hash::FxHashSet as HashSet;

use crate::{
  BoxModule, ChunkByUkey, ChunkGroup, ChunkGroupByUkey, ChunkGroupUkey, ChunkUkey,
  ModuleGraphConnection, ModuleIdentifier, OutputOptions, RuntimeGlobals, RuntimeSpec,
  RuntimeSpecMap, RuntimeSpecSet,
};
use crate::{ChunkGraph, ModuleGraph};

#[derive(Debug, Clone, Default)]
pub struct ChunkGraphModule {
  pub id: Option<String>,
  pub(crate) entry_in_chunks: HashSet<ChunkUkey>,
  pub chunks: HashSet<ChunkUkey>,
  pub(crate) runtime_requirements: Option<RuntimeSpecMap<RuntimeGlobals>>,
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

impl ChunkGraph {
  pub fn add_module(&mut self, module_identifier: ModuleIdentifier) {
    self
      .chunk_graph_module_by_module_identifier
      .entry(module_identifier)
      .or_insert_with(ChunkGraphModule::new);
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
      .unwrap_or_else(|| panic!("Module({}) should be added before using", module_identifier))
  }

  pub(crate) fn get_chunk_graph_module(
    &self,
    module_identifier: ModuleIdentifier,
  ) -> &ChunkGraphModule {
    self
      .chunk_graph_module_by_module_identifier
      .get(&module_identifier)
      .unwrap_or_else(|| panic!("Module({}) should be added before using", module_identifier))
  }

  pub fn get_module_chunks(&self, module_identifier: ModuleIdentifier) -> &HashSet<ChunkUkey> {
    let chunk_graph_module = self
      .chunk_graph_module_by_module_identifier
      .get(&module_identifier)
      .unwrap_or_else(|| panic!("Module({}) should be added before using", module_identifier));
    &chunk_graph_module.chunks
  }

  pub fn get_number_of_module_chunks(&self, module_identifier: ModuleIdentifier) -> usize {
    let cgm = self.get_chunk_graph_module(module_identifier);
    cgm.chunks.len()
  }

  pub fn add_module_runtime_requirements(
    &mut self,
    module_identifier: ModuleIdentifier,
    runtime: &RuntimeSpec,
    runtime_requirements: RuntimeGlobals,
  ) {
    let cgm = self.get_chunk_graph_module_mut(module_identifier);

    if let Some(runtime_requirements_map) = &mut cgm.runtime_requirements {
      if let Some(value) = runtime_requirements_map.get_mut(runtime) {
        value.insert(runtime_requirements);
      } else {
        runtime_requirements_map.set(runtime.clone(), runtime_requirements);
      }
    } else {
      let mut runtime_requirements_map = RuntimeSpecMap::default();
      runtime_requirements_map.set(runtime.clone(), runtime_requirements);
      cgm.runtime_requirements = Some(runtime_requirements_map);
    }
  }

  pub fn get_module_runtime_requirements(
    &self,
    module_identifier: ModuleIdentifier,
    runtime: &RuntimeSpec,
  ) -> Option<&RuntimeGlobals> {
    let cgm = self.get_chunk_graph_module(module_identifier);
    if let Some(runtime_requirements) = &cgm.runtime_requirements {
      if let Some(runtime_requirements) = runtime_requirements.get(runtime) {
        return Some(runtime_requirements);
      }
    }
    None
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

  pub fn get_module_id(&self, module_identifier: ModuleIdentifier) -> &Option<String> {
    let cgm = self.get_chunk_graph_module(module_identifier);
    &cgm.id
  }

  pub fn set_module_id(&mut self, module_identifier: ModuleIdentifier, id: String) {
    let cgm = self.get_chunk_graph_module_mut(module_identifier);
    cgm.id = Some(id);
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

  pub fn get_module_graph_hash(
    &self,
    hash_options: &OutputOptions,
    module: &BoxModule,
    module_graph: &ModuleGraph,
    with_connections: bool,
  ) -> String {
    let mut hasher = RspackHash::new(&hash_options.hash_function);

    fn process_module_graph_module(
      hasher: &mut RspackHash,
      module: &BoxModule,
      module_graph: &ModuleGraph,
    ) {
      module.identifier().dyn_hash(hasher);
      module.source_types().dyn_hash(hasher);
      module_graph.is_async(&module.identifier()).dyn_hash(hasher);
      module_graph
        .get_exports_info(&module.identifier())
        .dyn_hash(hasher);
    }

    // hash module build_info
    module_graph
      .get_module_hash(&module.identifier())
      .dyn_hash(&mut hasher);
    // hash module graph module
    process_module_graph_module(&mut hasher, module, module_graph);

    if with_connections {
      let strict: bool = module_graph
        .module_graph_module_by_identifier(&module.identifier())
        .unwrap_or_else(|| {
          panic!(
            "Module({}) should be added before using",
            module.identifier()
          )
        })
        .get_strict_harmony_module();
      let mut connections = module_graph
        .get_outgoing_connections(module)
        .into_iter()
        .collect::<Vec<_>>();

      connections.sort_by(|a, b| a.module_identifier.cmp(&b.module_identifier));

      // hash connection module graph modules
      for connection in connections {
        module_graph
          .module_graph_module_by_identifier(&connection.module_identifier)
          .unwrap_or_else(|| {
            panic!(
              "Module({}) should be added before using",
              connection.module_identifier
            )
          })
          .get_exports_type(strict)
          .dyn_hash(&mut hasher);
        process_module_graph_module(
          &mut hasher,
          module_graph
            .module_by_identifier(&connection.module_identifier)
            .unwrap_or_else(|| {
              panic!(
                "Module({}) should be added before using",
                connection.module_identifier
              )
            }),
          module_graph,
        );
      }
    }

    hasher
      .digest(&hash_options.hash_digest)
      .rendered(hash_options.hash_digest_length)
      .to_owned()
  }
}
