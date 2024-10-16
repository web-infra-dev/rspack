//!  There are methods whose verb is `ChunkGraphModule`

use std::hash::{Hash, Hasher};

use rspack_collections::{IdentifierSet, UkeySet};
use rspack_hash::RspackHashDigest;
use rspack_util::ext::DynHash;
use rustc_hash::FxHasher;
use tracing::instrument;

use crate::{
  get_chunk_group_from_ukey, AsyncDependenciesBlockIdentifier, ChunkByUkey, ChunkGroup,
  ChunkGroupByUkey, ChunkGroupUkey, ChunkUkey, Compilation, ModuleGraph, ModuleIdentifier,
  RuntimeGlobals, RuntimeSpec, RuntimeSpecMap, RuntimeSpecSet,
};
use crate::{ChunkGraph, Module};

#[derive(Debug, Clone, Default)]
pub struct ChunkGraphModule {
  pub id: Option<String>,
  pub(crate) entry_in_chunks: UkeySet<ChunkUkey>,
  pub chunks: UkeySet<ChunkUkey>,
  pub(crate) runtime_in_chunks: UkeySet<ChunkUkey>,
}

impl ChunkGraphModule {
  pub fn new() -> Self {
    Self {
      id: None,
      entry_in_chunks: Default::default(),
      chunks: Default::default(),
      runtime_in_chunks: Default::default(),
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

  pub fn get_module_chunks(&self, module_identifier: ModuleIdentifier) -> &UkeySet<ChunkUkey> {
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

  pub fn set_module_runtime_requirements(
    compilation: &mut Compilation,
    module_identifier: ModuleIdentifier,
    map: RuntimeSpecMap<RuntimeGlobals>,
  ) {
    compilation
      .cgm_runtime_requirements_results
      .set_runtime_requirements(module_identifier, map);
  }

  pub fn get_module_runtime_requirements<'c>(
    compilation: &'c Compilation,
    module_identifier: ModuleIdentifier,
    runtime: &RuntimeSpec,
  ) -> Option<&'c RuntimeGlobals> {
    compilation
      .cgm_runtime_requirements_results
      .get(&module_identifier, runtime)
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

  pub fn get_module_id(&self, module_identifier: ModuleIdentifier) -> Option<&str> {
    let cgm = self.get_chunk_graph_module(module_identifier);
    cgm.id.as_deref()
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

  pub fn get_module_hash<'c>(
    compilation: &'c Compilation,
    module_identifier: ModuleIdentifier,
    runtime: &RuntimeSpec,
  ) -> Option<&'c RspackHashDigest> {
    compilation
      .cgm_hash_results
      .get(&module_identifier, runtime)
  }

  pub fn set_module_hashes(
    compilation: &mut Compilation,
    module_identifier: ModuleIdentifier,
    hashes: RuntimeSpecMap<RspackHashDigest>,
  ) {
    compilation
      .cgm_hash_results
      .set_hashes(module_identifier, hashes);
  }

  #[instrument("chunk_graph:get_module_graph_hash", skip_all, fields(module = ?module.identifier()))]
  pub fn get_module_graph_hash(
    &self,
    module: &dyn Module,
    compilation: &Compilation,
    runtime: Option<&RuntimeSpec>,
  ) -> u64 {
    let mut hasher = FxHasher::default();
    self
      .get_module_graph_hash_without_connections(module, compilation, runtime)
      .hash(&mut hasher);
    let strict = module.get_strict_harmony_module();
    let mg = compilation.get_module_graph();
    let connections = mg
      .get_outgoing_connections(&module.identifier())
      .into_iter()
      .collect::<Vec<_>>();
    if !connections.is_empty() {
      let mut visited_modules = IdentifierSet::default();
      visited_modules.insert(module.identifier());
      for connection in connections {
        let module_identifier = connection.module_identifier();
        if visited_modules.contains(module_identifier) {
          continue;
        }
        if connection.active_state(&mg, runtime).is_false() {
          continue;
        }
        visited_modules.insert(*module_identifier);
        let module = mg
          .module_by_identifier(module_identifier)
          .expect("should have module")
          .as_ref();
        module.get_exports_type(&mg, strict).hash(&mut hasher);
        self.get_module_graph_hash_without_connections(module, compilation, runtime);
      }
    }
    hasher.finish()
  }

  fn get_module_graph_hash_without_connections(
    &self,
    module: &dyn Module,
    compilation: &Compilation,
    runtime: Option<&RuntimeSpec>,
  ) -> u64 {
    let mut hasher = FxHasher::default();
    let mg = compilation.get_module_graph();
    let module_identifier = module.identifier();
    let cgm = self.get_chunk_graph_module(module_identifier);
    cgm.id.as_ref().dyn_hash(&mut hasher);
    module.source_types().dyn_hash(&mut hasher);
    ModuleGraph::is_async(compilation, &module_identifier).dyn_hash(&mut hasher);
    mg.get_exports_info(&module_identifier)
      .update_hash(&mg, &mut hasher, compilation, runtime);
    hasher.finish()
  }
}
