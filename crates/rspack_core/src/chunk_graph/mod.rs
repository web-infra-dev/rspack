use std::hash::BuildHasherDefault;

use dashmap::DashMap;
use rspack_identifier::{Identifier, IdentifierHasher, IdentifierMap};
use rustc_hash::FxHashMap as HashMap;

use crate::AsyncDependenciesBlockId;
use crate::{ChunkGroupUkey, ChunkUkey};

pub mod chunk_graph_chunk;
pub mod chunk_graph_module;
pub use chunk_graph_chunk::ChunkGraphChunk;
pub use chunk_graph_chunk::ChunkSizeOptions;
pub use chunk_graph_module::ChunkGraphModule;

#[derive(Debug, Clone, Default)]
pub struct ChunkGraph {
  pub split_point_module_identifier_to_chunk_ukey: IdentifierMap<ChunkUkey>,

  /// If a module is imported dynamically, it will be assigned to a unique ChunkGroup
  pub(crate) block_to_chunk_group_ukey: HashMap<AsyncDependenciesBlockId, ChunkGroupUkey>,

  pub chunk_graph_module_by_module_identifier:
    DashMap<Identifier, ChunkGraphModule, BuildHasherDefault<IdentifierHasher>>,
  chunk_graph_chunk_by_chunk_ukey:
    DashMap<ChunkUkey, ChunkGraphChunk, BuildHasherDefault<IdentifierHasher>>,
}
