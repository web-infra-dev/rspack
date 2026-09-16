use rspack_collections::IdentifierMap;
use rustc_hash::FxHashMap as HashMap;

use crate::{AsyncDependenciesBlockIdentifierMap, ChunkGroupUkey, ChunkUkey, ModuleIdentifier};

mod map;
pub use map::ChunkMap;
mod set;
pub use set::ChunkSet;

mod slot_map;
pub use slot_map::ChunkSlotMap;

pub mod chunk_graph_chunk;
pub mod chunk_graph_module;
pub use chunk_graph_chunk::{ChunkGraphChunk, ChunkIdMap, ChunkSizeOptions, IndexChunkIdMap};
pub use chunk_graph_module::{ChunkGraphModule, ModuleId, ModuleIdMap};

/// Relationships kept separate from Chunk values for disjoint field borrowing.
/// Clone copies relationships as part of a graph snapshot.
#[derive(Debug, Clone, Default)]
pub struct ChunkGraphTopology {
  /// If a module is imported dynamically, it will be assigned to a unique ChunkGroup
  pub(crate) block_to_chunk_group_ukey: AsyncDependenciesBlockIdentifierMap<ChunkGroupUkey>,

  pub(crate) chunk_graph_module_by_module_identifier: IdentifierMap<ChunkGraphModule>,
  chunk_graph_chunk_by_chunk_ukey: ChunkSlotMap<ChunkGraphChunk>,

  runtime_ids: HashMap<String, Option<String>>,
}

impl ChunkGraphTopology {
  pub fn is_entry_module(&self, module_id: &ModuleIdentifier) -> bool {
    let cgm = self.expect_chunk_graph_module(*module_id);
    !cgm.entry_in_chunks.is_empty()
  }
}

/// Owns Chunk storage and relationships. Cloning copies a graph snapshot while
/// preserving existing keys; subsequent allocations receive fresh identities.
#[derive(Debug, Clone, Default)]
pub struct ChunkGraph {
  pub chunks: ChunkSlotMap<crate::Chunk>,
  pub topology: ChunkGraphTopology,
  // Allocation belongs to the graph, never to its individual value tables.
  free_slots: Vec<u32>,
}

impl std::ops::Deref for ChunkGraph {
  type Target = ChunkGraphTopology;
  fn deref(&self) -> &Self::Target {
    &self.topology
  }
}
impl std::ops::DerefMut for ChunkGraph {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.topology
  }
}

impl ChunkGraph {
  pub fn create_chunk(
    &mut self,
    name: Option<String>,
    kind: crate::ChunkKind,
  ) -> rspack_error::Result<ChunkUkey> {
    let index = match self.free_slots.last() {
      Some(index) => *index,
      None => u32::try_from(self.chunks.slot_count())
        .map_err(|_| rspack_error::error!("Chunk slot space exhausted"))?,
    };
    // Allocate before changing either table: identity exhaustion leaves the graph intact.
    let key = ChunkUkey::allocate(index)?;
    self.free_slots.pop();
    self.chunks.insert(key, crate::Chunk::new(key, name, kind));
    self
      .topology
      .chunk_graph_chunk_by_chunk_ukey
      .insert(key, ChunkGraphChunk::default());
    Ok(key)
  }

  /// Add the target to the source's groups and merge its runtime and name hints.
  /// Collect first, then apply: the source Chunk only needs an immutable borrow.
  pub fn split_chunk(
    &mut self,
    source: &ChunkUkey,
    target: &ChunkUkey,
    groups: &mut crate::ChunkGroupByUkey,
  ) {
    assert_ne!(source, target, "Cannot split a Chunk into itself");
    // Validate both handles before updating any group relationships.
    self.chunks.expect_get(target);
    let source = self.chunks.expect_get(source);
    let mut data =
      crate::ChunkSplitData::with_capacity(source.groups().len(), source.id_name_hints().len());
    source.split_collect_new_chunk_data(*target, groups, &mut data);
    data.apply_to(self.chunks.expect_get_mut(target));
  }

  pub(crate) fn remove_chunk(
    &mut self,
    key: &ChunkUkey,
    groups: &mut crate::ChunkGroupByUkey,
  ) -> Option<crate::Chunk> {
    // Both tables share membership. Removing the topology row validates the
    // complete key, including stale identities after slot reuse.
    let relations = self.topology.chunk_graph_chunk_by_chunk_ukey.remove(key)?;
    for module in relations
      .modules
      .iter()
      .chain(relations.entry_modules.keys())
      .chain(relations.runtime_modules.iter())
    {
      if let Some(data) = self
        .topology
        .chunk_graph_module_by_module_identifier
        .get_mut(module)
      {
        data.chunks.remove(key);
        data.entry_in_chunks.remove(key);
        data.runtime_in_chunks.remove(key);
      }
    }
    let mut chunk = self.chunks.remove(key).expect("live Chunk");
    chunk.disconnect_from_groups(groups);
    self.free_slots.push(key.index() as u32);
    Some(chunk)
  }
}

impl ChunkGraph {
  pub fn integrate_chunks(
    &mut self,
    a: &ChunkUkey,
    b: &ChunkUkey,
    groups: &mut crate::ChunkGroupByUkey,
    modules: &crate::ModuleGraph,
  ) {
    self
      .topology
      .integrate_chunks(a, b, &mut self.chunks, groups, modules);
  }
}
