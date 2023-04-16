#![feature(map_many_mut)]

pub(crate) mod cache_group;
pub(crate) mod chunks_info_item;
pub(crate) mod common;
pub(crate) mod plugin;

pub use crate::{
  cache_group::CacheGroup,
  common::{
    create_all_chunk_filter, create_async_chunk_filter, create_default_module_filter,
    create_initial_chunk_filter, create_module_filter, create_module_filter_from_regex,
    create_module_filter_from_rspack_regex,
  },
  plugin::{PluginOptions, SplitChunksPlugin},
};

pub(crate) struct Mb(f64);

impl Mb {
  pub fn from_mb(mb: f64) -> Self {
    Self(mb * 1024.0 * 1024.0)
  }

  pub fn as_bytes(&self) -> f64 {
    self.0
  }
}
