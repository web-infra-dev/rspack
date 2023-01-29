use derivative::Derivative;
use rspack_core::ModuleType;

use crate::{ChunkType, OptimizationSplitChunksSizes, TestFn};

#[derive(Derivative)]
#[derivative(Debug)]
#[derive(Clone, Default)]
pub struct CacheGroupOptions {
  pub automatic_name_delimiter: Option<String>,
  /// What kind of chunks should be selected.
  pub chunks: Option<ChunkType>,
  pub enforce: Option<bool>,
  pub enforce_size_threshold: Option<OptimizationSplitChunksSizes>,
  pub filename: Option<String>,
  pub id_hint: Option<String>,
  // TODO: supports pub fn layer: RegExp | string | Function;
  pub max_async_requests: Option<u32>,
  pub max_async_size: Option<f64>,
  pub max_initial_requests: Option<u32>,
  pub max_initial_size: Option<f64>,
  pub max_size: Option<f64>,
  pub min_chunks: Option<u32>,
  pub min_remaining_size: Option<f64>,
  pub min_size: Option<f64>,
  pub min_size_reduction: Option<f64>,
  pub name: Option<String>,
  pub priority: Option<i32>,
  pub reuse_existing_chunk: Option<bool>,
  #[derivative(Debug = "ignore")]
  pub test: Option<TestFn>,
  pub r#type: Option<ModuleType>,
  // TODO: supports: used_exports: Option<bool>,
}
