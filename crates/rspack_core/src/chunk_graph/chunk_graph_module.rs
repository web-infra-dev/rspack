//!  There are methods whose verb is `ChunkGraphModule`
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::Hasher;

use rspack_identifier::Identifier;
use rspack_util::ext::DynHash;
use rustc_hash::FxHashSet as HashSet;

use crate::update_hash::{UpdateHashContext, UpdateRspackHash};
use crate::ChunkGraph;
use crate::{
  get_chunk_group_from_ukey, AsyncDependenciesBlockIdentifier, BoxModule, ChunkByUkey, ChunkGroup,
  ChunkGroupByUkey, ChunkGroupUkey, ChunkUkey, Compilation, ExportsHash, ModuleIdentifier,
  RuntimeGlobals, RuntimeSpec, RuntimeSpecMap, RuntimeSpecSet,
};

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
      .or_default();
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
      let chunk = chunk_by_ukey.expect_get(chunk_ukey);
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

  pub fn get_block_chunk_group<'a>(
    &self,
    block: &AsyncDependenciesBlockIdentifier,
    chunk_group_by_ukey: &'a ChunkGroupByUkey,
  ) -> Option<&'a ChunkGroup> {
    self
      .block_to_chunk_group_ukey
      .get(block)
      .and_then(|ukey| get_chunk_group_from_ukey(ukey, chunk_group_by_ukey))
  }

  pub fn connect_block_and_chunk_group(
    &mut self,
    block: AsyncDependenciesBlockIdentifier,
    chunk_group: ChunkGroupUkey,
  ) {
    self.block_to_chunk_group_ukey.insert(block, chunk_group);
  }

  pub fn get_module_graph_hash(
    &self,
    module: &BoxModule,
    compilation: &Compilation,
    runtime: Option<&RuntimeSpec>,
    with_connections: bool,
  ) -> String {
    let mut hasher = DefaultHasher::new();
    let mut connection_hash_cache: HashMap<Identifier, u64> = HashMap::new();
    let module_graph = &compilation.module_graph;

    let process_module_graph_module = |module: &BoxModule, strict: Option<bool>| -> u64 {
      let mut hasher = DefaultHasher::new();
      module.identifier().dyn_hash(&mut hasher);
      module.source_types().dyn_hash(&mut hasher);
      module_graph
        .is_async(&module.identifier())
        .dyn_hash(&mut hasher);

      module_graph
        .get_exports_info(&module.identifier())
        .export_info_hash(&mut hasher, module_graph, &mut HashSet::default());

      module
        .get_blocks()
        .iter()
        .filter_map(|id| module_graph.block_by_id(id))
        .for_each(|block| {
          block.update_hash(
            &mut hasher,
            &UpdateHashContext {
              compilation,
              runtime,
            },
          )
        });

      // NOTE:
      // Webpack use module.getExportsType() to generate hash
      // but the module graph may be modified in it
      // and exports type is calculated from build meta and exports info
      // so use them to generate hash directly to avoid mutable access to module graph
      if let Some(strict) = strict {
        if let Some(build_meta) = module.build_meta() {
          strict.dyn_hash(&mut hasher);
          build_meta.default_object.dyn_hash(&mut hasher);
          build_meta.exports_type.dyn_hash(&mut hasher);
        }
      }

      hasher.finish()
    };

    // hash module build_info
    module_graph
      .get_module_hash(&module.identifier())
      .dyn_hash(&mut hasher);
    // hash module graph module
    process_module_graph_module(module, None).dyn_hash(&mut hasher);

    let strict: bool = module_graph
      .module_by_identifier(&module.identifier())
      .unwrap_or_else(|| {
        panic!(
          "Module({}) should be added before using",
          module.identifier()
        )
      })
      .get_strict_harmony_module();

    if with_connections {
      let mut connections = module_graph
        .get_outgoing_connections(module)
        .into_iter()
        .collect::<Vec<_>>();

      connections.sort_by(|a, b| a.module_identifier.cmp(&b.module_identifier));

      // hash connection module graph modules
      for connection in connections {
        if let Some(connection_hash) = connection_hash_cache.get(&connection.module_identifier) {
          connection_hash.dyn_hash(&mut hasher)
        } else {
          let connection_hash = process_module_graph_module(
            module_graph
              .module_by_identifier(&connection.module_identifier)
              .unwrap_or_else(|| {
                panic!(
                  "Module({}) should be added before using",
                  connection.module_identifier
                )
              }),
            Some(strict),
          );
          connection_hash.dyn_hash(&mut hasher);
          connection_hash_cache.insert(connection.module_identifier, connection_hash);
        }
      }
    }

    format!("{:016x}", hasher.finish())
  }
}
