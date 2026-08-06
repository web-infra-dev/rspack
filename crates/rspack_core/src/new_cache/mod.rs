// The compiler integration is added after these foundational cache layers.
#![allow(dead_code, unused_imports)]

mod file_cache_strategy;
mod idle_file_cache;
mod memory_cache;

use std::sync::Arc;

pub use file_cache_strategy::FileCacheStrategy;
pub use idle_file_cache::IdleFileCache;
pub use memory_cache::{MemoryCache, MemoryCacheGetResult};

/// Serialized cache payload shared by the memory and filesystem cache layers.
pub type CacheData = Arc<[u8]>;

/// String representation of the inputs used to validate a cache entry.
pub type Etag = Arc<str>;
