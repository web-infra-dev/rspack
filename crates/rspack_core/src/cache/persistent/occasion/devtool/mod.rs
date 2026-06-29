use std::sync::Arc;

use rayon::prelude::*;
use rspack_cacheable::{
  cacheable,
  with::{AsPreset, AsVec},
};
use rspack_error::Result;
use rspack_sources::{BoxSource, ConcatSource, SourceExt};
use rustc_hash::FxHashMap;

use super::{
  super::{codec::CacheCodec, storage::Storage},
  Occasion,
};
use crate::{AssetInfo, CompilationAsset, RayonConsumer};

pub const SCOPE: &str = "occasion_source_map_dev_tool_plugin";

#[cacheable]
struct Entry {
  #[cacheable(with=AsVec<AsPreset>)]
  pub append: Vec<BoxSource>,
  pub source_map: Option<SourceMapAssetEntry>,
}

#[cacheable]
struct SourceMapAssetEntry {
  pub filename: String,
  #[cacheable(with=AsPreset)]
  pub source: BoxSource,
}

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
pub struct SourceMapDevToolPluginCacheArtifact {
  entries: FxHashMap<CacheKey, Option<CacheEntry>>,
  pending_writes: Vec<CacheKey>,
  pending_removes: Vec<CacheKey>,
}

#[derive(Debug, Clone)]
struct CacheEntry {
  pub asset_append: Vec<BoxSource>,
  pub source_map: Option<(String, BoxSource)>,
}

impl SourceMapDevToolPluginCacheArtifact {
  fn cache_key(filename: &str, asset: &CompilationAsset) -> Option<CacheKey> {
    asset.get_source()?;
    CacheKey::new(filename, &asset.info.version)
  }

  #[allow(clippy::type_complexity)]
  pub fn take(
    &mut self,
    filename: &str,
    asset: &CompilationAsset,
  ) -> Option<(
    CompilationAsset,
    Option<(String, CompilationAsset)>,
    Vec<BoxSource>,
  )> {
    let cache_key = Self::cache_key(filename, asset)?;

    let CacheEntry {
      asset_append,
      source_map,
    } = self.entries.get_mut(&cache_key).and_then(Option::take)?;

    let source = asset.get_source()?.clone();
    let source = if asset_append.is_empty() {
      source
    } else {
      let mut children = Vec::with_capacity(asset_append.len() + 1);
      children.push(source);
      children.extend(asset_append.iter().cloned());
      ConcatSource::new(children).boxed()
    };

    let source_asset = CompilationAsset::new(Some(source), (*asset.info).clone());
    let source_map = source_map.map(|(filename, source)| {
      let mut source_map_asset_info = AssetInfo::default().with_development(Some(true));
      source_map_asset_info.version = asset.info.version.clone();
      (
        filename,
        CompilationAsset::new(Some(source), source_map_asset_info),
      )
    });

    Some((source_asset, source_map, asset_append))
  }

  pub fn store<'a>(
    &mut self,
    items: impl IntoIterator<
      Item = (
        &'a str,
        &'a CompilationAsset,
        &'a [BoxSource],
        Option<(&'a str, &'a CompilationAsset)>,
      ),
    >,
  ) {
    self.pending_writes.clear();
    self.pending_removes.clear();

    let pending_removes = &mut self.pending_removes;
    self.entries.retain(|key, entry| {
      if entry.is_some() {
        pending_removes.push(key.clone());
        false
      } else {
        true
      }
    });

    for item in items {
      let Some((cache_key, cache_entry)) = Self::cache_entry(item) else {
        continue;
      };

      match self.entries.entry(cache_key) {
        std::collections::hash_map::Entry::Occupied(mut occupied) => {
          occupied.insert(Some(cache_entry));
        }
        std::collections::hash_map::Entry::Vacant(vacant) => {
          let cache_key = vacant.key().clone();
          vacant.insert(Some(cache_entry));
          self.pending_writes.push(cache_key);
        }
      }
    }

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

  fn cache_entry(
    item: (
      &str,
      &CompilationAsset,
      &[BoxSource],
      Option<(&str, &CompilationAsset)>,
    ),
  ) -> Option<(CacheKey, CacheEntry)> {
    let (filename, asset, asset_append, source_map) = item;
    let cache_key = Self::cache_key(filename, asset)?;
    let source_map = match source_map {
      Some((filename, asset)) => {
        let source = asset.get_source()?;
        Some((filename.to_string(), source.clone()))
      }
      None => None,
    };

    Some((
      cache_key,
      CacheEntry {
        asset_append: asset_append.to_vec(),
        source_map,
      },
    ))
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
        let entry = artifact.entries.get(key)?.as_ref()?;
        let storage_entry = Entry {
          append: entry.asset_append.clone(),
          source_map: entry
            .source_map
            .as_ref()
            .map(|(filename, source)| SourceMapAssetEntry {
              filename: filename.clone(),
              source: source.clone(),
            }),
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
      "saved {} and removed {} source map persistent cache entries",
      artifact.pending_writes.len(),
      artifact.pending_removes.len(),
    );
  }

  #[tracing::instrument(name = "Cache::Occasion::SourceMap::recovery", skip_all)]
  async fn recovery(&self, storage: &dyn Storage) -> Result<SourceMapDevToolPluginCacheArtifact> {
    let items = storage.load(SCOPE).await?;
    let entries = items
      .into_par_iter()
      .filter_map(|(key, value)| {
        let key = match self.codec.decode::<CacheKey>(&key) {
          Ok(key) => key,
          Err(err) => {
            tracing::warn!("source map persistent cache key decode failed: {:?}", err);
            return None;
          }
        };
        match self.codec.decode::<Entry>(&value) {
          Ok(entry) => Some((
            key,
            Some(CacheEntry {
              asset_append: entry.append,
              source_map: entry
                .source_map
                .map(|source_map| (source_map.filename, source_map.source)),
            }),
          )),
          Err(err) => {
            tracing::warn!("source map persistent cache decode failed: {:?}", err);
            None
          }
        }
      })
      .collect::<FxHashMap<CacheKey, Option<CacheEntry>>>();

    tracing::debug!(
      "recovered {} source map persistent cache entries",
      entries.len()
    );
    Ok(SourceMapDevToolPluginCacheArtifact {
      entries,
      pending_writes: Vec::new(),
      pending_removes: Vec::new(),
    })
  }
}
