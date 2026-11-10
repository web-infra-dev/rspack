use std::sync::Arc;

use rayon::{iter::Either, prelude::*};
use rspack_cacheable::cacheable;
use rspack_error::Result;
use rspack_sources::BoxSource;
use rustc_hash::FxHashMap;

use super::{super::storage::Storage, BoundEntry, Occasion, Recovered};
use crate::{
  CompilationAsset, CompilationLogger, Logger, RayonConsumer,
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
}

impl SourceMapDevToolPluginCache {
  pub fn get(&self, filename: &str, version: &str) -> Option<&CachedSourceMapDevToolPluginEntry> {
    self.entries.get(&CacheKey::new(filename, version)?)
  }

  #[allow(clippy::type_complexity)]
  pub fn restore(
    &self,
    filename: &str,
    asset: &CompilationAsset,
    version: &str,
    name_version: &str,
  ) -> Option<(
    CompilationAsset,
    Option<(String, CompilationAsset)>,
    Vec<BoxSource>,
  )> {
    let entry = self.get(filename, version)?;
    if entry.name_version != name_version {
      return None;
    }
    entry.restore(asset)
  }

  /// Reuse the existing entries after validating the complete map input set.
  pub fn reuse_unchanged(&mut self) {
    self.pending_writes.clear();
    self.pending_removes.clear();
  }

  pub fn store<'a>(
    &mut self,
    items: impl IntoIterator<Item = (&'a str, &'a str, CachedSourceMapDevToolPluginEntry)>,
  ) {
    let entries = items
      .into_iter()
      .filter_map(|(filename, version, entry)| Some((CacheKey::new(filename, version)?, entry)))
      .collect::<FxHashMap<_, _>>();
    self.pending_removes = self
      .entries
      .keys()
      .filter(|key| !entries.contains_key(*key))
      .cloned()
      .collect();
    self.pending_writes = entries
      .iter()
      .filter_map(|(key, entry)| {
        let unchanged = self.entries.get(key).is_some_and(|old| {
          old.name_version == entry.name_version
            && old.compilation_version == entry.compilation_version
        });
        (!unchanged).then(|| key.clone())
      })
      .collect();
    self.entries = entries;
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

  fn scope(&self) -> &'static str {
    SCOPE
  }

  #[tracing::instrument(name = "Cache::Occasion::SourceMap::reset", skip_all)]
  fn reset(&self, storage: &mut dyn Storage) {
    storage.reset(SCOPE);
  }

  #[tracing::instrument(name = "Cache::Occasion::SourceMap::save", skip_all)]
  fn save(&self, storage: &mut dyn Storage, cache_item: &SourceMapDevToolPluginCache) {
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
        match BoundEntry::encode(&self.codec, entry, false) {
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
  async fn recovery(
    &self,
    storage: &dyn Storage,
    logger: &CompilationLogger,
  ) -> Result<Recovered<SourceMapDevToolPluginCache>> {
    let items = storage.load(SCOPE).await?;
    let (entries, dropped): (FxHashMap<_, _>, Vec<_>) = items
      .into_par_iter()
      .filter_map(|(key_bytes, value)| {
        let key = match self.codec.decode::<CacheKey>(&key_bytes) {
          Ok(key) => key,
          Err(err) => {
            logger.warn(format!(
              "source map persistent cache key decode failed: {err}"
            ));
            return None;
          }
        };
        match BoundEntry::decode::<CachedSourceMapDevToolPluginEntry>(&self.codec, &value) {
          Ok(Some(entry)) => Some(Either::Left((key, entry))),
          Ok(None) => Some(Either::Right(key_bytes)),
          Err(err) => {
            logger.warn(format!("source map persistent cache decode failed: {err}"));
            None
          }
        }
      })
      .partition_map(|entry| entry);

    tracing::debug!(
      "recovered {} source map persistent cache entries",
      entries.len()
    );
    Ok(Recovered {
      item: SourceMapDevToolPluginCache {
        entries,
        pending_writes: Vec::new(),
        pending_removes: Vec::new(),
      },
      dropped,
    })
  }
}
