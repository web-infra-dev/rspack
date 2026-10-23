pub mod devtool;
pub mod make;
pub mod minimize;

use std::future::Future;

pub use devtool::{SourceMapDevToolPluginCache, SourceMapDevToolPluginOccasion};
pub use make::MakeOccasion;
pub use minimize::{MinimizeOccasion, MinimizePersistentCache};
use rspack_cacheable::{
  __private::rkyv::{Archive, Deserialize, Serialize, bytecheck::CheckBytes},
  Deserializer, Serializer, Validator, cacheable,
};
use rspack_error::Result;

use super::storage::Storage;
use crate::{
  CompilationLogger,
  cache::{CacheCodec, RootBinding},
};

/// Envelope that records which root an entry is bound to. Only written in portable mode;
/// without a portable root, entries keep the plain encoding.
#[cacheable]
struct BoundEntry {
  bound_root: Option<u64>,
  bytes: Vec<u8>,
}

impl BoundEntry {
  fn encode<T>(codec: &CacheCodec, value: &T, root_sensitive: bool) -> Result<Vec<u8>>
  where
    T: for<'a> Serialize<Serializer<'a>>,
  {
    let Some(root_hash) = codec.root_hash() else {
      return codec.encode(value);
    };
    let (bytes, binding) = codec.encode_with_binding(value)?;
    let bound_root = (root_sensitive || matches!(binding, RootBinding::Bound)).then_some(root_hash);
    codec.encode(&Self { bound_root, bytes })
  }

  /// Returns `None` when the entry is bound to a different root.
  fn decode<T>(codec: &CacheCodec, bytes: &[u8]) -> Result<Option<T>>
  where
    T: Archive,
    T::Archived: for<'a> CheckBytes<Validator<'a>> + Deserialize<T, Deserializer>,
  {
    let Some(root_hash) = codec.root_hash() else {
      return codec.decode(bytes).map(Some);
    };
    let entry: Self = codec.decode(bytes)?;
    if entry
      .bound_root
      .is_some_and(|bound_root| bound_root != root_hash)
    {
      return Ok(None);
    }
    codec.decode(&entry.bytes).map(Some)
  }
}

/// A recovered cache item and the storage keys of entries dropped because they are bound to
/// another root.
pub struct Recovered<T> {
  pub item: T,
  pub dropped: Vec<Vec<u8>>,
}

/// An `Occasion` represents a distinct phase of the persistent cache lifecycle.
///
/// Each occasion owns one storage scope and is responsible for:
/// - serialising its cache item into storage (`save`)
/// - deserialising its cache item from storage (`recovery`)
/// - clearing its scope when the cached data is stale (`reset`)
///
/// `BuildDeps` and `Snapshot` are not occasions: they operate across multiple
/// scopes and have more complex lifecycle semantics.
pub trait Occasion {
  /// The data produced/consumed by this occasion.
  type CacheItem: Send;

  /// Human-readable occasion name used in persistent cache logging.
  fn name(&self) -> &'static str;

  /// Clear this occasion's scope in storage.
  fn reset(&self, storage: &mut dyn Storage);

  /// Persist `cache_item` into storage. Only called when not in readonly mode.
  fn save(&self, storage: &mut dyn Storage, cache_item: &Self::CacheItem);

  /// Load and reconstruct the cache item from storage.
  fn recovery<'a>(
    &'a self,
    storage: &'a dyn Storage,
    logger: &'a CompilationLogger,
  ) -> impl Future<Output = Result<Self::CacheItem>> + Send + 'a;
}
