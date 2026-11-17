use std::sync::Arc;

use rayon::prelude::*;
use rspack_cacheable::cacheable;
use rspack_error::Result;
use rspack_util::fx_hash::FxDashMap;

use super::{super::storage::Storage, Occasion};
use crate::{
  RayonConsumer,
  cache::{CacheCodec, CachedSourceMapDevToolPluginEntry},
};

pub const SCOPE: &str = "occasion_source_map_dev_tool_plugin";

#[cacheable]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct CacheKey {
  filename: String,
  version: String,
}

#[derive(PartialEq, Eq, Debug)]
enum EntryState {
  Unused,
  Reused,
  Updated,
}

#[derive(Debug)]
struct Entry {
  value: Arc<CachedSourceMapDevToolPluginEntry>,
  state: EntryState,
}

#[derive(Debug, Default)]
pub struct SourceMapDevToolPluginCache {
  entries: FxDashMap<CacheKey, Entry>,
  pending_invalid_entry_keys: Vec<Vec<u8>>,
}

impl SourceMapDevToolPluginCache {
  pub(crate) fn into_hot_cache(mut self) -> Self {
    self.entries.retain(|_, entry| {
      let keep = entry.state != EntryState::Unused;
      entry.state = EntryState::Unused;
      keep
    });
    self.pending_invalid_entry_keys.clear();
    self
  }

  pub fn get(
    &self,
    filename: &str,
    version: &str,
  ) -> Option<Arc<CachedSourceMapDevToolPluginEntry>> {
    let key = CacheKey {
      filename: filename.into(),
      version: version.into(),
    };
    let mut entry = self.entries.get_mut(&key)?;
    if entry.state == EntryState::Unused {
      entry.state = EntryState::Reused;
    }
    Some(Arc::clone(&entry.value))
  }

  pub fn store(&self, filename: &str, version: String, value: CachedSourceMapDevToolPluginEntry) {
    self.entries.insert(
      CacheKey {
        filename: filename.into(),
        version,
      },
      Entry {
        value: Arc::new(value),
        state: EntryState::Updated,
      },
    );
  }

  pub fn invalidate(&self, filename: &str, version: &str) {
    if let Some(mut entry) = self.entries.get_mut(&CacheKey {
      filename: filename.into(),
      version: version.into(),
    }) {
      entry.state = EntryState::Unused;
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

    // Sweep once after every DevTool instance has processed its own asset selection.
    let mut writes = Vec::new();
    for entry in &cache_item.entries {
      match entry.state {
        EntryState::Unused => match self.codec.encode(entry.key()) {
          Ok(key) => storage.remove(SCOPE, &key),
          Err(err) => tracing::warn!("source map persistent cache key encode failed: {:?}", err),
        },
        EntryState::Updated => writes.push((entry.key().clone(), Arc::clone(&entry.value))),
        EntryState::Reused => {}
      }
    }
    writes
      .into_par_iter()
      .filter_map(|(key, value)| {
        match self
          .codec
          .encode(&key)
          .and_then(|key| self.codec.encode(value.as_ref()).map(|value| (key, value)))
        {
          Ok(pair) => Some(pair),
          Err(err) => {
            tracing::warn!("source map persistent cache encode failed: {:?}", err);
            None
          }
        }
      })
      .consume(|(key, value)| storage.set(SCOPE, key, value));
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
          Ok(value) => Ok((
            key,
            Entry {
              value: Arc::new(value),
              state: EntryState::Unused,
            },
          )),
          Err(err) => {
            tracing::warn!("source map persistent cache decode failed: {:?}", err);
            Err(key_bytes)
          }
        }
      })
      .partition_map::<Vec<_>, Vec<_>, _, _, _>(|result| match result {
        Ok(entry) => itertools::Either::Left(entry),
        Err(raw_key) => itertools::Either::Right(raw_key),
      });

    tracing::debug!(
      "recovered {} source map persistent cache entries and scheduled {} invalid entries for removal",
      entries.len(),
      pending_invalid_entry_keys.len(),
    );
    Ok(SourceMapDevToolPluginCache {
      entries: entries.into_iter().collect(),
      pending_invalid_entry_keys,
    })
  }
}
