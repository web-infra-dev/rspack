use std::{any::Any, fmt, ops::Deref, sync::Arc};

use rspack_cacheable::{cacheable, cacheable_dyn};

use super::etag::Etag;

/// Actual data stored in [`CacheValue`].
///
/// Concrete data types must be cacheable, and their implementations must use
/// `#[rspack_cacheable::cacheable_dyn]` so they can be restored through this
/// trait object.
#[cacheable_dyn]
pub trait CacheValueData: Any + fmt::Debug + Send + Sync {}

/// Shared immutable cache value.
///
/// Cloning this wrapper only increments an `Arc`; the cached object itself is
/// never cloned.
pub struct CacheValue<T>(Arc<T>);

impl<T> CacheValue<T> {
  pub fn new(value: T) -> Self {
    Self(Arc::new(value))
  }

  pub fn from_arc(value: Arc<T>) -> Self {
    Self(value)
  }

  pub fn as_arc(&self) -> &Arc<T> {
    &self.0
  }

  pub fn into_arc(self) -> Arc<T> {
    self.0
  }
}

impl<T: CacheValueData> CacheValue<T> {
  pub(super) fn erase(self) -> ErasedCacheValue {
    ErasedCacheValue(self.0)
  }
}

impl<T> Clone for CacheValue<T> {
  fn clone(&self) -> Self {
    Self(self.0.clone())
  }
}

impl<T: fmt::Debug> fmt::Debug for CacheValue<T> {
  fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
    self.0.fmt(formatter)
  }
}

impl<T> Deref for CacheValue<T> {
  type Target = T;

  fn deref(&self) -> &Self::Target {
    self.0.as_ref()
  }
}

impl<T> From<Arc<T>> for CacheValue<T> {
  fn from(value: Arc<T>) -> Self {
    Self::from_arc(value)
  }
}

/// Type-erased value used only inside the shared cache layers.
///
/// Serialization is performed by the filesystem strategy only when the
/// background job processes idle writes.
#[cacheable]
#[derive(Clone)]
pub(super) struct ErasedCacheValue(Arc<dyn CacheValueData>);

impl ErasedCacheValue {
  pub(super) fn downcast<T: CacheValueData>(self) -> Option<CacheValue<T>> {
    let value: Arc<dyn Any + Send + Sync> = self.0;
    Arc::downcast(value).ok().map(CacheValue::from_arc)
  }
}

impl fmt::Debug for ErasedCacheValue {
  fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
    self.0.fmt(formatter)
  }
}

#[cacheable]
#[derive(Debug)]
pub(super) struct CacheEntry {
  etag: Option<Etag>,
  value: ErasedCacheValue,
}

impl CacheEntry {
  pub(super) fn new(etag: Option<Etag>, value: ErasedCacheValue) -> Self {
    Self { etag, value }
  }

  pub(super) fn matches(&self, etag: &Option<Etag>) -> bool {
    &self.etag == etag
  }

  pub(super) fn value(&self) -> &ErasedCacheValue {
    &self.value
  }

  pub(super) fn into_value(self) -> ErasedCacheValue {
    self.value
  }
}
