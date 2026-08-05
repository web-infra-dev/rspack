use std::{borrow::Cow, sync::Arc};

use rayon::prelude::*;
use rspack_cacheable::{cacheable, utils::OwnedOrRef};
use rspack_error::Result;
use rspack_sources::{
  BoxSource, CachedSource, ConcatSource, OriginalSource, RawBufferSource, RawStringSource,
  ReplaceSource, Source, SourceMap, SourceMapSource,
};
use rustc_hash::FxHashMap;

use super::{
  super::{codec::CacheCodec, storage::Storage},
  Occasion,
};
use crate::{CompilationAsset, RayonConsumer};

pub const SCOPE: &str = "occasion_source_map_dev_tool_plugin";

#[cacheable]
struct Entry<'a> {
  source_map: OwnedOrRef<'a, SourceMapCacheData>,
}

/// Compact source map data cached by `SourceMapDevToolPlugin`.
#[cacheable]
#[derive(Debug, PartialEq, Eq)]
struct SourceMapCacheData {
  mappings: String,
  sources: Vec<String>,
  names: Vec<String>,
  source_root: Option<String>,
  debug_id: Option<String>,
  ignore_list: Option<Vec<u32>>,
  /// Stable source-content ordinals. `u32::MAX` represents an empty string.
  source_content_indices: Vec<u32>,
}

/// Per-asset cache key for `SourceMapDevToolPlugin`.
///
/// Config or option changes should invalidate the whole persistent cache
/// generation via `cache.buildDependencies` or `cache.version`.
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

/// Visit source contents in the deterministic order used by the compact cache.
/// Changing this order requires invalidating existing persistent cache data.
fn visit_sources_content_by_orders<'a>(source: &'a dyn Source, visitor: &mut dyn FnMut(&'a str)) {
  if let Some(source) = source.as_any().downcast_ref::<OriginalSource>() {
    visitor(source.value());
    return;
  }

  if source.as_any().is::<RawStringSource>() || source.as_any().is::<RawBufferSource>() {
    return;
  }

  if let Some(source) = source.as_any().downcast_ref::<SourceMapSource>() {
    for content in source.source_map().sources_content() {
      visitor(content);
    }
    if let Some(inner_source_map) = source.inner_source_map() {
      for content in inner_source_map.sources_content() {
        visitor(content);
      }
    }
    if let Some(original_source) = source.original_source() {
      visitor(original_source);
    }
    return;
  }

  if let Some(source) = source.as_any().downcast_ref::<ConcatSource>() {
    for child in source.children() {
      visit_sources_content_by_orders(child.as_ref(), visitor);
    }
    return;
  }

  if let Some(source) = source.as_any().downcast_ref::<ReplaceSource>() {
    visit_sources_content_by_orders(source.inner().as_ref(), visitor);
    return;
  }

  if let Some(source) = source.as_any().downcast_ref::<CachedSource>() {
    visit_sources_content_by_orders(source.inner().as_ref(), visitor);
  }
}

fn source_content_indices(source: &dyn Source, source_map: &SourceMap<'_>) -> Option<Vec<u32>> {
  let sources_content = source_map.sources_content();
  if sources_content.iter().all(|content| content.is_empty()) {
    return Some(vec![u32::MAX; sources_content.len()]);
  }

  let mut content_ordinals =
    FxHashMap::with_capacity_and_hasher(sources_content.len(), Default::default());
  let mut ordinal = 0u32;
  visit_sources_content_by_orders(source, &mut |content| {
    content_ordinals.entry(content.as_ptr()).or_insert(ordinal);
    ordinal += 1;
  });

  sources_content
    .iter()
    .map(|content| {
      if content.is_empty() {
        Some(u32::MAX)
      } else {
        content_ordinals.get(&content.as_ptr()).copied()
      }
    })
    .collect()
}

fn restore_sources_content<'a>(source: &'a dyn Source, indices: &[u32]) -> Option<Vec<&'a str>> {
  let mut contents = vec![""; indices.len()];
  let mut requests = indices
    .iter()
    .copied()
    .enumerate()
    .filter_map(|(output_index, ordinal)| (ordinal != u32::MAX).then_some((ordinal, output_index)))
    .collect::<Vec<_>>();
  if requests.is_empty() {
    return Some(contents);
  }
  requests.sort_unstable_by_key(|(ordinal, _)| *ordinal);

  let mut request_index = 0;
  let mut ordinal = 0u32;
  visit_sources_content_by_orders(source, &mut |content| {
    while let Some((requested_ordinal, output_index)) = requests.get(request_index).copied() {
      if requested_ordinal != ordinal {
        break;
      }
      contents[output_index] = content;
      request_index += 1;
    }
    ordinal += 1;
  });
  (request_index == requests.len()).then_some(contents)
}

fn restore_source_map(
  source: BoxSource,
  source_map_cache_data: SourceMapCacheData,
) -> Option<SourceMap<'static>> {
  let SourceMapCacheData {
    mappings,
    sources,
    names,
    source_root,
    debug_id,
    ignore_list,
    source_content_indices,
  } = source_map_cache_data;
  SourceMap::with_source(source, move |source| {
    let sources_content = restore_sources_content(source, &source_content_indices)?;
    let mut source_map = SourceMap::new(
      Cow::Owned(mappings),
      sources.into_iter().map(Cow::Owned).collect(),
      sources_content.into_iter().map(Cow::Borrowed).collect(),
      names.into_iter().map(Cow::Owned).collect(),
    );
    source_map.set_source_root(source_root.map(Cow::Owned));
    source_map.set_debug_id(debug_id.map(Cow::Owned));
    source_map.set_ignore_list(ignore_list.map(Cow::Owned));
    Some(source_map)
  })
}

fn create_source_map_cache_data(
  source: &dyn Source,
  source_map: &SourceMap<'_>,
) -> Option<SourceMapCacheData> {
  let source_content_indices = source_content_indices(source, source_map)?;
  Some(SourceMapCacheData {
    mappings: source_map.mappings().to_string(),
    sources: source_map
      .sources()
      .iter()
      .map(|source| source.to_string())
      .collect(),
    names: source_map
      .names()
      .iter()
      .map(|name| name.to_string())
      .collect(),
    source_root: source_map.source_root().map(ToString::to_string),
    debug_id: source_map.get_debug_id().map(ToString::to_string),
    ignore_list: source_map.ignore_list().map(<[u32]>::to_vec),
    source_content_indices,
  })
}

#[derive(Debug, Default)]
pub struct SourceMapDevToolPluginCacheArtifact {
  entries: FxHashMap<CacheKey, Option<SourceMapCacheData>>,
  pending_writes: Vec<CacheKey>,
  pending_removes: Vec<CacheKey>,
  pending_invalid_entry_keys: Vec<Vec<u8>>,
}

impl SourceMapDevToolPluginCacheArtifact {
  /// Recover source maps after the current compilation assets are available.
  /// Source-content restoration is CPU-bound and runs in parallel across
  /// assets, while cache entry state is updated serially.
  pub fn take<'a>(
    &mut self,
    items: impl IntoIterator<Item = (&'a str, &'a CompilationAsset)>,
  ) -> Vec<Option<SourceMap<'static>>> {
    // Memory cache reuses this artifact across rebuilds, so storage mutations
    // from the previous compilation must not be replayed after its save.
    self.pending_writes.clear();
    self.pending_removes.clear();

    let recovered = items
      .into_iter()
      .map(|(filename, asset)| {
        let source = asset.get_source()?.clone();
        let cache_key = CacheKey::new(filename, &asset.info.version)?;
        let source_map_cache_data = self.entries.get_mut(&cache_key).and_then(Option::take)?;
        Some((cache_key, source, source_map_cache_data))
      })
      .collect::<Vec<_>>()
      .into_par_iter()
      .map(|item| {
        item.map(|(cache_key, source, source_map_cache_data)| {
          let source_map = restore_source_map(source, source_map_cache_data);
          (cache_key, source_map)
        })
      })
      .collect::<Vec<_>>();

    recovered
      .into_iter()
      .map(|item| match item {
        Some((_, Some(source_map))) => Some(source_map),
        Some((cache_key, None)) => {
          self.entries.remove(&cache_key);
          self.pending_removes.push(cache_key);
          None
        }
        None => None,
      })
      .collect()
  }

  pub fn store<'a>(
    &mut self,
    items: impl IntoIterator<Item = (&'a str, &'a CompilationAsset, &'a SourceMap<'static>)>,
  ) {
    // Entries that were recovered but not consumed by this compilation are
    // stale. Keep consumed entries (`None`) so valid hits can be reinserted
    // without rewriting their storage value.
    let pending_removes = &mut self.pending_removes;
    self.entries.retain(|key, entry| {
      if entry.is_some() {
        pending_removes.push(key.clone());
        false
      } else {
        true
      }
    });

    let source_map_cache_entries = items
      .into_iter()
      .filter_map(|(filename, asset, source_map)| {
        let source = asset.get_source()?;
        let cache_key = CacheKey::new(filename, &asset.info.version)?;
        Some((cache_key, source.as_ref(), source_map))
      })
      .collect::<Vec<_>>()
      .into_par_iter()
      .filter_map(|(cache_key, source, source_map)| {
        create_source_map_cache_data(source, source_map).map(|source_map| (cache_key, source_map))
      })
      .collect::<Vec<_>>();

    for (cache_key, source_map) in source_map_cache_entries {
      match self.entries.entry(cache_key) {
        std::collections::hash_map::Entry::Occupied(mut occupied) => {
          occupied.insert(Some(source_map));
        }
        std::collections::hash_map::Entry::Vacant(vacant) => {
          let cache_key = vacant.key().clone();
          vacant.insert(Some(source_map));
          self.pending_writes.push(cache_key);
        }
      }
    }

    // Consumed entries that were not reinserted are no longer cacheable.
    let pending_removes = &mut self.pending_removes;
    self.entries.retain(|key, entry| {
      if entry.is_none() {
        pending_removes.push(key.clone());
        false
      } else {
        true
      }
    });
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
    for key in &artifact.pending_invalid_entry_keys {
      storage.remove(SCOPE, key);
    }

    for key in &artifact.pending_removes {
      match self.codec.encode(key) {
        Ok(key) => storage.remove(SCOPE, &key),
        Err(err) => {
          tracing::warn!("source map persistent cache key encode failed: {:?}", err);
        }
      }
    }

    artifact
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
        let source_map = artifact.entries.get(key)?.as_ref()?;
        let storage_entry = Entry {
          source_map: source_map.into(),
        };
        match self.codec.encode(&storage_entry) {
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
      "saved {}, removed {}, and removed {} invalid source map persistent cache entries",
      artifact.pending_writes.len(),
      artifact.pending_removes.len(),
      artifact.pending_invalid_entry_keys.len(),
    );
  }

  #[tracing::instrument(name = "Cache::Occasion::SourceMap::recovery", skip_all)]
  async fn recovery(&self, storage: &dyn Storage) -> Result<SourceMapDevToolPluginCacheArtifact> {
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
        match self.codec.decode::<Entry>(&value) {
          Ok(entry) => Ok((key, Some(entry.source_map.into_owned()))),
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
    Ok(SourceMapDevToolPluginCacheArtifact {
      entries,
      pending_writes: Vec::new(),
      pending_removes: Vec::new(),
      pending_invalid_entry_keys,
    })
  }
}
