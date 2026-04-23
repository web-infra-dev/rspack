use std::sync::{
  Arc, RwLock,
  atomic::{AtomicBool, Ordering},
};

use readable_identifier::*;
use rustc_hash::FxHashMap as HashMap;

pub type ModuleStaticCache = Arc<ModuleStaticCacheInner>;

/// This cache is used to cache the information of modules that are not changed after `make`.
#[derive(Debug, Default)]
pub struct ModuleStaticCacheInner {
  /// this is a fast-path check to avoid hash check
  cache_enabled: AtomicBool,
  readable_identifier_cache: ReadableIdentifierCache,
}

impl ModuleStaticCacheInner {
  pub fn enable_new_cache(&self) {
    // Only cache the readable identifier of compilation context
    self.cache_enabled.store(true, Ordering::Release);
    self.readable_identifier_cache.clear();
  }

  pub fn disable_cache(&self) {
    self.cache_enabled.store(false, Ordering::Release);
    self.readable_identifier_cache.clear();
  }
  pub fn cached_readable_identifier<F: FnOnce() -> String>(
    &self,
    key: ReadableIdentifierCacheKey,
    f: F,
  ) -> Arc<str> {
    if !self.cache_enabled.load(Ordering::Acquire) {
      return Arc::<str>::from(f());
    }

    match self.readable_identifier_cache.get_arc(&key) {
      Some(value) => value,
      None => {
        let value = Arc::<str>::from(f());
        self
          .readable_identifier_cache
          .set_arc(key, Arc::clone(&value));
        value
      }
    }
  }
}

pub(super) mod readable_identifier {
  use super::*;
  use crate::ModuleIdentifier;

  // When using compilation context, the context string should be `None`
  pub type ReadableIdentifierCacheKey = (ModuleIdentifier, Option<String>);

  #[derive(Debug, Default)]
  pub struct ReadableIdentifierCache {
    cache: RwLock<HashMap<ReadableIdentifierCacheKey, Arc<str>>>,
  }

  impl ReadableIdentifierCache {
    pub fn clear(&self) {
      self.cache.write().expect("should get lock").clear();
    }

    pub fn get_arc(&self, key: &ReadableIdentifierCacheKey) -> Option<Arc<str>> {
      let inner = self.cache.read().expect("should get lock");
      inner.get(key).cloned()
    }

    pub fn set_arc(&self, key: ReadableIdentifierCacheKey, value: Arc<str>) {
      self
        .cache
        .write()
        .expect("should get lock")
        .insert(key, value);
    }
  }
}

#[cfg(test)]
mod tests {
  use std::sync::Arc;

  use super::ModuleStaticCacheInner;
  use crate::ModuleIdentifier;

  #[test]
  fn cached_readable_identifier_reuses_the_same_allocation() {
    let cache = ModuleStaticCacheInner::default();
    cache.enable_new_cache();

    let key = (ModuleIdentifier::from("module-a"), None);
    let first = cache.cached_readable_identifier(key.clone(), || "alpha".to_string());
    let second = cache.cached_readable_identifier(key, || "beta".to_string());

    assert_eq!(&*first, "alpha");
    assert_eq!(&*second, "alpha");
    assert!(Arc::ptr_eq(&first, &second));
  }
}
