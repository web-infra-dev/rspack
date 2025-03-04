//!  There are methods whose verb is `ChunkGraphModule`

use std::borrow::Borrow;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::sync::Arc;

use rspack_collections::{IdentifierSet, UkeySet};
use rspack_hash::RspackHashDigest;
use rspack_macros::cacheable;
use rspack_util::ext::DynHash;
use rustc_hash::FxHasher;
use serde::{Serialize, Serializer};
use tracing::instrument;

use crate::{
  for_each_runtime, AsyncDependenciesBlockIdentifier, ChunkByUkey, ChunkGroup, ChunkGroupByUkey,
  ChunkGroupUkey, ChunkUkey, Compilation, ModuleGraph, ModuleIdentifier, ModuleIdsArtifact,
  RuntimeGlobals, RuntimeSpec, RuntimeSpecMap, RuntimeSpecSet,
};
use crate::{ChunkGraph, Module};

#[cacheable]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ModuleId {
  inner: Arc<str>,
}

impl From<String> for ModuleId {
  fn from(s: String) -> Self {
    Self { inner: s.into() }
  }
}

impl From<&str> for ModuleId {
  fn from(s: &str) -> Self {
    Self { inner: s.into() }
  }
}

impl From<u32> for ModuleId {
  fn from(n: u32) -> Self {
    Self::from(n.to_string())
  }
}

impl fmt::Display for ModuleId {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}", self.as_str())
  }
}

impl Serialize for ModuleId {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    if let Some(n) = self.as_number() {
      serializer.serialize_u32(n)
    } else {
      serializer.serialize_str(self.as_str())
    }
  }
}

impl Borrow<str> for ModuleId {
  fn borrow(&self) -> &str {
    self.as_str()
  }
}

impl ModuleId {
  pub fn as_number(&self) -> Option<u32> {
    self.inner.parse::<u32>().ok()
  }

  pub fn as_str(&self) -> &str {
    &self.inner
  }
}

#[derive(Debug, Clone, Default)]
pub struct ChunkGraphModule {
  pub(super) entry_in_chunks: UkeySet<ChunkUkey>,
  pub chunks: UkeySet<ChunkUkey>,
  pub(super) runtime_in_chunks: UkeySet<ChunkUkey>,
}

impl ChunkGraphModule {
  pub fn new() -> Self {
    Self {
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

  pub fn remove_module(&mut self, module_identifier: ModuleIdentifier) {
    let Some(cgm) = self
      .chunk_graph_module_by_module_identifier
      .remove(&module_identifier)
    else {
      // already removed
      return;
    };

    for chunk in cgm.chunks {
      let chunk_graph_chunk = self.expect_chunk_graph_chunk_mut(chunk);
      chunk_graph_chunk.modules.remove(&module_identifier);
      chunk_graph_chunk.entry_modules.remove(&module_identifier);
    }

    self
      .block_to_chunk_group_ukey
      .remove(&module_identifier.into());
  }

  pub fn is_module_in_chunk(
    &self,
    module_identifier: &ModuleIdentifier,
    chunk_ukey: ChunkUkey,
  ) -> bool {
    let chunk_graph_chunk = self.expect_chunk_graph_chunk(&chunk_ukey);
    chunk_graph_chunk.modules.contains(module_identifier)
  }

  pub(crate) fn expect_chunk_graph_module_mut(
    &mut self,
    module_identifier: ModuleIdentifier,
  ) -> &mut ChunkGraphModule {
    self
      .chunk_graph_module_by_module_identifier
      .get_mut(&module_identifier)
      .unwrap_or_else(|| panic!("Module({module_identifier}) should be added before using"))
  }

  pub(crate) fn expect_chunk_graph_module(
    &self,
    module_identifier: ModuleIdentifier,
  ) -> &ChunkGraphModule {
    self
      .chunk_graph_module_by_module_identifier
      .get(&module_identifier)
      .unwrap_or_else(|| panic!("Module({module_identifier}) should be added before using"))
  }

  pub(crate) fn get_chunk_graph_module_mut(
    &mut self,
    module_identifier: ModuleIdentifier,
  ) -> Option<&mut ChunkGraphModule> {
    self
      .chunk_graph_module_by_module_identifier
      .get_mut(&module_identifier)
  }

  pub fn get_module_chunks(&self, module_identifier: ModuleIdentifier) -> &UkeySet<ChunkUkey> {
    let chunk_graph_module = self
      .chunk_graph_module_by_module_identifier
      .get(&module_identifier)
      .unwrap_or_else(|| panic!("Module({module_identifier}) should be added before using"));
    &chunk_graph_module.chunks
  }

  pub fn get_number_of_module_chunks(&self, module_identifier: ModuleIdentifier) -> usize {
    let cgm = self.expect_chunk_graph_module(module_identifier);
    cgm.chunks.len()
  }

  pub fn set_module_runtime_requirements(
    compilation: &mut Compilation,
    module_identifier: ModuleIdentifier,
    map: RuntimeSpecMap<RuntimeGlobals>,
  ) {
    compilation
      .cgm_runtime_requirements_artifact
      .set_runtime_requirements(module_identifier, map);
  }

  pub fn get_module_runtime_requirements<'c>(
    compilation: &'c Compilation,
    module_identifier: ModuleIdentifier,
    runtime: &RuntimeSpec,
  ) -> Option<&'c RuntimeGlobals> {
    compilation
      .cgm_runtime_requirements_artifact
      .get(&module_identifier, runtime)
  }

  pub fn get_module_runtimes(
    &self,
    module_identifier: ModuleIdentifier,
    chunk_by_ukey: &ChunkByUkey,
  ) -> RuntimeSpecSet {
    let cgm = self.expect_chunk_graph_module(module_identifier);
    let mut runtimes = RuntimeSpecSet::default();
    for chunk_ukey in cgm.chunks.iter() {
      let chunk = chunk_by_ukey.expect_get(chunk_ukey);
      runtimes.set(chunk.runtime().clone());
    }
    runtimes
  }

  pub fn get_module_runtimes_iter<'a>(
    &self,
    module_identifier: ModuleIdentifier,
    chunk_by_ukey: &'a ChunkByUkey,
  ) -> impl Iterator<Item = &'a RuntimeSpec> + use<'a, '_> {
    let cgm = self.expect_chunk_graph_module(module_identifier);
    cgm.chunks.iter().map(|chunk_ukey| {
      let chunk = chunk_by_ukey.expect_get(chunk_ukey);
      chunk.runtime()
    })
  }

  pub fn get_module_id(
    module_ids: &ModuleIdsArtifact,
    module_identifier: ModuleIdentifier,
  ) -> Option<&ModuleId> {
    module_ids.get(&module_identifier)
  }

  pub fn set_module_id(
    module_ids: &mut ModuleIdsArtifact,
    module_identifier: ModuleIdentifier,
    id: ModuleId,
  ) -> bool {
    if let Some(old_id) = module_ids.insert(module_identifier, id.clone()) {
      old_id != id
    } else {
      true
    }
  }

  pub fn get_block_chunk_group<'a>(
    &self,
    block: &AsyncDependenciesBlockIdentifier,
    chunk_group_by_ukey: &'a ChunkGroupByUkey,
  ) -> Option<&'a ChunkGroup> {
    self
      .block_to_chunk_group_ukey
      .get(block)
      .and_then(|ukey| chunk_group_by_ukey.get(ukey))
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
      .cgm_hash_artifact
      .get(&module_identifier, runtime)
  }

  pub fn set_module_hashes(
    compilation: &mut Compilation,
    module_identifier: ModuleIdentifier,
    hashes: RuntimeSpecMap<RspackHashDigest>,
  ) -> bool {
    compilation
      .cgm_hash_artifact
      .set_hashes(module_identifier, hashes)
  }

  pub fn try_get_module_chunks(
    &self,
    module_identifier: &ModuleIdentifier,
  ) -> Option<&UkeySet<ChunkUkey>> {
    self
      .chunk_graph_module_by_module_identifier
      .get(module_identifier)
      .map(|cgm| &cgm.chunks)
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
    let strict = module.get_strict_esm_module();
    let mg = compilation.get_module_graph();
    let mut visited_modules = IdentifierSet::default();
    visited_modules.insert(module.identifier());
    for connection in mg
      .get_outgoing_connections_in_order(&module.identifier())
      .filter_map(|c| mg.connection_by_dependency_id(c))
    {
      let module_identifier = connection.module_identifier();
      if visited_modules.contains(module_identifier) {
        continue;
      }
      let active_state = connection.active_state(&mg, runtime);
      if active_state.is_false() {
        continue;
      }
      visited_modules.insert(*module_identifier);
      for_each_runtime(
        runtime,
        |runtime| {
          let runtime = runtime.map(|r| RuntimeSpec::from_iter([r.as_str().into()]));
          let active_state = connection.active_state(&mg, runtime.as_ref());
          active_state.hash(&mut hasher);
        },
        true,
      );
      let module = mg
        .module_by_identifier(module_identifier)
        .expect("should have module")
        .as_ref();
      module.get_exports_type(&mg, strict).hash(&mut hasher);
      self
        .get_module_graph_hash_without_connections(module, compilation, runtime)
        .hash(&mut hasher);
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
    Self::get_module_id(&compilation.module_ids_artifact, module_identifier).dyn_hash(&mut hasher);
    module.source_types().dyn_hash(&mut hasher);
    ModuleGraph::is_async(compilation, &module_identifier).dyn_hash(&mut hasher);
    mg.get_exports_info(&module_identifier)
      .update_hash(&mg, &mut hasher, compilation, runtime);
    hasher.finish()
  }
}
