// Port of https://github.com/webpack/webpack/blob/4b4ca3bb53f36a5b8fc6bc1bd976ed7af161bd80/lib/optimize/SplitChunksPlugin.js#L55

use derivative::Derivative;
use rspack_core::SourceType;
use rustc_hash::FxHashMap as HashMap;

use crate::{ChunkFilterFn, SplitChunksNameFn};

pub(crate) type SplitChunkSizes = HashMap<SourceType, f64>;
#[derive(Derivative)]
#[derivative(Debug)]
pub struct CacheGroupSource {
  pub key: String,
  pub priority: Option<i32>,

  #[derivative(Debug = "ignore")]
  pub get_name: Option<SplitChunksNameFn>,
  #[derivative(Debug = "ignore")]
  pub chunks_filter: Option<ChunkFilterFn>,
  pub enforce: Option<bool>,
  pub min_size: SplitChunkSizes,
  pub min_size_reduction: SplitChunkSizes,
  pub min_remaining_size: SplitChunkSizes,
  pub enforce_size_threshold: SplitChunkSizes,
  pub max_async_size: SplitChunkSizes,
  pub max_initial_size: SplitChunkSizes,
  pub min_chunks: Option<u32>,
  pub max_async_requests: Option<u32>,
  pub max_initial_requests: Option<u32>,
  pub filename: Option<String>,
  pub id_hint: Option<String>,
  pub automatic_name_delimiter: String,
  pub reuse_existing_chunk: Option<bool>,
  // TODO: supports used_exports
  // pub used_exports: bool,
}
