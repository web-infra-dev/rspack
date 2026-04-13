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

/// The value struct of current storage scope
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

/// In-memory representation of cached minimize results.
///
/// Keys are content hashes (computed from source content + plugin options).
/// Values are the minimized source and optional extracted comments.
#[derive(Debug, Default)]
pub struct MinimizeCacheArtifact {
  entries: FxHashMap<Vec<u8>, CachedMinimizeEntry>,
  /// Keys of entries that were added during this build and need to be persisted.
  dirty_keys: Vec<Vec<u8>>,
}

/// A single cached minimize result.
#[derive(Debug, Clone)]
pub struct CachedMinimizeEntry {
  pub source: BoxSource,
  pub extracted_comments: Option<CachedExtractedComments>,
}

/// Cached extracted comments data.
#[derive(Debug, Clone)]
pub struct CachedExtractedComments {
  pub source: BoxSource,
  pub comments_file_name: String,
}

impl MinimizeCacheArtifact {
  /// Look up a cached minimize result by content hash key.
  pub fn get(&self, key: &[u8]) -> Option<&CachedMinimizeEntry> {
    self.entries.get(key)
  }

  /// Insert a new minimize result. Marks the key as dirty for persistence.
  pub fn insert(&mut self, key: Vec<u8>, entry: CachedMinimizeEntry) {
    self.dirty_keys.push(key.clone());
    self.entries.insert(key, entry);
  }
}

/// Minimize Occasion is used to save minimized asset results.
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
  type Artifact = MinimizeCacheArtifact;

  #[tracing::instrument(name = "Cache::Occasion::Minimize::reset", skip_all)]
  fn reset(&self, storage: &mut dyn Storage) {
    storage.reset(SCOPE);
  }

  #[tracing::instrument(name = "Cache::Occasion::Minimize::save", skip_all)]
  fn save(&self, storage: &mut dyn Storage, artifact: &MinimizeCacheArtifact) {
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
          Ok(bytes) => Some((key.clone(), bytes)),
          Err(err) => {
            tracing::warn!("minimize cache encode failed: {:?}", err);
            None
          }
        }
      })
      .consume(|(key, bytes)| {
        storage.set(SCOPE, key, bytes);
      });

    tracing::debug!("saved {} minimize cache entries", artifact.dirty_keys.len());
  }

  #[tracing::instrument(name = "Cache::Occasion::Minimize::recovery", skip_all)]
  async fn recovery(&self, storage: &dyn Storage) -> Result<MinimizeCacheArtifact> {
    let items = storage.load(SCOPE).await?;
    let mut entries = FxHashMap::default();
    entries.reserve(items.len());

    for (key, value) in items {
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
          tracing::warn!("minimize cache decode failed: {:?}", err);
        }
      }
    }

    tracing::debug!("recovered {} minimize cache entries", entries.len());
    Ok(MinimizeCacheArtifact {
      entries,
      dirty_keys: Vec::new(),
    })
  }
}
