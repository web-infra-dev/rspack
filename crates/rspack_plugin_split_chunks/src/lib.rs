#![feature(map_many_mut)]
#![feature(let_chains)]

mod common;
mod module_group;
mod options;
mod plugin;

pub use common::{
  create_all_chunk_filter, create_async_chunk_filter, create_chunk_filter_from_str,
  create_default_module_type_filter, create_initial_chunk_filter,
  create_regex_chunk_filter_from_str, ChunkFilter, FallbackCacheGroup, ModuleTypeFilter,
  SplitChunkSizes,
};
pub use options::cache_group::CacheGroup;
pub use options::cache_group_test::{CacheGroupTest, CacheGroupTestFnCtx};
pub use options::chunk_name::{ChunkNameGetter, ChunkNameGetterFnCtx};
pub use plugin::{PluginOptions, SplitChunksPlugin};
