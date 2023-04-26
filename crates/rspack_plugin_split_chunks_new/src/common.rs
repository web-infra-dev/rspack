use std::sync::Arc;

use rspack_core::{Chunk, ChunkGroupByUkey, Module, SourceType};
use rustc_hash::FxHashMap;

pub type ChunkFilter = Arc<dyn Fn(&Chunk, &ChunkGroupByUkey) -> bool + Send + Sync>;

pub fn create_async_chunk_filter() -> ChunkFilter {
  Arc::new(|chunk, chunk_group_db| !chunk.can_be_initial(chunk_group_db))
}

pub fn create_initial_chunk_filter() -> ChunkFilter {
  Arc::new(|chunk, chunk_group_db| chunk.can_be_initial(chunk_group_db))
}

pub fn create_all_chunk_filter() -> ChunkFilter {
  Arc::new(|_chunk, _chunk_group_db| true)
}

pub fn create_chunk_filter_from_str(chunks: &str) -> ChunkFilter {
  match chunks {
    "initial" => create_initial_chunk_filter(),
    "async" => create_async_chunk_filter(),
    "all" => create_all_chunk_filter(),
    _ => panic!("Invalid chunk type: {chunks}"),
  }
}

pub type ModuleFilter = Arc<dyn Fn(&dyn Module) -> bool + Send + Sync>;

pub fn create_default_module_filter() -> ModuleFilter {
  Arc::new(|_| true)
}

pub fn create_module_filter_from_rspack_regex(re: rspack_regex::RspackRegex) -> ModuleFilter {
  Arc::new(move |module| {
    module
      .name_for_condition()
      .map_or(false, |name| re.test(&name))
  })
}

pub fn create_module_filter_from_regex(re: regex::Regex) -> ModuleFilter {
  Arc::new(move |module| {
    module
      .name_for_condition()
      .map_or(false, |name| re.is_match(&name))
  })
}

pub fn create_module_filter(re: Option<String>) -> ModuleFilter {
  re.map(|test| {
    let re = regex::Regex::new(&test).unwrap_or_else(|_| panic!("Invalid regex: {}", &test));
    create_module_filter_from_regex(re)
  })
  .unwrap_or_else(create_default_module_filter)
}

pub(crate) type SplitChunkSizes = FxHashMap<SourceType, f64>;
