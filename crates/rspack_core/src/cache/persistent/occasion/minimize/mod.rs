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

#[cacheable]
struct Entry {
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
pub struct MinimizePersistentCacheArtifact {
  entries: FxHashMap<MinimizeCacheKey, CachedMinimizeEntry>,
  /// Keys of entries that were added during this build and need to be persisted.
  dirty_keys: Vec<MinimizeCacheKey>,
}

#[derive(Debug, Clone)]
pub struct CachedMinimizeEntry {
  pub source: BoxSource,
  pub extracted_comments: Option<CachedExtractedComments>,
}

#[derive(Debug, Clone)]
pub struct CachedExtractedComments {
  pub source: BoxSource,
  pub comments_file_name: String,
}

impl MinimizePersistentCacheArtifact {
  pub fn get(&self, key: MinimizeCacheKey) -> Option<&CachedMinimizeEntry> {
    self.entries.get(&key)
  }

  pub fn insert(&mut self, key: MinimizeCacheKey, entry: CachedMinimizeEntry) {
    self.dirty_keys.push(key);
    self.entries.insert(key, entry);
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

#[async_trait::async_trait]
impl Occasion for MinimizeOccasion {
  type Artifact = MinimizePersistentCacheArtifact;

  fn name(&self) -> &'static str {
    "minimize"
  }

  #[tracing::instrument(name = "Cache::Occasion::Minimize::reset", skip_all)]
  fn reset(&self, storage: &mut dyn Storage) {
    storage.reset(SCOPE);
  }

  #[tracing::instrument(name = "Cache::Occasion::Minimize::save", skip_all)]
  fn save(&self, storage: &mut dyn Storage, artifact: &MinimizePersistentCacheArtifact) {
    // Only persist entries that were added during this build.
    artifact
      .dirty_keys
      .par_iter()
      .filter_map(|key| {
        let entry = artifact.entries.get(key)?;
        let storage_entry = Entry {
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
      artifact.dirty_keys.len()
    );
  }

  #[tracing::instrument(name = "Cache::Occasion::Minimize::recovery", skip_all)]
  async fn recovery(&self, storage: &dyn Storage) -> Result<MinimizePersistentCacheArtifact> {
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
            CachedMinimizeEntry {
              source: entry.source,
              extracted_comments: entry.extracted_comments.map(|ec| CachedExtractedComments {
                source: ec.source,
                comments_file_name: ec.comments_file_name,
              }),
            },
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
    Ok(MinimizePersistentCacheArtifact {
      entries,
      dirty_keys: Vec::new(),
    })
  }
}
