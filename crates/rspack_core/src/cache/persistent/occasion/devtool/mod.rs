use std::{mem::size_of, sync::Arc};

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

pub const SCOPE: &str = "occasion_source_map_dev_tool_plugin_v2";

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
/// The plugin options are intentionally not part of this key. Option changes
/// are expected to invalidate the whole persistent cache via
/// `cache.buildDependencies` or `cache.version`, while this key only
/// distinguishes assets within a valid cache generation.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct SourceMapDevToolPluginCacheKey {
  filename: Arc<str>,
  version: Arc<str>,
}

impl SourceMapDevToolPluginCacheKey {
  fn new(filename: &str, version: &str) -> Option<Self> {
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
  entries: FxHashMap<SourceMapDevToolPluginCacheKey, Option<CachedSourceMapDevToolPluginEntry>>,
  pending_writes: Vec<SourceMapDevToolPluginCacheKey>,
  pending_removes: Vec<SourceMapDevToolPluginCacheKey>,
}

#[derive(Debug, Clone)]
pub struct CachedSourceMapDevToolPluginEntry {
  pub asset_append: Vec<BoxSource>,
  pub source_map: Option<CachedSourceMapDevToolPluginAsset>,
}

#[derive(Debug, Clone)]
pub struct CachedSourceMapDevToolPluginAsset {
  pub filename: String,
  pub source: BoxSource,
}

impl SourceMapDevToolPluginCacheArtifact {
  fn cache_key(filename: &str, asset: &CompilationAsset) -> Option<SourceMapDevToolPluginCacheKey> {
    asset.get_source()?;
    SourceMapDevToolPluginCacheKey::new(filename, &asset.info.version)
  }

  pub fn take(
    &mut self,
    filename: &str,
    asset: &CompilationAsset,
  ) -> Option<(
    CompilationAsset,
    Option<(String, CompilationAsset)>,
    Vec<BoxSource>,
  )> {
    let Some(cache_key) = Self::cache_key(filename, asset) else {
      return None;
    };

    let CachedSourceMapDevToolPluginEntry {
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
    let source_map = source_map.map(|source_map| {
      let mut source_map_asset_info = AssetInfo::default().with_development(Some(true));
      source_map_asset_info.version = asset.info.version.clone();
      (
        source_map.filename,
        CompilationAsset::new(Some(source_map.source), source_map_asset_info),
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
      let Some((cache_key, entry)) = Self::cache_entry_from_store_item(item) else {
        continue;
      };

      match self.entries.entry(cache_key) {
        std::collections::hash_map::Entry::Occupied(mut occupied) => {
          occupied.insert(Some(entry));
        }
        std::collections::hash_map::Entry::Vacant(vacant) => {
          let cache_key = vacant.key().clone();
          vacant.insert(Some(entry));
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

  pub fn reset_pending_changes(&mut self) {
    self.pending_writes.clear();
    self.pending_removes.clear();
  }

  fn cache_entry_from_store_item(
    item: (
      &str,
      &CompilationAsset,
      &[BoxSource],
      Option<(&str, &CompilationAsset)>,
    ),
  ) -> Option<(
    SourceMapDevToolPluginCacheKey,
    CachedSourceMapDevToolPluginEntry,
  )> {
    let (filename, asset, asset_append, source_map) = item;
    let cache_key = Self::cache_key(filename, asset)?;
    let source_map = match source_map {
      Some((filename, asset)) => {
        let source = asset.get_source()?;
        Some(CachedSourceMapDevToolPluginAsset {
          filename: filename.to_string(),
          source: source.clone(),
        })
      }
      None => None,
    };

    Some((
      cache_key,
      CachedSourceMapDevToolPluginEntry {
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
      storage.remove(SCOPE, &key.to_bytes());
    }

    artifact
      .pending_writes
      .par_iter()
      .filter_map(|key| {
        let entry = artifact.entries.get(key)?.as_ref()?;
        let storage_entry = Entry {
          append: entry.asset_append.clone(),
          source_map: entry
            .source_map
            .as_ref()
            .map(|source_map| SourceMapAssetEntry {
              filename: source_map.filename.clone(),
              source: source_map.source.clone(),
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
        let Some(key) = SourceMapDevToolPluginCacheKey::from_bytes(&key) else {
          tracing::warn!("source map persistent cache key has invalid length");
          return None;
        };
        match self.codec.decode::<Entry>(&value) {
          Ok(entry) => Some((
            key,
            Some(CachedSourceMapDevToolPluginEntry {
              asset_append: entry.append,
              source_map: entry
                .source_map
                .map(|source_map| CachedSourceMapDevToolPluginAsset {
                  filename: source_map.filename,
                  source: source_map.source,
                }),
            }),
          )),
          Err(err) => {
            tracing::warn!("source map persistent cache decode failed: {:?}", err);
            None
          }
        }
      })
      .collect::<FxHashMap<
        SourceMapDevToolPluginCacheKey,
        Option<CachedSourceMapDevToolPluginEntry>,
      >>();

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
