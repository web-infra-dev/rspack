use rspack_cacheable::cacheable;
use rspack_collections::Identifiable;
use rspack_error::Result;
use rspack_hash::{HashFunction, RspackHasher};
use rspack_loader_runner::{Content, LoaderContext};
use rspack_sources::SourceMap;

use crate::{CacheFacade, CacheValue, Etag, ItemCacheFacade, Module, RunnerContext};

fn loader_cache_key(module_identifier: &str, loader_name: &str) -> String {
  fn push_segment(key: &mut String, segment: &str) {
    key.push('|');
    key.push_str(&segment.len().to_string());
    key.push(':');
    key.push_str(segment);
  }

  let mut key = String::new();
  push_segment(&mut key, module_identifier);
  push_segment(&mut key, loader_name);
  key
}

#[doc(hidden)]
pub fn loader_cache_item(
  storage: &CacheFacade,
  module_identifier: &str,
  loader_name: &str,
  etag: Etag,
) -> ItemCacheFacade {
  let key = loader_cache_key(module_identifier, loader_name);
  storage.get_item_cache(&key, Some(etag))
}

#[cacheable]
#[derive(Clone)]
struct LoaderCacheEntry {
  content: Option<Vec<u8>>,
  content_is_string: bool,
  source_map: Option<String>,
}

pub(crate) struct LoaderCacheMissState {
  etag: Etag,
  diagnostics_len: usize,
}

pub(crate) enum LoaderCacheAction {
  Disabled,
  Hit,
  Miss(Box<LoaderCacheMissState>),
}

fn cache_miss_action(context: &LoaderContext<RunnerContext>, etag: Etag) -> LoaderCacheAction {
  LoaderCacheAction::Miss(Box::new(LoaderCacheMissState {
    etag,
    diagnostics_len: context.diagnostics.len(),
  }))
}

fn input_etag(context: &LoaderContext<RunnerContext>) -> Option<Etag> {
  let loader = context.current_loader();
  let mut hasher = RspackHasher::new(&HashFunction::Xxhash64);
  rspack_hash::rspack_hash_object!(&mut hasher, {
    "content" => context.content()?,
    "options" => loader.options_cache_key(),
    "version" => loader.loader_version(),
  });
  Some(Etag::from(format!("{:016x}", hasher.finish())))
}

pub(crate) fn before_normal_loader(
  context: &mut LoaderContext<RunnerContext>,
) -> Result<LoaderCacheAction> {
  if !context.current_loader().cache() || !context.cacheable {
    return Ok(LoaderCacheAction::Disabled);
  }
  // The minimal cache only supports loaders whose observable input is content.
  if context.additional_data().is_some()
    || !context.parse_meta.is_empty()
    || !context.context.module.build_info().assets.is_empty()
    || !context.context.module.build_info().extras.is_empty()
  {
    return Ok(LoaderCacheAction::Disabled);
  }
  let Some(etag) = input_etag(context) else {
    return Ok(LoaderCacheAction::Disabled);
  };
  let loader_name = context.current_loader().loader_name();
  let module_identifier = context.context.module.identifier();
  let item_cache = loader_cache_item(
    &context.context.loader_cache,
    module_identifier.as_str(),
    loader_name,
    etag.clone(),
  );

  if let Some(entry) = item_cache.get::<LoaderCacheEntry>()? {
    let content = match (&entry.content, entry.content_is_string) {
      (Some(content), true) => {
        // SAFETY: String cache entries are written exclusively from `Content::String`.
        let content = unsafe { String::from_utf8_unchecked(content.clone()) };
        Some(Content::String(content))
      }
      (Some(content), false) => Some(Content::Buffer(content.clone())),
      (None, _) => None,
    };
    let source_map = entry
      .source_map
      .clone()
      .and_then(|source_map| SourceMap::from_json(source_map).ok());
    context.__finish_with((content, source_map, None));
    return Ok(LoaderCacheAction::Hit);
  }

  Ok(cache_miss_action(context, etag))
}

pub(crate) fn after_normal_loader(
  context: &LoaderContext<RunnerContext>,
  state: &LoaderCacheMissState,
) -> Result<()> {
  if !context.cacheable
    || context.diagnostics.len() != state.diagnostics_len
    || !context.context.module.build_info().assets.is_empty()
    || !context.context.module.build_info().extras.is_empty()
    || context.additional_data().is_some()
    || !context.parse_meta.is_empty()
  {
    return Ok(());
  }

  // Dynamic loader dependencies are intentionally outside the minimal cache
  // contract and are neither stored nor replayed on a cache hit.
  let (content, content_is_string) = match context.content() {
    Some(Content::String(content)) => (Some(content.as_bytes().to_vec()), true),
    Some(Content::Buffer(content)) => (Some(content.clone()), false),
    None => (None, false),
  };
  let entry = LoaderCacheEntry {
    content,
    content_is_string,
    source_map: context.source_map().map(SourceMap::to_json),
  };
  let loader_name = context.current_loader().loader_name();
  let module_identifier = context.context.module.identifier();
  let item_cache = loader_cache_item(
    &context.context.loader_cache,
    module_identifier.as_str(),
    loader_name,
    state.etag.clone(),
  );
  item_cache.store(CacheValue::new(entry))
}
