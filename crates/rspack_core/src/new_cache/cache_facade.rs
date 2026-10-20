use std::sync::Arc;

use super::{Cache, CacheKey, CacheValue, Etag, cache_value::CacheValueData};

/// A namespaced view of the shared cache.
///
/// This is the minimal equivalent of webpack's `CacheFacade`: it prefixes
/// identifiers with a fixed namespace and creates item facades.
#[derive(Debug, Clone)]
pub struct CacheFacade {
  cache: Arc<Cache>,
  name: String,
  allow_import: bool,
}

impl CacheFacade {
  pub(crate) fn new(cache: Arc<Cache>, name: String, allow_import: bool) -> Self {
    Self {
      cache,
      name,
      allow_import,
    }
  }

  pub(crate) fn get_child_cache(&self, name: &str) -> Self {
    Self::new(
      Arc::clone(&self.cache),
      format!("{}|{name}", self.name),
      self.allow_import,
    )
  }

  pub fn get_item_cache(&self, identifier: &str, etag: Option<Etag>) -> ItemCacheFacade {
    ItemCacheFacade {
      cache: self.cache.clone(),
      key: self.key(identifier),
      etag,
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

  pub(crate) fn module_profile(&self) -> Option<&str> {
    self.cache.module_profile()
  }

  pub(crate) fn loader_profile(&self) -> Option<&str> {
    self.cache.loader_profile()
  }

  pub(crate) fn get_imported<T: CacheValueData>(
    &self,
    identifier: &str,
    etag: Etag,
  ) -> Option<CacheValue<T>> {
    // Child compilers have independent dependency ID allocators and build contracts.
    // The draft supports only the root compiler's namespace.
    if !self.allow_import {
      return None;
    }
    self.cache.get_imported(self.key(identifier), etag)
  }

  fn key(&self, identifier: &str) -> CacheKey {
    CacheKey::from([self.name.as_ref(), identifier].join("|"))
  }
}

/// A cache facade with a fixed identifier and etag.
#[derive(Debug, Clone)]
pub struct ItemCacheFacade {
  cache: Arc<Cache>,
  key: CacheKey,
  etag: Option<Etag>,
}

impl ItemCacheFacade {
  pub fn get<T: CacheValueData>(&self) -> Option<CacheValue<T>> {
    self.cache.get(self.key.clone(), self.etag.clone())
  }

  pub fn store<T: CacheValueData>(&self, value: CacheValue<T>) {
    self.cache.store(self.key.clone(), self.etag.clone(), value)
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
