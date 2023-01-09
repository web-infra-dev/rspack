// Port of https://github.com/webpack/webpack/blob/4b4ca3bb53f36a5b8fc6bc1bd976ed7af161bd80/lib/optimize/SplitChunksPlugin.js#L78

use derivative::Derivative;

use crate::{cache_group_source::SplitChunkSizes, ChunkFilter, GetName};

#[derive(Derivative)]
#[derivative(Debug)]
pub struct CacheGroup {
  pub key: String,
  pub priority: isize,
  #[derivative(Debug = "ignore")]
  pub get_name: GetName,
  #[derivative(Debug = "ignore")]
  pub chunks_filter: ChunkFilter,
  pub min_chunks: usize,
  pub max_async_requests: usize,
  pub max_initial_requests: usize,
  pub filename: Option<String>,
  pub id_hint: String,
  pub automatic_name_delimiter: String,
  pub reuse_existing_chunk: bool,
  // TODO: supports used_exports
  // pub used_exports: bool,
  pub min_size: SplitChunkSizes,
  pub min_size_reduction: SplitChunkSizes,
  pub min_remaining_size: SplitChunkSizes,
  pub enforce_size_threshold: SplitChunkSizes,
  pub max_async_size: SplitChunkSizes,
  pub max_initial_size: SplitChunkSizes,
  pub(crate) validate_size: bool,
  pub(crate) min_size_for_max_size: SplitChunkSizes,
}
