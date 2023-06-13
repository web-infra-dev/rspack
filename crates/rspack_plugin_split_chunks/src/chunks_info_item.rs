use derivative::Derivative;
use rspack_core::ChunkUkey;
use rspack_identifier::IdentifierSet;
use rustc_hash::FxHashSet;

use crate::{cache_group::CacheGroup, cache_group_source::SplitChunkSizes, CacheGroupByKey};

#[derive(Derivative)]
#[derivative(Debug)]
pub(crate) struct ChunksInfoItem {
  // Sortable Module Set
  #[derivative(Debug = "ignore")]
  pub modules: IdentifierSet,
  pub cache_group: String,
  pub cache_group_index: usize,
  pub name: Option<String>,
  pub sizes: SplitChunkSizes,
  pub chunks: FxHashSet<ChunkUkey>,
  pub _reusable_chunks: FxHashSet<ChunkUkey>,
  // bigint | Chunk
  // pub chunks_keys: Hash
}

impl ChunksInfoItem {
  pub(crate) fn cache_group<'cache_group>(
    &self,
    map: &'cache_group CacheGroupByKey,
  ) -> &'cache_group CacheGroup {
    map
      .get(&self.cache_group)
      .unwrap_or_else(|| panic!("Cache group not found: {}", self.cache_group))
  }
}
