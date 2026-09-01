use std::sync::Arc;

use rspack_error::Result;

use super::{Cache, CacheKey, CacheValue, Etag, cache_value::CacheValueData};

/// A namespaced view of the shared cache.
///
/// This is the minimal equivalent of webpack's `CacheFacade`: it prefixes
/// identifiers with a fixed namespace and creates child or item facades.
#[derive(Debug, Clone)]
pub struct CacheFacade {
  cache: Cache,
  name: Arc<str>,
}

impl CacheFacade {
  pub(crate) fn new(cache: Cache, name: impl Into<Arc<str>>) -> Self {
    Self {
      cache,
      name: name.into(),
    }
  }

  pub fn get_child_cache(&self, name: &str) -> Self {
    Self {
      cache: self.cache.clone(),
      name: join_name(&self.name, name, true),
    }
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
  ) -> Result<Option<CacheValue<T>>> {
    self.cache.get(self.key(identifier), etag)
  }

  pub(crate) fn get_without_memory<T: CacheValueData>(
    &self,
    identifier: &str,
    etag: Option<Etag>,
  ) -> Result<Option<CacheValue<T>>> {
    self.cache.get_without_memory(self.key(identifier), etag)
  }

  pub fn store<T: CacheValueData>(
    &self,
    identifier: &str,
    etag: Option<Etag>,
    value: CacheValue<T>,
  ) -> Result<()> {
    self.cache.store(self.key(identifier), etag, value)
  }

  pub(crate) fn store_without_memory<T: CacheValueData>(
    &self,
    identifier: &str,
    etag: Option<Etag>,
    value: CacheValue<T>,
  ) -> Result<()> {
    self
      .cache
      .store_without_memory(self.key(identifier), etag, value)
  }

  fn key(&self, identifier: &str) -> CacheKey {
    CacheKey::from(join_name(&self.name, identifier, true))
  }
}

/// A cache facade with a fixed identifier and etag.
#[derive(Debug, Clone)]
pub struct ItemCacheFacade {
  cache: Cache,
  key: CacheKey,
  etag: Option<Etag>,
}

impl ItemCacheFacade {
  pub fn get<T: CacheValueData>(&self) -> Result<Option<CacheValue<T>>> {
    self.cache.get(self.key.clone(), self.etag.clone())
  }

  pub fn store<T: CacheValueData>(&self, value: CacheValue<T>) -> Result<()> {
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

  pub fn get<T: CacheValueData>(&self) -> Result<Option<CacheValue<T>>> {
    for item in &self.items {
      if let Some(value) = item.get()? {
        return Ok(Some(value));
      }
    }
    Ok(None)
  }

  pub fn store<T: CacheValueData>(&self, value: CacheValue<T>) -> Result<()> {
    for item in &self.items {
      item.store(value.clone())?;
    }
    Ok(())
  }
}

fn join_name(prefix: &str, name: &str, with_separator: bool) -> Arc<str> {
  let mut result = String::with_capacity(prefix.len() + name.len() + usize::from(with_separator));
  result.push_str(prefix);
  if with_separator {
    result.push('|');
  }
  result.push_str(name);
  result.into()
}
