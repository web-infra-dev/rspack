#![feature(option_get_or_insert_default)]

mod plugin;
use std::sync::Arc;

pub use plugin::SplitChunksPlugin;
mod options;
pub use options::*;
use rspack_core::{Chunk, ChunkGroupByUkey, Module};

pub(crate) type TestFn = Arc<dyn Fn(&dyn Module) -> bool + Sync + Send>;
pub(crate) type GetName = Arc<dyn Fn(&dyn Module) -> String + Sync + Send>;
pub(crate) type ChunkFilter = Arc<dyn Fn(&Chunk, &ChunkGroupByUkey) -> bool + Send + Sync>;
