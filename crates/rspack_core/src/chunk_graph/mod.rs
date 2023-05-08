use rspack_identifier::IdentifierMap;
use rustc_hash::FxHashMap as HashMap;

use crate::{ChunkGroupUkey, ChunkUkey};

pub mod chunk_graph_chunk;
pub mod chunk_graph_module;
pub use chunk_graph_chunk::ChunkGraphChunk;
pub use chunk_graph_module::ChunkGraphModule;

#[derive(Debug, Clone, Default)]
pub struct ChunkGraph {
  pub split_point_module_identifier_to_chunk_ukey: IdentifierMap<ChunkUkey>,

  /// If a module is imported dynamically, it will be assigned to a unique ChunkGroup
  pub(crate) block_to_chunk_group_ukey: IdentifierMap<ChunkGroupUkey>,

  pub chunk_graph_module_by_module_identifier: IdentifierMap<ChunkGraphModule>,
  chunk_graph_chunk_by_chunk_ukey: HashMap<ChunkUkey, ChunkGraphChunk>,
}
