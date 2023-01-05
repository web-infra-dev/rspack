#![feature(option_get_or_insert_default)]

mod plugin;
use std::sync::Arc;

use hashbrown::HashMap;
pub use plugin::SplitChunksPlugin;
mod options;
pub use options::*;
use rspack_core::{Chunk, ChunkGroupByUkey, Module};

pub type TestFn = Arc<dyn Fn(&dyn Module) -> bool + Sync + Send>;
pub(crate) type GetName = Arc<dyn Fn(&dyn Module) -> Option<String> + Sync + Send>;
pub(crate) type ChunkFilter = Arc<dyn Fn(&Chunk, &ChunkGroupByUkey) -> bool + Send + Sync>;

mod utils;
pub(crate) type CacheGroupByKey = HashMap<String, CacheGroup>;
