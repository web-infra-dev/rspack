use std::sync::Arc;

use super::{
  Cache, CacheKey, CacheRecord, CacheValue, Etag,
  cache_value::CacheValueData,
  statistics::{CacheCounters, CacheStatistics},
};

/// A namespaced view of the shared cache.
///
/// This is the minimal equivalent of webpack's `CacheFacade`: it prefixes
/// identifiers with a fixed namespace and creates item facades.
#[derive(Debug, Clone)]
pub struct CacheFacade {
  cache: Arc<Cache>,
  name: Arc<str>,
  statistics: Option<Arc<CacheStatistics>>,
  counters: Option<Arc<CacheCounters>>,
}

impl CacheFacade {
  pub(super) fn new(
    cache: Arc<Cache>,
    name: Arc<str>,
    statistics: Option<Arc<CacheStatistics>>,
  ) -> Self {
    let counters = statistics
      .as_ref()
      .map(|statistics| statistics.register(Arc::clone(&name)));
    Self {
      cache,
      name,
      statistics,
      counters,
    }
  }

  pub(crate) fn get_child_cache(&self, name: &str) -> Self {
    Self::new(
      Arc::clone(&self.cache),
      format!("{}|{name}", self.name).into(),
      self.statistics.clone(),
    )
  }

  pub fn get_item_cache(&self, identifier: &str, etag: Option<Etag>) -> ItemCacheFacade {
    ItemCacheFacade {
      cache: self.cache.clone(),
      key: self.key(identifier),
      etag,
      counters: self.counters.clone(),
    }
  }

  pub fn get<T: CacheValueData>(
    &self,
    identifier: &str,
    etag: Option<Etag>,
  ) -> Option<CacheValue<T>> {
    self.cache.get(self.key(identifier), etag)
  }

  pub fn store<T: CacheValueData>(
    &self,
    identifier: &str,
    etag: Option<Etag>,
    value: CacheValue<T>,
  ) {
    self.cache.store(self.key(identifier), etag, value)
  }

  fn key(&self, identifier: &str) -> CacheKey {
    CacheKey::from([self.name.as_ref(), identifier].join("|"))
  }

  /// Records the final outcome of a cache lookup in this namespace.
  pub fn record(&self, record: CacheRecord) {
    if let Some(counters) = &self.counters {
      counters.record(record);
    }
  }
}

/// A cache facade with a fixed identifier and etag.
#[derive(Debug, Clone)]
pub struct ItemCacheFacade {
  cache: Arc<Cache>,
  key: CacheKey,
  etag: Option<Etag>,
  counters: Option<Arc<CacheCounters>>,
}

impl ItemCacheFacade {
  pub fn get<T: CacheValueData>(&self) -> Option<CacheValue<T>> {
    self.cache.get(self.key.clone(), self.etag.clone())
  }

  pub fn store<T: CacheValueData>(&self, value: CacheValue<T>) {
    self.cache.store(self.key.clone(), self.etag.clone(), value)
  }

  /// Records the final lookup outcome in the originating facade's namespace.
  pub fn record(&self, record: CacheRecord) {
    if let Some(counters) = &self.counters {
      counters.record(record);
    }
  }
}

/// A cache facade backed by multiple item caches.
///
/// Reads return the first available value. Stores write the value to every
/// item, allowing equivalent cache entries to be addressed by multiple keys.
#[derive(Debug, Clone)]
pub struct MultiItemCache {
  items: Vec<ItemCacheFacade>,
}

impl MultiItemCache {
  pub fn new(items: impl IntoIterator<Item = ItemCacheFacade>) -> Self {
    Self {
      items: items.into_iter().collect(),
    }
  }

  pub fn get<T: CacheValueData>(&self) -> Option<CacheValue<T>> {
    for item in &self.items {
      if let Some(value) = item.get() {
        return Some(value);
      }
    }
    None
  }

  pub fn store<T: CacheValueData>(&self, value: CacheValue<T>) {
    for item in &self.items {
      item.store(value.clone());
    }
  }
}
