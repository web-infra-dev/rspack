use std::fmt::Debug;

use derivative::Derivative;
use hashbrown::HashMap;
use rspack_core::{Chunk, ChunkGroupByUkey, ModuleType, SourceType};

use crate::{cache_group_source::SplitChunkSizes, ChunkFilter, GetName, TestFn};

#[derive(Clone, Copy)]
pub enum ChunkType {
  Initial,
  Async,
  All,
  // Custom(Box<dyn Fn(&Chunk) -> bool + Sync + Send>),
}

impl TryFrom<&str> for ChunkType {
  type Error = String;

  fn try_from(value: &str) -> Result<Self, Self::Error> {
    match value {
      "initial" => Ok(ChunkType::Initial),
      "async" => Ok(ChunkType::Async),
      "all" => Ok(ChunkType::All),
      _ => Err(format!("Invalid chunk type: {value}")),
    }
  }
}

impl ChunkType {
  pub fn is_selected(&self, chunk: &Chunk, chunk_group_by_ukey: &ChunkGroupByUkey) -> bool {
    match self {
      ChunkType::Initial => chunk.can_be_initial(chunk_group_by_ukey),
      ChunkType::Async => !chunk.can_be_initial(chunk_group_by_ukey),
      ChunkType::All => true,
    }
  }
}

impl Debug for ChunkType {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::Initial => write!(f, "Initial"),
      Self::Async => write!(f, "Async"),
      Self::All => write!(f, "All"),
      // Self::Custom(_) => write!(f, "Custom"),
    }
  }
}

#[derive(Derivative)]
#[derivative(Debug)]
#[derive(Clone, Default)]
pub struct CacheGroupOptions {
  pub priority: Option<i32>,
  pub reuse_existing_chunk: Option<bool>,
  pub r#type: Option<ModuleType>,
  #[derivative(Debug = "ignore")]
  pub test: Option<TestFn>,
  pub filename: Option<String>,
  pub enforce: Option<bool>,
  pub id_hint: Option<String>,

  /// What kind of chunks should be selected.
  pub chunks: Option<ChunkType>,
  pub automatic_name_delimiter: Option<String>,
  pub max_async_requests: Option<u32>,
  pub max_initial_requests: Option<u32>,
  pub min_chunks: Option<u32>,
  // hide_path_info: Option<bool>,
  pub min_size: Option<f64>,
  pub min_size_reduction: Option<f64>,
  pub enforce_size_threshold: Option<f64>,
  pub min_remaining_size: Option<f64>,
  // layer: Option<String>,
  pub max_size: Option<f64>,
  pub max_async_size: Option<f64>,
  pub max_initial_size: Option<f64>,
  // TODO: Option<supports> function
  pub name: Option<String>,
  // used_exports: Option<bool>,
}

pub type SizeType = SourceType;

#[derive(Default, Debug)]
pub struct SplitChunksOptions {
  pub cache_groups: HashMap<String, CacheGroupOptions>,
  /// What kind of chunks should be selected.
  pub chunks: Option<ChunkType>,
  pub automatic_name_delimiter: Option<String>,
  pub max_async_requests: Option<usize>,
  pub max_initial_requests: Option<usize>,
  pub default_size_types: Option<Vec<SizeType>>,
  pub min_chunks: Option<u32>,
  // hide_path_info: Option<bool>,
  pub min_size: Option<f64>,
  pub min_size_reduction: Option<f64>,
  pub enforce_size_threshold: Option<f64>,
  pub min_remaining_size: Option<f64>,
  // layer: Option<String>,
  pub max_size: Option<f64>,
  pub max_async_size: Option<f64>,
  pub max_initial_size: Option<f64>,
  // TODO: supports function
  pub name: Option<String>,
  pub fallback_cache_group: Option<FallbackCacheGroup>,
  // used_exports: bool,
}

#[derive(Debug)]
pub struct FallbackCacheGroup {
  pub automatic_name_delimiter: Option<String>,
  pub chunks: Option<ChunkType>,
  pub max_async_size: Option<f64>,
  pub max_initial_size: Option<f64>,
  pub max_size: Option<f64>,
  pub min_size: Option<f64>,
  pub min_size_reduction: Option<f64>,
}

#[derive(Derivative)]
#[derivative(Debug)]
pub struct NormalizedOptions {
  pub default_size_types: Vec<SizeType>,
  pub min_size: SplitChunkSizes,
  pub min_size_reduction: SplitChunkSizes,
  pub min_remaining_size: SplitChunkSizes,
  pub enforce_size_threshold: SplitChunkSizes,
  pub max_async_size: SplitChunkSizes,
  pub max_initial_size: SplitChunkSizes,
  pub min_chunks: u32,
  pub max_async_requests: u32,
  pub max_initial_requests: u32,
  pub filename: Option<String>,
  #[derivative(Debug = "ignore")]
  pub get_name: GetName,
  #[derivative(Debug = "ignore")]
  pub chunk_filter: ChunkFilter,
  pub fallback_cache_group: NormalizedFallbackCacheGroup,
}

#[derive(Derivative, Clone)]
#[derivative(Debug)]
pub struct NormalizedFallbackCacheGroup {
  #[derivative(Debug = "ignore")]
  pub chunks_filter: ChunkFilter,
  pub min_size: SplitChunkSizes,
  pub max_async_size: SplitChunkSizes,
  pub max_initial_size: SplitChunkSizes,
  pub automatic_name_delimiter: String,
}
