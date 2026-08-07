use std::{any::Any, fmt, ops::Deref, sync::Arc};

use rspack_cacheable::{
  __private::rkyv::{Archive, Deserialize, Serialize, bytecheck::CheckBytes},
  Deserializer, Serializer, Validator, cacheable,
};
use rspack_error::Result;

use super::etag::Etag;
use crate::cache::persistent::codec::CacheCodec;

/// Shared immutable cache value.
///
/// Cloning this wrapper only increments an `Arc`; the cached object itself is
/// never cloned.
pub struct CacheValue<T>(Arc<T>);

impl<T> CacheValue<T> {
  pub fn new(value: T) -> Self {
    Self(Arc::new(value))
  }

  pub fn as_arc(&self) -> &Arc<T> {
    &self.0
  }

  pub fn into_arc(self) -> Arc<T> {
    self.0
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
    Self(value)
  }
}

/// Internal marker automatically implemented for cacheable values.
pub(crate) trait CacheValueData:
  Any
  + Send
  + Sync
  + Sized
  + Archive<Archived: for<'a> CheckBytes<Validator<'a>> + Deserialize<Self, Deserializer>>
  + for<'a> Serialize<Serializer<'a>>
{
}

impl<T> CacheValueData for T
where
  T: Any + Send + Sync + Archive + for<'a> Serialize<Serializer<'a>>,
  T::Archived: for<'a> CheckBytes<Validator<'a>> + Deserialize<T, Deserializer>,
{
}

pub(super) type CacheValueEncoder = fn(&CacheEntry, &CacheCodec) -> Result<Vec<u8>>;
pub(super) type CacheValueDecoder =
  fn(&[u8], Option<&Etag>, &CacheCodec) -> Result<Option<ErasedCacheValue>>;

/// Type-erased value used only inside the shared cache layers.
#[derive(Clone)]
pub(super) struct ErasedCacheValue(Arc<dyn Any + Send + Sync>);

impl ErasedCacheValue {
  fn new<T: Any + Send + Sync>(value: Arc<T>) -> Self {
    Self(value)
  }

  pub(super) fn downcast<T: Any + Send + Sync>(self) -> Option<CacheValue<T>> {
    Arc::downcast(self.0).ok().map(CacheValue::from)
  }
}

impl fmt::Debug for ErasedCacheValue {
  fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
    formatter
      .debug_struct("ErasedCacheValue")
      .finish_non_exhaustive()
  }
}

#[cacheable]
struct StoredCacheEntry<T> {
  etag: Option<Etag>,
  value: Arc<T>,
}

#[derive(Debug)]
pub(super) struct CacheEntry {
  etag: Option<Etag>,
  value: ErasedCacheValue,
}

impl CacheEntry {
  pub(super) fn new(etag: Option<Etag>, value: ErasedCacheValue) -> Self {
    Self { etag, value }
  }

  pub(super) fn matches(&self, etag: Option<&Etag>) -> bool {
    self.etag.as_ref() == etag
  }

  pub(super) fn value(&self) -> &ErasedCacheValue {
    &self.value
  }
}

#[allow(private_bounds)]
impl<T: CacheValueData> CacheValue<T> {
  pub(super) fn erase(self) -> ErasedCacheValue {
    ErasedCacheValue::new(self.0)
  }

  pub(super) fn encoder() -> CacheValueEncoder {
    encode_cache_entry::<T>
  }

  pub(super) fn decoder() -> CacheValueDecoder {
    decode_cache_entry::<T>
  }
}

fn encode_cache_entry<T: CacheValueData>(
  entry: &CacheEntry,
  codec: &CacheCodec,
) -> Result<Vec<u8>> {
  let value = entry
    .value
    .0
    .clone()
    .downcast::<T>()
    .map_err(|_| rspack_error::error!("Cache value type mismatch"))?;
  codec.encode(&StoredCacheEntry {
    etag: entry.etag.clone(),
    value,
  })
}

fn decode_cache_entry<T: CacheValueData>(
  bytes: &[u8],
  etag: Option<&Etag>,
  codec: &CacheCodec,
) -> Result<Option<ErasedCacheValue>> {
  let entry = codec.decode::<StoredCacheEntry<T>>(bytes)?;
  Ok((entry.etag.as_ref() == etag).then(|| ErasedCacheValue::new(entry.value)))
}
