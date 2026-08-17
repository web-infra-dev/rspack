use std::{
  any::Any,
  sync::atomic::{AtomicU32, Ordering},
};

use rspack_util::fx_hash::FxDashMap;

use super::{CacheKey, CacheValue, Etag, cache_value::CacheEntry};

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

#[derive(Debug)]
enum MemoryCacheValue {
  Miss,
  Hit(CacheEntry),
}

#[derive(Debug)]
struct MemoryCacheEntry {
  value: MemoryCacheValue,
  ttl: AtomicU32,
}

impl MemoryCacheEntry {
  fn new(value: MemoryCacheValue, ttl: u32) -> Self {
    Self {
      value,
      ttl: AtomicU32::new(ttl),
    }
  }

  fn refresh(&self, ttl: u32) {
    self.ttl.store(ttl, Ordering::Relaxed);
  }

  fn start_next_generation(&mut self) -> bool {
    let ttl = self.ttl.get_mut();
    if *ttl == 0 {
      false
    } else {
      *ttl -= 1;
      true
    }
  }
}

/// In-memory cache with generation-based garbage collection.
#[derive(Debug)]
pub struct MemoryCache {
  max_generations: u32,
  entries: FxDashMap<CacheKey, MemoryCacheEntry>,
}

impl Default for MemoryCache {
  fn default() -> Self {
    Self::new(1)
  }
}

impl MemoryCache {
  pub fn new(max_generations: u32) -> Self {
    Self {
      max_generations,
      entries: FxDashMap::default(),
    }
  }

  pub fn get<T: Any + Send + Sync>(
    &self,
    key: &CacheKey,
    etag: Option<&Etag>,
  ) -> MemoryCacheGetResult<T> {
    let Some(entry) = self.entries.get(key) else {
      return MemoryCacheGetResult::NotCached;
    };
    entry.refresh(self.max_generations);
    let entry = match &entry.value {
      MemoryCacheValue::Miss => return MemoryCacheGetResult::Miss,
      MemoryCacheValue::Hit(entry) => entry,
    };
    if entry.matches(etag) {
      entry
        .value()
        .clone()
        .downcast()
        .map_or(MemoryCacheGetResult::Miss, MemoryCacheGetResult::Hit)
    } else {
      MemoryCacheGetResult::Miss
    }
  }

  pub fn store<T: Any + Send + Sync>(
    &self,
    key: CacheKey,
    etag: Option<Etag>,
    value: CacheValue<T>,
  ) {
    self.entries.insert(
      key,
      MemoryCacheEntry::new(
        MemoryCacheValue::Hit(CacheEntry::new(etag, value.erase_memory())),
        self.max_generations,
      ),
    );
  }

  pub fn store_miss(&self, key: CacheKey) {
    self.entries.insert(
      key,
      MemoryCacheEntry::new(MemoryCacheValue::Miss, self.max_generations),
    );
  }

  /// Starts a new compilation generation and evicts entries that were not
  /// accessed during the configured number of previous generations.
  pub fn start_next_generation(&self) {
    self
      .entries
      .retain(|_, entry| entry.start_next_generation());
  }

  pub fn clear(&self) {
    self.entries.clear();
  }
}
