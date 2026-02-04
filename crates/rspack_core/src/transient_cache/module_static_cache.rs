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
  freezed: AtomicBool,
  readable_identifier_cache: ReadableIdentifierCache,
}

impl ModuleStaticCacheInner {
  pub fn freeze(&self) {
    // Only cache the readable identifier of compilation context
    self.readable_identifier_cache.freeze();
    self.freezed.store(true, Ordering::Release);
  }

  pub fn unfreeze(&self) {
    self.freezed.store(false, Ordering::Release);
  }
  pub fn cached_readable_identifier<F: FnOnce() -> String>(
    &self,
    key: ReadableIdentifierCacheKey,
    f: F,
  ) -> String {
    if !self.freezed.load(Ordering::Acquire) {
      return f();
    }

    match self.readable_identifier_cache.get(&key) {
      Some(value) => value,
      None => {
        let value = f();
        self.readable_identifier_cache.set(key, value.clone());
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
    cache: RwLock<HashMap<ReadableIdentifierCacheKey, String>>,
  }

  impl ReadableIdentifierCache {
    pub fn freeze(&self) {
      self.cache.write().expect("should get lock").clear();
    }

    pub fn get(&self, key: &ReadableIdentifierCacheKey) -> Option<String> {
      let inner = self.cache.read().expect("should get lock");
      inner.get(key).cloned()
    }

    pub fn set(&self, key: ReadableIdentifierCacheKey, value: String) {
      self
        .cache
        .write()
        .expect("should get lock")
        .insert(key, value);
    }
  }
}
