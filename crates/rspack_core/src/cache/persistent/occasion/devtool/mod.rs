use std::{mem::size_of, sync::Arc};

use rayon::prelude::*;
use rspack_cacheable::cacheable;
use rspack_error::Result;
use rustc_hash::{FxHashMap, FxHashSet};

use super::{
  super::{codec::CacheCodec, storage::Storage},
  Occasion,
};
use crate::{CompilationAsset, RayonConsumer};

pub const SCOPE: &str = "occasion_source_map";

#[cacheable]
struct Entry {
  pub asset: CompilationAsset,
  pub source_map: Option<SourceMapAssetEntry>,
}

#[cacheable]
struct SourceMapAssetEntry {
  pub filename: String,
  pub asset: CompilationAsset,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SourceMapDevToolPluginCacheKey {
  filename: Arc<str>,
  version: Arc<str>,
}

impl SourceMapDevToolPluginCacheKey {
  pub fn new(filename: &str, version: &str) -> Option<Self> {
    if version.is_empty() {
      return None;
    }
    Some(Self {
      filename: Arc::from(filename),
      version: Arc::from(version),
    })
  }

  fn to_bytes(&self) -> Vec<u8> {
    let filename = self.filename.as_bytes();
    let version = self.version.as_bytes();
    let mut bytes = Vec::with_capacity(size_of::<u64>() + filename.len() + version.len());
    bytes.extend_from_slice(&(filename.len() as u64).to_le_bytes());
    bytes.extend_from_slice(filename);
    bytes.extend_from_slice(version);
    bytes
  }

  fn from_bytes(bytes: &[u8]) -> Option<Self> {
    let (filename_len, data) = bytes.split_at_checked(size_of::<u64>())?;
    let filename_len = u64::from_le_bytes(filename_len.try_into().ok()?);
    let filename_len = usize::try_from(filename_len).ok()?;
    let (filename, version) = data.split_at_checked(filename_len)?;
    Some(Self {
      filename: Arc::from(String::from_utf8(filename.to_vec()).ok()?),
      version: Arc::from(String::from_utf8(version.to_vec()).ok()?),
    })
  }
}

#[derive(Debug, Default)]
pub struct SourceMapDevToolPluginCacheArtifact {
  entries: FxHashMap<SourceMapDevToolPluginCacheKey, CachedSourceMapDevToolPluginEntry>,
  /// Keys of entries that were added during this build and need to be persisted.
  dirty_keys: Vec<SourceMapDevToolPluginCacheKey>,
}

#[derive(Debug, Clone)]
pub struct CachedSourceMapDevToolPluginEntry {
  pub asset: CompilationAsset,
  pub source_map: Option<CachedSourceMapDevToolPluginAsset>,
}

#[derive(Debug, Clone)]
pub struct CachedSourceMapDevToolPluginAsset {
  pub filename: String,
  pub asset: CompilationAsset,
}

impl SourceMapDevToolPluginCacheArtifact {
  pub fn get(
    &self,
    key: &SourceMapDevToolPluginCacheKey,
  ) -> Option<&CachedSourceMapDevToolPluginEntry> {
    self.entries.get(key)
  }

  pub fn insert(
    &mut self,
    key: SourceMapDevToolPluginCacheKey,
    entry: CachedSourceMapDevToolPluginEntry,
  ) {
    self.dirty_keys.push(key.clone());
    self.entries.insert(key, entry);
  }

  pub fn retain_current_keys(&mut self, current_keys: &FxHashSet<SourceMapDevToolPluginCacheKey>) {
    self.entries.retain(|key, _| current_keys.contains(key));
  }

  pub fn reset_pending_changes(&mut self) {
    self.dirty_keys.clear();
  }
}

#[derive(Debug)]
pub struct SourceMapDevToolPluginOccasion {
  codec: Arc<CacheCodec>,
}

impl SourceMapDevToolPluginOccasion {
  pub fn new(codec: Arc<CacheCodec>) -> Self {
    Self { codec }
  }
}

impl Occasion for SourceMapDevToolPluginOccasion {
  type Artifact = SourceMapDevToolPluginCacheArtifact;

  fn name(&self) -> &'static str {
    "source map"
  }

  #[tracing::instrument(name = "Cache::Occasion::SourceMap::reset", skip_all)]
  fn reset(&self, storage: &mut dyn Storage) {
    storage.reset(SCOPE);
  }

  #[tracing::instrument(name = "Cache::Occasion::SourceMap::save", skip_all)]
  fn save(&self, storage: &mut dyn Storage, artifact: &SourceMapDevToolPluginCacheArtifact) {
    // Only persist entries that were added during this build.
    artifact
      .dirty_keys
      .par_iter()
      .filter_map(|key| {
        let entry = artifact.entries.get(key)?;
        let storage_entry = Entry {
          asset: entry.asset.clone(),
          source_map: entry
            .source_map
            .as_ref()
            .map(|source_map| SourceMapAssetEntry {
              filename: source_map.filename.clone(),
              asset: source_map.asset.clone(),
            }),
        };
        match self.codec.encode(&storage_entry) {
          Ok(bytes) => Some((key.to_bytes(), bytes)),
          Err(err) => {
            tracing::warn!("source map persistent cache encode failed: {:?}", err);
            None
          }
        }
      })
      .consume(|(key, bytes)| {
        storage.set(SCOPE, key, bytes);
      });

    tracing::debug!(
      "saved {} source map persistent cache entries",
      artifact.dirty_keys.len(),
    );
  }

  #[tracing::instrument(name = "Cache::Occasion::SourceMap::recovery", skip_all)]
  async fn recovery(&self, storage: &dyn Storage) -> Result<SourceMapDevToolPluginCacheArtifact> {
    let items = storage.load(SCOPE).await?;
    let mut entries = FxHashMap::default();
    entries.reserve(items.len());

    for (key, value) in items {
      let Some(key) = SourceMapDevToolPluginCacheKey::from_bytes(&key) else {
        tracing::warn!("source map persistent cache key has invalid length");
        continue;
      };
      match self.codec.decode::<Entry>(&value) {
        Ok(entry) => {
          entries.insert(
            key,
            CachedSourceMapDevToolPluginEntry {
              asset: entry.asset,
              source_map: entry
                .source_map
                .map(|source_map| CachedSourceMapDevToolPluginAsset {
                  filename: source_map.filename,
                  asset: source_map.asset,
                }),
            },
          );
        }
        Err(err) => {
          tracing::warn!("source map persistent cache decode failed: {:?}", err);
        }
      }
    }

    tracing::debug!(
      "recovered {} source map persistent cache entries",
      entries.len()
    );
    Ok(SourceMapDevToolPluginCacheArtifact {
      entries,
      dirty_keys: Vec::new(),
    })
  }
}
