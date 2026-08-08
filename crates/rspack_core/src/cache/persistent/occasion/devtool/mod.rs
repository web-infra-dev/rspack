use std::{borrow::Cow, sync::Arc};

use cow_utils::CowUtils;
use rayon::prelude::*;
use rspack_cacheable::cacheable;
use rspack_error::Result;
use rspack_sources::{
  BoxSource, CachedSource, ConcatSource, OriginalSource, RawBufferSource, RawStringSource,
  ReplaceSource, Source, SourceExt, SourceMap, SourceMapSource,
};
use rspack_util::base64;
use rustc_hash::{FxHashMap, FxHashSet};

use super::{
  super::{codec::CacheCodec, storage::Storage},
  Occasion,
};
use crate::{AssetInfo, CompilationAsset, RayonConsumer};

pub const SCOPE: &str = "occasion_source_map_dev_tool_plugin";

#[cacheable]
#[derive(Debug)]
struct CacheEntry {
  comments: SourceMapComments,
  source_map_asset: SourceMapAssetCacheData,
}

#[cacheable]
#[derive(Debug, Clone, Default)]
pub struct SourceMapComments {
  pub debug_id_comment: Option<String>,
  pub source_mapping_url_comment: String,
}

#[cacheable]
#[derive(Debug)]
struct SourceMapAssetCacheData {
  filename: Option<String>,
  source_map: SourceMapCacheData,
}

/// Compact source map data cached by `SourceMapDevToolPlugin`.
#[cacheable]
#[derive(Debug, PartialEq, Eq)]
struct SourceMapCacheData {
  file: Option<String>,
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

fn restore_source_map(source: BoxSource, cache_entry: &CacheEntry) -> Option<SourceMap<'static>> {
  SourceMap::with_source(source, |source| {
    let SourceMapCacheData {
      file,
      mappings,
      sources,
      names,
      source_root,
      debug_id,
      ignore_list,
      source_content_indices,
    } = &cache_entry.source_map_asset.source_map;
    let sources_content = restore_sources_content(source, source_content_indices)?;
    let mut source_map = SourceMap::new(
      Cow::Owned(mappings.clone()),
      sources.iter().cloned().map(Cow::Owned).collect(),
      sources_content.into_iter().map(Cow::Borrowed).collect(),
      names.iter().cloned().map(Cow::Owned).collect(),
    );
    source_map.set_file(file.clone().map(Cow::Owned));
    source_map.set_source_root(source_root.clone().map(Cow::Owned));
    source_map.set_debug_id(debug_id.clone().map(Cow::Owned));
    source_map.set_ignore_list(ignore_list.clone().map(Cow::Owned));
    Some(source_map)
  })
}

fn create_source_map_cache_data(
  source: &dyn Source,
  source_map: &SourceMap<'_>,
) -> Option<SourceMapCacheData> {
  let source_content_indices = source_content_indices(source, source_map)?;
  Some(SourceMapCacheData {
    file: source_map.file().map(ToString::to_string),
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

fn create_cache_entry(
  source: &dyn Source,
  comments: &SourceMapComments,
  source_map_filename: Option<&str>,
  source_map: &SourceMap<'static>,
) -> Option<CacheEntry> {
  Some(CacheEntry {
    comments: comments.clone(),
    source_map_asset: SourceMapAssetCacheData {
      filename: source_map_filename.map(ToString::to_string),
      source_map: create_source_map_cache_data(source, source_map)?,
    },
  })
}

fn render_inline_source_mapping_url_comment(
  source_mapping_url_comment: &str,
  source_map: &SourceMap<'_>,
) -> String {
  let source_map_json = source_map.to_json();
  let base64 = base64::encode_to_string(source_map_json.as_bytes());
  let source_map_url = format!("data:application/json;charset=utf-8;base64,{base64}");
  source_mapping_url_comment
    .cow_replace("[url]", &source_map_url)
    .into_owned()
}

fn restore_asset_source(
  source: BoxSource,
  source_map: &SourceMap<'_>,
  comments: &SourceMapComments,
  is_inline: bool,
) -> Option<BoxSource> {
  if is_inline && comments.source_mapping_url_comment.is_empty() {
    return None;
  }
  let source_mapping_url_comment = if comments.source_mapping_url_comment.is_empty() {
    None
  } else if is_inline {
    Some(render_inline_source_mapping_url_comment(
      &comments.source_mapping_url_comment,
      source_map,
    ))
  } else {
    Some(comments.source_mapping_url_comment.clone())
  };
  let comments_count = usize::from(comments.debug_id_comment.is_some())
    + usize::from(source_mapping_url_comment.is_some());
  if comments_count == 0 {
    return Some(source);
  }

  let mut children = Vec::with_capacity(comments_count + 1);
  children.push(source);
  if let Some(debug_id_comment) = &comments.debug_id_comment {
    children.push(RawStringSource::from(debug_id_comment.clone()).boxed());
  }
  if let Some(source_mapping_url_comment) = source_mapping_url_comment {
    children.push(RawStringSource::from(source_mapping_url_comment).boxed());
  }
  Some(ConcatSource::new(children).boxed())
}

#[allow(clippy::type_complexity)]
fn restore_cache_entry(
  source: BoxSource,
  asset_info: AssetInfo,
  cache_entry: &CacheEntry,
) -> Option<(
  CompilationAsset,
  Option<String>,
  SourceMap<'static>,
  SourceMapComments,
)> {
  let source_map = restore_source_map(source.clone(), cache_entry)?;
  let source_map_filename = cache_entry.source_map_asset.filename.clone();
  let source = restore_asset_source(
    source,
    &source_map,
    &cache_entry.comments,
    source_map_filename.is_none(),
  )?;
  Some((
    CompilationAsset::new(Some(source), asset_info),
    source_map_filename,
    source_map,
    cache_entry.comments.clone(),
  ))
}

#[derive(Debug, Default)]
pub struct SourceMapDevToolPluginCacheArtifact {
  entries: FxHashMap<CacheKey, Option<CacheEntry>>,
  pending_writes: Vec<CacheKey>,
  pending_removes: Vec<CacheKey>,
  pending_invalid_entry_keys: Vec<Vec<u8>>,
}

impl SourceMapDevToolPluginCacheArtifact {
  /// Recover mapped assets after the current compilation assets are available.
  /// Source-content restoration is CPU-bound and runs in parallel across assets.
  #[allow(clippy::type_complexity)]
  pub fn take<'a>(
    &mut self,
    items: impl IntoIterator<Item = (&'a str, &'a CompilationAsset)>,
  ) -> Vec<
    Option<(
      CompilationAsset,
      Option<String>,
      SourceMap<'static>,
      SourceMapComments,
    )>,
  > {
    // Memory cache reuses this artifact across rebuilds, so storage mutations
    // from the previous compilation must not be replayed after its save.
    self.pending_writes.clear();
    self.pending_removes.clear();

    let recovered = items
      .into_iter()
      .map(|(filename, asset)| {
        let source = asset.get_source()?.clone();
        let cache_key = CacheKey::new(filename, &asset.info.version)?;
        let cache_entry = self.entries.get_mut(&cache_key).and_then(Option::take)?;
        Some((cache_key, source, asset.get_info().clone(), cache_entry))
      })
      .collect::<Vec<_>>()
      .into_par_iter()
      .map(|item| {
        item.map(|(cache_key, source, asset_info, cache_entry)| {
          let mapped_asset = restore_cache_entry(source, asset_info, &cache_entry);
          (cache_key, cache_entry, mapped_asset)
        })
      })
      .collect::<Vec<_>>();

    recovered
      .into_iter()
      .map(|item| match item {
        Some((cache_key, cache_entry, Some(mapped_asset))) => {
          self.entries.insert(cache_key, Some(cache_entry));
          Some(mapped_asset)
        }
        Some((cache_key, _, None)) => {
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
    items: impl IntoIterator<
      Item = (
        &'a str,
        &'a CompilationAsset,
        &'a SourceMapComments,
        Option<&'a str>,
        &'a SourceMap<'static>,
      ),
    >,
  ) {
    let mut active_keys = FxHashSet::default();
    let uncached_items = items
      .into_iter()
      .filter_map(
        |(filename, asset, comments, source_map_filename, source_map)| {
          let source = asset.get_source()?;
          let cache_key = CacheKey::new(filename, &asset.info.version)?;
          active_keys.insert(cache_key.clone());
          if self
            .entries
            .get(&cache_key)
            .and_then(Option::as_ref)
            .is_some()
          {
            return None;
          }
          Some((
            cache_key,
            source.as_ref(),
            comments,
            source_map_filename,
            source_map,
          ))
        },
      )
      .collect::<Vec<_>>()
      .into_par_iter()
      .filter_map(
        |(cache_key, source, comments, source_map_filename, source_map)| {
          create_cache_entry(source, comments, source_map_filename, source_map)
            .map(|cache_entry| (cache_key, cache_entry))
        },
      )
      .collect::<Vec<_>>();

    for (cache_key, cache_entry) in uncached_items {
      match self.entries.entry(cache_key) {
        std::collections::hash_map::Entry::Occupied(_) => {}
        std::collections::hash_map::Entry::Vacant(vacant) => {
          let cache_key = vacant.key().clone();
          vacant.insert(Some(cache_entry));
          self.pending_writes.push(cache_key);
        }
      }
    }

    // Entries not returned by this compilation are stale.
    let pending_removes = &mut self.pending_removes;
    self.entries.retain(|key, _| {
      if !active_keys.contains(key) {
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
        let cache_entry = artifact.entries.get(key)?.as_ref()?;
        match self.codec.encode(cache_entry) {
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
        match self.codec.decode::<CacheEntry>(&value) {
          Ok(entry) => Ok((key, Some(entry))),
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
