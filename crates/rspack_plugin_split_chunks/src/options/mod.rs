use std::fmt::Debug;

use derivative::Derivative;
use rspack_core::{Chunk, ChunkGroupByUkey, SourceType};

use crate::{cache_group_source::SplitChunkSizes, ChunkFilterFn, SplitChunksNameFn};

mod split_chunks_options;
pub use split_chunks_options::*;
mod cache_group_options;
pub use cache_group_options::*;

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

pub type SizeType = SourceType;

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
  pub get_name: SplitChunksNameFn,
  #[derivative(Debug = "ignore")]
  pub chunk_filter: ChunkFilterFn,
  pub fallback_cache_group: NormalizedFallbackCacheGroup,
}

#[derive(Derivative, Clone)]
#[derivative(Debug)]
pub struct NormalizedFallbackCacheGroup {
  #[derivative(Debug = "ignore")]
  pub chunks_filter: ChunkFilterFn,
  pub min_size: SplitChunkSizes,
  pub max_async_size: SplitChunkSizes,
  pub max_initial_size: SplitChunkSizes,
  pub automatic_name_delimiter: String,
}
