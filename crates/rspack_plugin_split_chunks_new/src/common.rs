use std::{
  future::{self, Future},
  ops::{Deref, DerefMut},
  pin::Pin,
  sync::Arc,
};

use futures_util::FutureExt;
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

fn create_default_module_filter() -> ModuleFilter {
  Arc::new(|_| true)
}

pub fn create_module_filter_from_rspack_regex(re: rspack_regex::RspackRegex) -> ModuleFilter {
  Arc::new(move |module| {
    module
      .name_for_condition()
      .map_or(false, |name| re.test(&name))
  })
}

pub fn create_module_filter(re: Option<String>) -> ModuleFilter {
  re.map(|test| {
    let re =
      rspack_regex::RspackRegex::new(&test).unwrap_or_else(|_| panic!("Invalid regex: {}", &test));
    create_module_filter_from_rspack_regex(re)
  })
  .unwrap_or_else(create_default_module_filter)
}

#[derive(Debug, Default)]
pub struct SplitChunkSizes(FxHashMap<SourceType, f64>);

impl SplitChunkSizes {
  pub fn empty() -> Self {
    Self(Default::default())
  }

  pub fn with_initial_value(default_size_types: &[SourceType], initial_bytes: f64) -> Self {
    Self(
      default_size_types
        .iter()
        .map(|ty| (*ty, initial_bytes))
        .collect(),
    )
  }
}

impl Deref for SplitChunkSizes {
  type Target = FxHashMap<SourceType, f64>;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl DerefMut for SplitChunkSizes {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.0
  }
}

type PinFutureBox<T> = Pin<Box<dyn Future<Output = T> + Send>>;

pub type ChunkNameGetter = Arc<dyn Fn(&dyn Module) -> PinFutureBox<Option<String>> + Send + Sync>;

pub fn create_chunk_name_getter_by_const_name(name: String) -> ChunkNameGetter {
  Arc::new(move |_module| future::ready(Some(name.clone())).boxed())
}

pub fn create_empty_chunk_name_getter() -> ChunkNameGetter {
  Arc::new(move |_module| future::ready(None).boxed())
}
