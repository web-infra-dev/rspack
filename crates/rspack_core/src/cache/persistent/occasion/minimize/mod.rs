use std::sync::Arc;

use rayon::prelude::*;
use rspack_cacheable::{cacheable, with::AsPreset};
use rspack_error::Result;
use rspack_sources::BoxSource;
use rustc_hash::FxHashMap;

use super::{
  super::{codec::CacheCodec, storage::Storage},
  Occasion,
};
use crate::RayonConsumer;

pub const SCOPE: &str = "occasion_minimize";

/// Identity of the asset that produced a cached minimize entry.
///
/// `MinimizeCacheKey` folds these values into a single `u64`, which is lossy:
/// a key that has been swapped, truncated, or collided still parses into a
/// well-formed key. Persisting the identity beside the value lets recovery
/// reject an entry that does not belong to the asset that requested it.
#[cacheable]
#[derive(Debug, Clone)]
pub struct EntryIdentity {
  pub filename: String,
  pub options_hash: u64,
  pub is_module: Option<bool>,
}

#[cacheable]
struct Entry {
  pub identity: EntryIdentity,
  #[cacheable(with=AsPreset)]
  pub source: BoxSource,
  pub extracted_comments: Option<ExtractedCommentsEntry>,
}

#[cacheable]
struct ExtractedCommentsEntry {
  #[cacheable(with=AsPreset)]
  pub source: BoxSource,
  pub comments_file_name: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MinimizeCacheKey(u64);

impl MinimizeCacheKey {
  pub fn new(hash: u64) -> Self {
    Self(hash)
  }

  fn to_bytes(self) -> Vec<u8> {
    self.0.to_ne_bytes().to_vec()
  }

  fn from_bytes(bytes: &[u8]) -> Option<Self> {
    <[u8; 8]>::try_from(bytes)
      .ok()
      .map(|b| Self(u64::from_ne_bytes(b)))
  }
}

#[derive(Debug, Default)]
pub struct MinimizePersistentCache {
  entries: FxHashMap<MinimizeCacheKey, (EntryIdentity, CachedMinimizeEntry)>,
  /// Keys of entries that were added during this build and need to be persisted.
  dirty_keys: Vec<MinimizeCacheKey>,
}

#[cacheable]
#[derive(Debug, Clone)]
pub struct CachedMinimizeEntry {
  #[cacheable(with=AsPreset)]
  pub source: BoxSource,
  pub extracted_comments: Option<CachedExtractedComments>,
}

#[cacheable]
#[derive(Debug, Clone)]
pub struct CachedExtractedComments {
  #[cacheable(with=AsPreset)]
  pub source: BoxSource,
  pub comments_file_name: String,
}

impl MinimizePersistentCache {
  /// Look up a cached minimize result.
  ///
  /// The stored identity is compared against the asset that is asking for it.
  /// A mismatch means the key/value association in storage is not trustworthy,
  /// so the entry is treated as a miss and the asset is minimized again.
  pub fn get(
    &self,
    key: MinimizeCacheKey,
    filename: &str,
    options_hash: u64,
    is_module: Option<bool>,
  ) -> Option<&CachedMinimizeEntry> {
    let (identity, entry) = self.entries.get(&key)?;
    if identity.filename != filename
      || identity.options_hash != options_hash
      || identity.is_module != is_module
    {
      tracing::warn!(
        "minimize persistent cache entry does not belong to {}; treating as a miss",
        filename
      );
      return None;
    }
    Some(entry)
  }

  pub fn insert(
    &mut self,
    key: MinimizeCacheKey,
    identity: EntryIdentity,
    entry: CachedMinimizeEntry,
  ) {
    self.dirty_keys.push(key);
    self.entries.insert(key, (identity, entry));
  }
}

#[derive(Debug)]
pub struct MinimizeOccasion {
  codec: Arc<CacheCodec>,
}

impl MinimizeOccasion {
  pub fn new(codec: Arc<CacheCodec>) -> Self {
    Self { codec }
  }
}

impl Occasion for MinimizeOccasion {
  type CacheItem = MinimizePersistentCache;

  fn name(&self) -> &'static str {
    "minimize"
  }

  #[tracing::instrument(name = "Cache::Occasion::Minimize::reset", skip_all)]
  fn reset(&self, storage: &mut dyn Storage) {
    storage.reset(SCOPE);
  }

  #[tracing::instrument(name = "Cache::Occasion::Minimize::save", skip_all)]
  fn save(&self, storage: &mut dyn Storage, cache_item: &MinimizePersistentCache) {
    // Only persist entries that were added during this build.
    cache_item
      .dirty_keys
      .par_iter()
      .filter_map(|key| {
        let (identity, entry) = cache_item.entries.get(key)?;
        let storage_entry = Entry {
          identity: identity.clone(),
          source: entry.source.clone(),
          extracted_comments: entry
            .extracted_comments
            .as_ref()
            .map(|ec| ExtractedCommentsEntry {
              source: ec.source.clone(),
              comments_file_name: ec.comments_file_name.clone(),
            }),
        };
        match self.codec.encode(&storage_entry) {
          Ok(bytes) => Some((key.to_bytes(), bytes)),
          Err(err) => {
            tracing::warn!("minimize persistent cache encode failed: {:?}", err);
            None
          }
        }
      })
      .consume(|(key, bytes)| {
        storage.set(SCOPE, key, bytes);
      });

    tracing::debug!(
      "saved {} minimize persistent cache entries",
      cache_item.dirty_keys.len()
    );
  }

  #[tracing::instrument(name = "Cache::Occasion::Minimize::recovery", skip_all)]
  async fn recovery(&self, storage: &dyn Storage) -> Result<MinimizePersistentCache> {
    let items = storage.load(SCOPE).await?;
    let mut entries = FxHashMap::default();
    entries.reserve(items.len());

    for (key, value) in items {
      let Some(key) = MinimizeCacheKey::from_bytes(&key) else {
        tracing::warn!("minimize persistent cache key has invalid length");
        continue;
      };
      match self.codec.decode::<Entry>(&value) {
        Ok(entry) => {
          entries.insert(
            key,
            (
              entry.identity,
              CachedMinimizeEntry {
                source: entry.source,
                extracted_comments: entry.extracted_comments.map(|ec| CachedExtractedComments {
                  source: ec.source,
                  comments_file_name: ec.comments_file_name,
                }),
              },
            ),
          );
        }
        Err(err) => {
          tracing::warn!("minimize persistent cache decode failed: {:?}", err);
        }
      }
    }

    tracing::debug!(
      "recovered {} minimize persistent cache entries",
      entries.len()
    );
    Ok(MinimizePersistentCache {
      entries,
      dirty_keys: Vec::new(),
    })
  }
}

#[cfg(test)]
mod tests {
  use rspack_sources::{RawStringSource, SourceExt};

  use super::*;
  use crate::cache::persistent::storage::MemoryStorage;

  const OPTIONS_HASH: u64 = 7;

  fn entry(filename: &str, body: &str) -> Entry {
    Entry {
      identity: EntryIdentity {
        filename: filename.to_string(),
        options_hash: OPTIONS_HASH,
        is_module: Some(false),
      },
      source: RawStringSource::from(body.to_string()).boxed(),
      extracted_comments: None,
    }
  }

  /// Storage verifies pack integrity, not the key/value association inside a
  /// pack. A recovered entry whose identity does not match the asset asking
  /// for it must be discarded rather than returned as that asset's output.
  #[tokio::test]
  async fn recovery_should_reject_cross_wired_entries() -> Result<()> {
    let codec = Arc::new(CacheCodec::new(None));
    let occasion = MinimizeOccasion::new(codec.clone());
    let mut storage = MemoryStorage::default();

    let key_a = MinimizeCacheKey::new(1);
    let key_b = MinimizeCacheKey::new(2);

    // Swap the two keys, as a truncated write or a partially restored cache can.
    storage.set(SCOPE, key_b.to_bytes(), codec.encode(&entry("a.js", "A"))?);
    storage.set(SCOPE, key_a.to_bytes(), codec.encode(&entry("b.js", "B"))?);

    let cache = occasion.recovery(&storage).await?;
    assert!(
      cache
        .get(key_a, "a.js", OPTIONS_HASH, Some(false))
        .is_none()
    );
    assert!(
      cache
        .get(key_b, "b.js", OPTIONS_HASH, Some(false))
        .is_none()
    );

    Ok(())
  }

  #[tokio::test]
  async fn recovery_should_keep_matching_entries() -> Result<()> {
    let codec = Arc::new(CacheCodec::new(None));
    let occasion = MinimizeOccasion::new(codec.clone());
    let mut storage = MemoryStorage::default();

    let key_a = MinimizeCacheKey::new(1);
    storage.set(SCOPE, key_a.to_bytes(), codec.encode(&entry("a.js", "A"))?);

    let cache = occasion.recovery(&storage).await?;
    let cached = cache
      .get(key_a, "a.js", OPTIONS_HASH, Some(false))
      .expect("matching entry should be a cache hit");
    assert_eq!(cached.source.source().into_string_lossy(), "A");

    Ok(())
  }

  #[tokio::test]
  async fn recovery_should_reject_entries_with_stale_options() -> Result<()> {
    let codec = Arc::new(CacheCodec::new(None));
    let occasion = MinimizeOccasion::new(codec.clone());
    let mut storage = MemoryStorage::default();

    let key_a = MinimizeCacheKey::new(1);
    storage.set(SCOPE, key_a.to_bytes(), codec.encode(&entry("a.js", "A"))?);

    let cache = occasion.recovery(&storage).await?;
    assert!(
      cache
        .get(key_a, "a.js", OPTIONS_HASH + 1, Some(false))
        .is_none()
    );
    assert!(cache.get(key_a, "a.js", OPTIONS_HASH, Some(true)).is_none());

    Ok(())
  }
}
