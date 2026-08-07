use dashmap::DashMap;
use rspack_util::fx_hash::FxDashMap;

use super::{
  CacheKey, CacheValue, Etag,
  cache_value::{CacheEntry, CacheValueData},
};

/// Result of looking up an item in the memory cache.
///
/// `NotCached` lets the caller continue with the filesystem cache. `Miss`
/// records a known miss and stops lower cache layers from being queried.
#[derive(Debug)]
pub enum MemoryCacheGetResult<T> {
  NotCached,
  Miss,
  Hit(CacheValue<T>),
}

impl<T> Clone for MemoryCacheGetResult<T> {
  fn clone(&self) -> Self {
    match self {
      Self::NotCached => Self::NotCached,
      Self::Miss => Self::Miss,
      Self::Hit(value) => Self::Hit(value.clone()),
    }
  }
}

/// In-memory cache equivalent to webpack's `MemoryCachePlugin`.
#[derive(Debug, Default)]
pub struct MemoryCache {
  entries: FxDashMap<CacheKey, Option<CacheEntry>>,
}

impl MemoryCache {
  pub fn get<T: CacheValueData>(
    &self,
    key: CacheKey,
    etag: Option<Etag>,
  ) -> MemoryCacheGetResult<T> {
    let Some(entry) = self.entries.get(&key) else {
      return MemoryCacheGetResult::NotCached;
    };
    let Some(entry) = entry.value() else {
      return MemoryCacheGetResult::Miss;
    };
    if entry.matches(&etag) {
      entry
        .value()
        .clone()
        .downcast()
        .map_or(MemoryCacheGetResult::Miss, MemoryCacheGetResult::Hit)
    } else {
      MemoryCacheGetResult::Miss
    }
  }

  pub fn store<T: CacheValueData>(&self, key: CacheKey, etag: Option<Etag>, value: CacheValue<T>) {
    self
      .entries
      .insert(key, Some(CacheEntry::new(etag, value.erase())));
  }

  pub fn store_miss(&self, key: CacheKey) {
    self.entries.insert(key, None);
  }

  pub fn clear(&self) {
    self.entries.clear();
  }
}
