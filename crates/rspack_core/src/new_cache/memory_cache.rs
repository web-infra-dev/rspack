use dashmap::DashMap;

use super::{CacheData, Etag};

#[derive(Debug, Clone)]
struct MemoryCacheEntry {
  etag: Option<Etag>,
  data: CacheData,
}

/// Result of looking up an item in the memory cache.
///
/// `NotCached` lets the caller continue with the filesystem cache. `Miss`
/// records a known miss and stops lower cache layers from being queried.
#[derive(Debug, Clone)]
pub enum MemoryCacheGetResult {
  NotCached,
  Miss,
  Hit(CacheData),
}

/// In-memory cache equivalent to webpack's `MemoryCachePlugin`.
#[derive(Debug, Default)]
pub struct MemoryCache {
  entries: DashMap<String, Option<MemoryCacheEntry>>,
}

impl MemoryCache {
  pub fn get(&self, identifier: &str, etag: Option<&str>) -> MemoryCacheGetResult {
    let Some(entry) = self.entries.get(identifier) else {
      return MemoryCacheGetResult::NotCached;
    };
    let Some(entry) = entry.value() else {
      return MemoryCacheGetResult::Miss;
    };
    if entry.etag.as_deref() == etag {
      MemoryCacheGetResult::Hit(entry.data.clone())
    } else {
      MemoryCacheGetResult::Miss
    }
  }

  pub fn store(&self, identifier: impl Into<String>, etag: Option<Etag>, data: CacheData) {
    self
      .entries
      .insert(identifier.into(), Some(MemoryCacheEntry { etag, data }));
  }

  pub fn store_miss(&self, identifier: impl Into<String>) {
    self.entries.insert(identifier.into(), None);
  }

  pub fn clear(&self) {
    self.entries.clear();
  }
}
