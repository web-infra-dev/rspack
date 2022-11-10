use std::{collections::HashMap, fmt::Debug, sync::Arc};

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
  pub key: Option<String>,
  pub priority: Option<isize>,
  pub get_name: Option<Arc<dyn Fn() -> String + Send + Sync>>,
  pub chunks_filter: Option<Arc<dyn Fn(&Chunk) -> bool + Send + Sync>>,
  pub enforce: Option<bool>,
  pub min_chunks: Option<usize>,
  pub max_async_requests: Option<usize>,
  pub max_initial_requests: Option<usize>,
  pub filename: Option<String>,
  pub id_hint: Option<String>,
  pub automatic_name_delimiter: String,
  pub reuse_existing_chunk: Option<bool>,
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

#[derive(Clone)]
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
      _ => Err(format!("Invalid chunk type: {}", value)),
    }
  }
}

impl ChunkType {
  pub fn is_selected(&self, chunk: &Chunk, chunk_group_by_ukey: &ChunkGroupByUkey) -> bool {
    match self {
      ChunkType::Initial => chunk.can_be_initial(chunk_group_by_ukey),
      ChunkType::Async => !chunk.can_be_initial(chunk_group_by_ukey),
      ChunkType::All => true,
      // ChunkType::Custom(f) => f(chunk),
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

#[derive(Clone, Default)]
pub struct CacheGroupOptions {
  pub priority: Option<isize>,
  pub reuse_existing_chunk: Option<bool>,
  pub r#type: Option<SizeType>,
  pub test: Option<Arc<dyn Fn(&ModuleGraphModule) -> bool + Sync + Send>>,
  pub filename: Option<String>,
  pub enforce: Option<bool>,
  pub id_hint: Option<String>,

  /// What kind of chunks should be selected.
  pub chunks: Option<ChunkType>,
  pub automatic_name_delimiter: Option<String>,
  pub max_async_requests: Option<usize>,
  pub max_initial_requests: Option<usize>,
  pub min_chunks: Option<usize>,
  // hide_path_info: Option<bool>,
  pub min_size: Option<usize>,
  pub min_size_reduction: Option<usize>,
  pub enforce_size_threshold: Option<usize>,
  pub min_remaining_size: Option<usize>,
  // layer: Option<String>,
  pub max_size: Option<usize>,
  pub max_async_size: Option<usize>,
  pub max_initial_size: Option<usize>,
  // TODO: Option<supports> function
  pub name: Option<String>,
  // used_exports: Option<bool>,
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
  Css,
}

#[derive(Debug, Default)]
pub struct SplitChunksOptions {
  pub cache_groups: HashMap<String, CacheGroupOptions>,
  /// What kind of chunks should be selected.
  pub chunks: Option<ChunkType>,
  pub automatic_name_delimiter: Option<String>,
  pub max_async_requests: Option<usize>,
  pub max_initial_requests: Option<usize>,
  pub default_size_types: Vec<Option<SizeType>>,
  pub min_chunks: Option<usize>,
  // hide_path_info: Option<bool>,
  pub min_size: Option<usize>,
  pub min_size_reduction: Option<usize>,
  pub enforce_size_threshold: Option<usize>,
  pub min_remaining_size: Option<usize>,
  // layer: Option<String>,
  pub max_size: Option<usize>,
  pub max_async_size: Option<usize>,
  pub max_initial_size: Option<usize>,
  // TODO: supports function
  // name: String,
  // used_exports: bool,
}
