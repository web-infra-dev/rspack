pub mod devtool;
pub mod make;
pub mod minimize;

use std::future::Future;

pub use devtool::{SourceMapDevToolPluginCache, SourceMapDevToolPluginOccasion};
pub use make::MakeOccasion;
pub use minimize::{
  CachedExtractedComments, CachedMinimizeEntry, MinimizeOccasion, MinimizePersistentCache,
};
use rspack_error::Result;

use super::storage::Storage;

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
  ) -> impl Future<Output = Result<Self::CacheItem>> + Send + 'a;
}
