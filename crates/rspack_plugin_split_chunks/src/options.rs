use std::{collections::HashMap, fmt::Debug, str::FromStr, sync::Arc};

use rspack_core::{Chunk, ChunkGroupByUkey, ModuleGraphModule};

pub(crate) type SplitChunkSizes = HashMap<SizeType, usize>;

impl Debug for CacheGroupSource {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("CacheGroupSource")
      .field("key", &self.key)
      .field("priority", &self.priority)
      .field("get_name", &"Fn")
      .field("chunks_filter", &"Fn")
      .field("enforce", &self.enforce)
      .field("min_chunks", &self.min_chunks)
      .field("max_async_requests", &self.max_async_requests)
      .field("max_initial_requests", &self.max_initial_requests)
      .field("filename", &self.filename)
      .field("id_hint", &self.id_hint)
      .field("automatic_name_delimiter", &self.automatic_name_delimiter)
      .field("reuse_existing_chunk", &self.reuse_existing_chunk)
      .field("min_size", &self.min_size)
      .field("min_size_reduction", &self.min_size_reduction)
      .field("min_remaining_size", &self.min_remaining_size)
      .field("enforce_size_threshold", &self.enforce_size_threshold)
      .field("max_async_size", &self.max_async_size)
      .field("max_initial_size", &self.max_initial_size)
      .finish()
  }
}

pub struct CacheGroupSource {
  pub key: String,
  pub priority: isize,
  pub get_name: Arc<dyn Fn() -> String + Send + Sync>,
  pub chunks_filter: Arc<dyn Fn(&Chunk) -> bool + Send + Sync>,
  pub enforce: bool,
  pub min_chunks: usize,
  pub max_async_requests: usize,
  pub max_initial_requests: usize,
  pub filename: String,
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
}

pub struct CacheGroup {
  pub key: String,
  pub priority: isize,
  pub get_name: Arc<dyn Fn() -> String + Send + Sync>,
  pub chunks_filter: Arc<dyn Fn(&Chunk) -> bool + Send + Sync>,
  pub min_chunks: usize,
  pub max_async_requests: usize,
  pub max_initial_requests: usize,
  pub filename: String,
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
}

impl Debug for CacheGroup {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("CacheGroup")
      .field("key", &self.key)
      .field("priority", &self.priority)
      .field("get_name", &"Fn")
      .field("chunks_filter", &"Fn")
      .field("min_chunks", &self.min_chunks)
      .field("max_async_requests", &self.max_async_requests)
      .field("max_initial_requests", &self.max_initial_requests)
      .field("filename", &self.filename)
      .field("id_hint", &self.id_hint)
      .field("automatic_name_delimiter", &self.automatic_name_delimiter)
      .field("reuse_existing_chunk", &self.reuse_existing_chunk)
      .field("min_size", &self.min_size)
      .field("min_size_reduction", &self.min_size_reduction)
      .field("min_remaining_size", &self.min_remaining_size)
      .field("enforce_size_threshold", &self.enforce_size_threshold)
      .field("max_async_size", &self.max_async_size)
      .field("max_initial_size", &self.max_initial_size)
      .finish()
  }
}

pub enum ChunkType {
  Initial,
  Async,
  Custom(Box<dyn Fn(&Chunk) -> bool + Sync + Send>),
}

impl TryFrom<&str> for ChunkType {
  type Error = String;

  fn try_from(value: &str) -> Result<Self, Self::Error> {
    match value {
      "initial" => Ok(ChunkType::Initial),
      "async" => Ok(ChunkType::Async),
      _ => Err(format!("Invalid chunk type: {}", value)),
    }
  }
}

impl ChunkType {
  pub fn is_selected(&self, chunk: &Chunk, chunk_group_by_ukey: &ChunkGroupByUkey) -> bool {
    match self {
      ChunkType::Initial => chunk.can_be_initial(chunk_group_by_ukey),
      ChunkType::Async => !chunk.can_be_initial(chunk_group_by_ukey),
      ChunkType::Custom(f) => f(chunk),
    }
  }
}

impl Debug for ChunkType {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::Initial => write!(f, "Initial"),
      Self::Async => write!(f, "Async"),
      Self::Custom(_) => write!(f, "Custom"),
    }
  }
}

pub struct CacheGroupOptions {
  pub priority: isize,
  pub reuse_existing_chunk: bool,
  pub r#type: SizeType,
  pub test: Arc<dyn Fn(&ModuleGraphModule) -> bool + Sync + Send>,
  pub filename: String,
  pub enforce: bool,
  pub id_hint: String,

  /// What kind of chunks should be selected.
  pub chunks: ChunkType,
  pub automatic_name_delimiter: String,
  pub max_async_requests: usize,
  pub max_initial_requests: usize,
  pub min_chunks: usize,
  // hide_path_info: bool,
  pub min_size: usize,
  pub min_size_reduction: usize,
  pub enforce_size_threshold: usize,
  pub min_remaining_size: usize,
  // layer: String,
  pub max_size: usize,
  pub max_async_size: usize,
  pub max_initial_size: usize,
  // TODO: supports function
  pub name: String,
  // used_exports: bool,
}

impl Debug for CacheGroupOptions {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("CacheGroupOptions")
      .field("priority", &self.priority)
      .field("reuse_existing_chunk", &self.reuse_existing_chunk)
      .field("test", &"Fn")
      .field("filename", &self.filename)
      .field("enforce", &self.enforce)
      .field("id_hint", &self.id_hint)
      .field("chunks", &self.chunks)
      .field("automatic_name_delimiter", &self.automatic_name_delimiter)
      .field("max_async_requests", &self.max_async_requests)
      .field("max_initial_requests", &self.max_initial_requests)
      .field("min_chunks", &self.min_chunks)
      .field("min_size", &self.min_size)
      .field("min_size_reduction", &self.min_size_reduction)
      .field("enforce_size_threshold", &self.enforce_size_threshold)
      .field("min_remaining_size", &self.min_remaining_size)
      .field("max_size", &self.max_size)
      .field("max_async_size", &self.max_async_size)
      .field("max_initial_size", &self.max_initial_size)
      .field("name", &self.name)
      .finish()
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SizeType {
  JavaScript,
  Unknown,
}

#[derive(Debug)]
pub struct SplitChunksOptions {
  pub cache_groups: HashMap<String, CacheGroupOptions>,
  /// What kind of chunks should be selected.
  pub chunks: ChunkType,
  pub automatic_name_delimiter: String,
  pub max_async_requests: usize,
  pub max_initial_requests: usize,
  pub default_size_types: Vec<SizeType>,
  pub min_chunks: usize,
  // hide_path_info: bool,
  pub min_size: usize,
  pub min_size_reduction: usize,
  pub enforce_size_threshold: usize,
  pub min_remaining_size: usize,
  // layer: String,
  pub max_size: usize,
  pub max_async_size: usize,
  pub max_initial_size: usize,
  // TODO: supports function
  // name: String,
  // used_exports: bool,
}

impl Default for SplitChunksOptions {
  fn default() -> Self {
    Self {
      chunks: ChunkType::Async,
      automatic_name_delimiter: "~".to_string(),
      max_async_requests: 30,
      max_initial_requests: 30,
      default_size_types: vec![SizeType::JavaScript, SizeType::Unknown],
      min_chunks: 1,
      min_size: 20000,
      min_size_reduction: 20000,
      enforce_size_threshold: 50000,
      min_remaining_size: 0,
      max_size: 0,
      max_async_size: usize::MAX,
      max_initial_size: usize::MAX,

      cache_groups: Default::default(),
    }
  }
}
