use std::sync::Arc;

use rayon::prelude::*;
use rspack_cacheable::cacheable;
use rspack_error::Result;
use rustc_hash::{FxHashMap, FxHashSet};

use super::{super::storage::Storage, Occasion};
use crate::{
  RayonConsumer,
  cache::{CacheCodec, CachedSourceMapDevToolPluginEntry},
};

pub const SCOPE: &str = "occasion_source_map_dev_tool_plugin";

/// Per-asset cache key for `SourceMapDevToolPlugin`.
///
/// This key is only used to distinguish assets inside a valid persistent cache
/// generation. It intentionally does not include `SourceMapDevToolPlugin` or
/// `output` options, even though source map output is option-dependent.
///
/// Config or option changes should invalidate the whole persistent cache
/// generation via `cache.buildDependencies` or `cache.version`. For
/// CLI-loaded configs, rspack-cli injects the resolved config file paths into
/// `cache.buildDependencies`, including files referenced by `extends`. For
/// programmatic API usage, callers that want config-only changes to invalidate
/// persistent cache are expected to provide equivalent build dependencies or
/// a cache version.
#[cacheable]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct CacheKey {
  filename: String,
  version: String,
}

impl CacheKey {
  fn new(filename: &str, version: &str) -> Option<Self> {
    if version.is_empty() {
      return None;
    }
    Some(Self {
      filename: filename.to_string(),
      version: version.to_string(),
    })
  }
}

#[derive(Debug, Default)]
pub struct SourceMapDevToolPluginCache {
  entries: FxHashMap<CacheKey, CachedSourceMapDevToolPluginEntry>,
  pending_writes: Vec<CacheKey>,
  pending_removes: Vec<CacheKey>,
  pending_invalid_entry_keys: Vec<Vec<u8>>,
}

impl SourceMapDevToolPluginCache {
  /// Keep entries across rebuilds. Storage mutations belong to the completed compilation.
  pub(crate) fn into_hot_cache(self) -> Self {
    Self {
      entries: self.entries,
      ..Default::default()
    }
  }

  pub fn get(&self, filename: &str, version: &str) -> Option<&CachedSourceMapDevToolPluginEntry> {
    self.entries.get(&CacheKey::new(filename, version)?)
  }

  pub fn store<'a>(
    &mut self,
    hits: impl Iterator<Item = (&'a str, &'a str)>,
    new_entries: impl Iterator<Item = (&'a str, &'a str, CachedSourceMapDevToolPluginEntry)>,
  ) {
    self.pending_removes.clear();
    self.pending_writes.clear();
    let hits: FxHashSet<_> = hits
      .filter_map(|(filename, version)| CacheKey::new(filename, version))
      .collect();
    self.entries.retain(|key, _| {
      if hits.contains(key) {
        true
      } else {
        self.pending_removes.push(key.clone());
        false
      }
    });
    for (filename, version, entry) in new_entries {
      if let Some(key) = CacheKey::new(filename, version) {
        self.pending_writes.push(key.clone());
        self.entries.insert(key, entry);
      }
    }
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
  type CacheItem = SourceMapDevToolPluginCache;

  fn name(&self) -> &'static str {
    "source map"
  }

  #[tracing::instrument(name = "Cache::Occasion::SourceMap::reset", skip_all)]
  fn reset(&self, storage: &mut dyn Storage) {
    storage.reset(SCOPE);
  }

  #[tracing::instrument(name = "Cache::Occasion::SourceMap::save", skip_all)]
  fn save(&self, storage: &mut dyn Storage, cache_item: &SourceMapDevToolPluginCache) {
    for key in &cache_item.pending_invalid_entry_keys {
      storage.remove(SCOPE, key);
    }

    for key in &cache_item.pending_removes {
      match self.codec.encode(key) {
        Ok(key) => storage.remove(SCOPE, &key),
        Err(err) => {
          tracing::warn!("source map persistent cache key encode failed: {:?}", err);
        }
      }
    }

    cache_item
      .pending_writes
      .par_iter()
      .filter_map(|key| {
        let key_bytes = match self.codec.encode(key) {
          Ok(bytes) => bytes,
          Err(err) => {
            tracing::warn!("source map persistent cache key encode failed: {:?}", err);
            return None;
          }
        };
        let entry = cache_item.entries.get(key)?;
        match self.codec.encode(entry) {
          Ok(bytes) => Some((key_bytes, bytes)),
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
      "saved {} and removed {} source map persistent cache entries",
      cache_item.pending_writes.len(),
      cache_item.pending_removes.len(),
    );
  }

  #[tracing::instrument(name = "Cache::Occasion::SourceMap::recovery", skip_all)]
  async fn recovery(&self, storage: &dyn Storage) -> Result<SourceMapDevToolPluginCache> {
    let items = storage.load(SCOPE).await?;
    let (entries, pending_invalid_entry_keys) = items
      .into_par_iter()
      .map(|(key_bytes, value)| {
        let key = match self.codec.decode::<CacheKey>(&key_bytes) {
          Ok(key) => key,
          Err(err) => {
            tracing::warn!("source map persistent cache key decode failed: {:?}", err);
            return Err(key_bytes);
          }
        };
        match self
          .codec
          .decode::<CachedSourceMapDevToolPluginEntry>(&value)
        {
          Ok(entry) => Ok((key, entry)),
          Err(err) => {
            tracing::warn!("source map persistent cache decode failed: {:?}", err);
            Err(key_bytes)
          }
        }
      })
      .partition_map::<FxHashMap<_, _>, Vec<_>, _, _, _>(|result| match result {
        Ok(entry) => itertools::Either::Left(entry),
        Err(raw_key) => itertools::Either::Right(raw_key),
      });

    tracing::debug!(
      "recovered {} source map persistent cache entries and scheduled {} invalid entries for removal",
      entries.len(),
      pending_invalid_entry_keys.len(),
    );
    Ok(SourceMapDevToolPluginCache {
      entries,
      pending_writes: Vec::new(),
      pending_removes: Vec::new(),
      pending_invalid_entry_keys,
    })
  }
}
