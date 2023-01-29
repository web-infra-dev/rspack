#![feature(option_get_or_insert_default)]
#![feature(map_many_mut)]

mod plugin;
use std::sync::Arc;

use rustc_hash::FxHashMap as HashMap;
mod options;
pub use options::*;
use rspack_core::{Chunk, ChunkGroupByUkey, Module};

pub type TestFn = Arc<dyn Fn(&dyn Module) -> bool + Sync + Send>;
pub(crate) type SplitChunksNameFn = Arc<dyn Fn(&dyn Module) -> Option<String> + Sync + Send>;
pub(crate) type ChunkFilterFn = Arc<dyn Fn(&Chunk, &ChunkGroupByUkey) -> bool + Send + Sync>;

mod utils;
pub(crate) type CacheGroupByKey = HashMap<String, CacheGroup>;
pub(crate) type ChunksInfoMap = HashMap<String, ChunksInfoItem>;

mod cache_group;
pub(crate) use cache_group::*;
mod cache_group_source;
pub(crate) use cache_group_source::*;
mod split_chunks_plugin;
// pub(crate) use split_chunks_plugin::*;
mod chunks_info_item;
pub(crate) use chunks_info_item::*;
mod max_size_queue_item;
// pub(crate) use max_size_queue_item::*;
// public
pub use split_chunks_plugin::SplitChunksPlugin;

// TODO: Webpack also supports a HashMap here, which is not supported yet.
pub(crate) type OptimizationSplitChunksSizes = f64;
pub(crate) type SplitChunksSizes = HashMap<SizeType, f64>;
