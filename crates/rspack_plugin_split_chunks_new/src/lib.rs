#![feature(map_many_mut)]

pub(crate) mod cache_group;
pub(crate) mod common;
pub(crate) mod module_group;
pub(crate) mod plugin;

pub use crate::{
  cache_group::CacheGroup,
  common::{
    create_all_chunk_filter, create_async_chunk_filter, create_chunk_filter_from_str,
    create_default_module_filter, create_initial_chunk_filter, create_module_filter,
    create_module_filter_from_regex, create_module_filter_from_rspack_regex,
  },
  plugin::{PluginOptions, SplitChunksPlugin},
};
