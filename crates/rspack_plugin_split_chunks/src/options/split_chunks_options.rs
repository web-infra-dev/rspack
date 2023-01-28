use std::collections::HashMap;

use derivative::Derivative;

use crate::{
  CacheGroupOptions, ChunkType, OptimizationSplitChunksSizes, SizeType, SplitChunksNameFn,
};

/// Align with https://github.com/webpack/webpack/blob/4b4ca3bb53f36a5b8fc6bc1bd976ed7af161bd80/types.d.ts#L8265
#[derive(Default, Derivative)]
#[derivative(Debug, Clone)]
pub struct SplitChunksOptions {
  pub automatic_name_delimiter: Option<String>,
  pub cache_groups: HashMap<String, CacheGroupOptions>,
  /// What kind of chunks should be selected.
  pub chunks: Option<ChunkType>,
  pub default_size_types: Option<Vec<SizeType>>,
  pub enforce_size_threshold: Option<OptimizationSplitChunksSizes>,
  pub fallback_cache_group: Option<SplitChunksOptionsCacheGroup>,
  pub filename: Option<String>,
  // TODO: Supports pub hide_path_info: Option<bool>,
  pub max_async_requests: Option<u32>,
  pub max_async_size: Option<f64>,
  pub max_initial_requests: Option<u32>,
  pub max_initial_size: Option<f64>,
  pub max_size: Option<f64>,
  pub min_chunks: Option<u32>,
  pub min_remaining_size: Option<f64>,
  pub min_size: Option<f64>,
  pub min_size_reduction: Option<f64>,
  #[derivative(Debug = "ignore")]
  pub name: Option<SplitChunksNameFn>,
  // TODO: Supports used_exports: bool,
}

#[derive(Debug, Default, Clone)]
pub struct SplitChunksOptionsCacheGroup {
  pub automatic_name_delimiter: Option<String>,
  pub chunks: Option<ChunkType>,
  pub max_async_size: Option<OptimizationSplitChunksSizes>,
  pub max_initial_size: Option<OptimizationSplitChunksSizes>,
  pub max_size: Option<OptimizationSplitChunksSizes>,
  pub min_size: Option<OptimizationSplitChunksSizes>,
  pub min_size_reduction: Option<OptimizationSplitChunksSizes>,
}
